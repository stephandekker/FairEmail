use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use uuid::Uuid;

use crate::core::provider::ProviderDatabase;
use crate::core::reauth::find_oauth_config_for_reauth;
use crate::core::{
    self, apply_custom_order, build_revocation_notification, clear_primary_if_deleted,
    collect_categories, group_by_category, move_account, remove_from_order, Account, AuthMethod,
    ConnectionState, ConnectionStateManager, CredentialRole, CredentialStore, RevocationEvent,
    SecretValue, ValidatedTokenResponse,
};
use crate::services::{AccountStore, AppSettings, SqliteOrderStore, SqliteSettingsStore};

/// Store all credentials for an account into the system keychain.
fn store_account_credentials(cred_store: &dyn CredentialStore, account: &Account) {
    let id = account.id();
    if !account.credential().is_empty() {
        if let Err(e) = cred_store.write(
            id,
            CredentialRole::ImapPassword,
            &SecretValue::new(account.credential().to_string()),
        ) {
            eprintln!("Warning: could not store IMAP credential in keychain: {e}");
        }
    }
    if let Some(smtp) = account.smtp() {
        if !smtp.credential.is_empty() {
            if let Err(e) = cred_store.write(
                id,
                CredentialRole::SmtpPassword,
                &SecretValue::new(smtp.credential.clone()),
            ) {
                eprintln!("Warning: could not store SMTP credential in keychain: {e}");
            }
        }
    }
}
use crate::ui::add_account_dialog;
use crate::ui::edit_account_dialog;
use crate::ui::export_dialog;
use crate::ui::import_dialog;
use crate::ui::import_provider_dialog;
use crate::ui::setup_wizard;
use crate::ui::smtp_identity_dialog;

/// Build the main application window with the account list and navigation pane.
pub(crate) fn build(
    app: &adw::Application,
    store: Rc<AccountStore>,
    settings_store: Rc<SqliteSettingsStore>,
    order_store: Rc<SqliteOrderStore>,
    credential_store: Rc<dyn crate::core::CredentialStore>,
) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(gettextrs::gettext("Fairmail – Accounts"))
        .default_width(720)
        .default_height(480)
        .build();

    let split_view = adw::NavigationSplitView::new();

    // -- Sidebar: account list --
    let sidebar_header = adw::HeaderBar::new();
    let add_btn = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(gettextrs::gettext("Add account"))
        .accessible_role(gtk::AccessibleRole::Button)
        .build();
    sidebar_header.pack_start(&add_btn);

    // FR-47: export button for exporting account configurations.
    let export_btn = gtk::Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text(gettextrs::gettext("Export accounts"))
        .accessible_role(gtk::AccessibleRole::Button)
        .build();
    sidebar_header.pack_start(&export_btn);

    // FR-49: import button for importing account configurations.
    let import_btn = gtk::Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text(gettextrs::gettext("Import accounts"))
        .accessible_role(gtk::AccessibleRole::Button)
        .build();
    sidebar_header.pack_start(&import_btn);

    // FR-26, FR-28: import custom OAuth provider configuration.
    let import_provider_btn = gtk::Button::builder()
        .icon_name("application-x-addon-symbolic")
        .tooltip_text(gettextrs::gettext("Import provider configuration"))
        .accessible_role(gtk::AccessibleRole::Button)
        .build();
    sidebar_header.pack_start(&import_provider_btn);

    // FR-21: toggle button for category grouping in the navigation pane.
    let category_toggle = gtk::ToggleButton::builder()
        .icon_name("view-list-symbolic")
        .tooltip_text(gettextrs::gettext("Group by category"))
        .accessible_role(gtk::AccessibleRole::ToggleButton)
        .build();
    sidebar_header.pack_end(&category_toggle);

    // FR-21, US-20: Reset order button restores default sort.
    let reset_order_btn = gtk::Button::builder()
        .icon_name("view-sort-ascending-symbolic")
        .tooltip_text(gettextrs::gettext("Reset order"))
        .accessible_role(gtk::AccessibleRole::Button)
        .build();
    sidebar_header.pack_end(&reset_order_btn);

    let account_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .css_classes(["navigation-sidebar"])
        .accessible_role(gtk::AccessibleRole::List)
        .build();
    account_list.set_placeholder(Some(
        &gtk::Label::builder()
            .label(gettextrs::gettext("No accounts yet"))
            .css_classes(["dim-label"])
            .margin_top(24)
            .margin_bottom(24)
            .build(),
    ));

    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&account_list)
        .build();

    let sidebar_toolbar = adw::ToolbarView::new();
    sidebar_toolbar.add_top_bar(&sidebar_header);
    sidebar_toolbar.set_content(Some(&scroll));

    let sidebar_page = adw::NavigationPage::builder()
        .title(gettextrs::gettext("Accounts"))
        .child(&sidebar_toolbar)
        .build();

    // -- Content pane (placeholder) --
    let content_header = adw::HeaderBar::new();
    let content_label = gtk::Label::builder()
        .label(gettextrs::gettext("Select an account"))
        .css_classes(["dim-label"])
        .vexpand(true)
        .build();
    let content_toolbar = adw::ToolbarView::new();
    content_toolbar.add_top_bar(&content_header);
    content_toolbar.set_content(Some(&content_label));

    let content_page = adw::NavigationPage::builder()
        .title(gettextrs::gettext("Details"))
        .child(&content_toolbar)
        .build();

    split_view.set_sidebar(Some(&sidebar_page));
    split_view.set_content(Some(&content_page));

    // US-5: Revocation banner for OAuth re-authorization notifications.
    // Hidden by default; shown when a token refresh fails definitively.
    let revocation_banner = adw::Banner::new("");
    revocation_banner.set_button_label(Some(&gettextrs::gettext("Re-authorize")));
    revocation_banner.set_revealed(false);

    let toast_overlay = adw::ToastOverlay::new();

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_box.append(&revocation_banner);
    main_box.append(&split_view);
    toast_overlay.set_child(Some(&main_box));

    window.set_content(Some(&toast_overlay));

    // US-5: track which account the revocation banner refers to.
    let banner_account_id: Rc<RefCell<Option<Uuid>>> = Rc::new(RefCell::new(None));

    // Shared mutable list of accounts for the sidebar model.
    let accounts: Rc<RefCell<Vec<Account>>> = Rc::new(RefCell::new(Vec::new()));

    // Shared mutable custom order (FR-20, AC-8).
    let custom_order: Rc<RefCell<Option<Vec<Uuid>>>> = Rc::new(RefCell::new(None));

    // Per-account connection state and diagnostics (FR-44, FR-45, FR-46, NFR-2).
    let conn_state_mgr: Rc<RefCell<ConnectionStateManager>> =
        Rc::new(RefCell::new(ConnectionStateManager::new()));

    // Load settings and initialise toggle state.
    let settings: Rc<RefCell<AppSettings>> =
        Rc::new(RefCell::new(settings_store.load().unwrap_or_default()));
    category_toggle.set_active(settings.borrow().category_display_enabled);

    // Populate from store on startup.
    {
        let loaded = store.load_all().unwrap_or_default();
        // Initialise connection state for each loaded account (FR-44).
        {
            let mut mgr = conn_state_mgr.borrow_mut();
            for acct in &loaded {
                mgr.ensure_account(acct.id());
            }
        }
        *accounts.borrow_mut() = loaded;
        // Load persisted order (AC-8).
        *custom_order.borrow_mut() = order_store.load().unwrap_or(None);
        rebuild_account_list(
            &account_list,
            &accounts.borrow(),
            settings.borrow().category_display_enabled,
            custom_order.borrow().as_deref(),
            &conn_state_mgr.borrow(),
        );
    }

    // FR-21: toggle category display on/off and persist the setting.
    category_toggle.connect_toggled(clone!(
        #[strong]
        settings_store,
        #[strong]
        settings,
        #[strong]
        accounts,
        #[strong]
        custom_order,
        #[strong]
        conn_state_mgr,
        #[weak]
        account_list,
        move |btn| {
            let enabled = btn.is_active();
            {
                let mut s = settings.borrow_mut();
                s.category_display_enabled = enabled;
                let _ = settings_store.save(&s);
            }
            rebuild_account_list(
                &account_list,
                &accounts.borrow(),
                enabled,
                custom_order.borrow().as_deref(),
                &conn_state_mgr.borrow(),
            );
        }
    ));

    // FR-21, US-20: Reset order to default (primary first, then alphabetical).
    reset_order_btn.connect_clicked(clone!(
        #[strong]
        order_store,
        #[strong]
        accounts,
        #[strong]
        custom_order,
        #[strong]
        settings,
        #[strong]
        conn_state_mgr,
        #[weak]
        account_list,
        move |_| {
            let _ = order_store.clear();
            *custom_order.borrow_mut() = None;
            rebuild_account_list(
                &account_list,
                &accounts.borrow(),
                settings.borrow().category_display_enabled,
                None,
                &conn_state_mgr.borrow(),
            );
        }
    ));

    // -- "Set as Primary" context menu on right-click (FR-24, FR-25) --
    let gesture = gtk::GestureClick::builder()
        .button(3) // right-click
        .build();
    gesture.connect_released(clone!(
        #[strong]
        store,
        #[strong]
        accounts,
        #[strong]
        settings,
        #[strong]
        custom_order,
        #[strong]
        conn_state_mgr,
        #[weak]
        account_list,
        move |gesture, _, x, y| {
            let widget = gesture.widget();
            if let Some(row) = account_list.row_at_y(y as i32) {
                let acct_id = row_account_id(&row);
                if let Some(acct_id) = acct_id {
                    let mut list = accounts.borrow_mut();
                    if let Ok(changed) = core::set_primary(&mut list, acct_id) {
                        if !changed.is_empty() {
                            for changed_id in &changed {
                                if let Some(a) = list.iter().find(|a| a.id() == *changed_id) {
                                    let _ = store.update(a.clone());
                                }
                            }
                        }
                    }
                    drop(list);
                    rebuild_account_list(
                        &account_list,
                        &accounts.borrow(),
                        settings.borrow().category_display_enabled,
                        custom_order.borrow().as_deref(),
                        &conn_state_mgr.borrow(),
                    );
                }
            }
            let _ = (widget, x);
        }
    ));
    account_list.add_controller(gesture);

    // -- Drag-and-drop reordering (FR-20, US-19) --
    // GTK4 DnD: we use DragSource and DropTarget on each row.
    // Since ListBox rows are rebuilt dynamically, we attach DnD in make_account_row.
    // Instead, we use a simpler approach: enable keyboard reorder and track drag via
    // ListBox's built-in row reorder is not directly supported, so we use DragSource
    // on the ListBox and a DropTarget to handle reorder.

    // We set up DnD on the ListBox level using a custom content type.
    let drag_source = gtk::DragSource::builder()
        .actions(gtk4::gdk::DragAction::MOVE)
        .build();

    let dragged_index: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));

    drag_source.connect_prepare(clone!(
        #[strong]
        dragged_index,
        #[weak]
        account_list,
        #[upgrade_or_panic]
        move |_source, x, y| {
            if let Some(row) = account_list.row_at_y(y as i32) {
                let idx = row.index();
                if idx >= 0 && row_account_id(&row).is_some() {
                    *dragged_index.borrow_mut() = Some(idx as usize);
                    let paintable = gtk::WidgetPaintable::new(Some(&row));
                    _source.set_icon(Some(&paintable), x as i32, y as i32);
                    let content = gtk4::gdk::ContentProvider::for_value(&idx.to_value());
                    return Some(content);
                }
            }
            *dragged_index.borrow_mut() = None;
            None
        }
    ));

    account_list.add_controller(drag_source);

    let drop_target = gtk::DropTarget::builder()
        .actions(gtk4::gdk::DragAction::MOVE)
        .build();
    drop_target.set_types(&[glib::Type::I32]);

    drop_target.connect_drop(clone!(
        #[strong]
        dragged_index,
        #[strong]
        accounts,
        #[strong]
        custom_order,
        #[strong]
        order_store,
        #[strong]
        settings,
        #[strong]
        conn_state_mgr,
        #[weak]
        account_list,
        #[upgrade_or]
        false,
        move |_target, _value, _x, y| {
            let from = match *dragged_index.borrow() {
                Some(i) => i,
                None => return false,
            };

            let to = match account_list.row_at_y(y as i32) {
                Some(row) => {
                    let idx = row.index();
                    if idx < 0 {
                        return false;
                    }
                    idx as usize
                }
                None => return false,
            };

            if from == to {
                return false;
            }

            // Build current display order as UUIDs.
            let accts = accounts.borrow();
            let current_order = match custom_order.borrow().as_ref() {
                Some(o) => apply_custom_order(&accts, o)
                    .iter()
                    .map(|&i| accts[i].id())
                    .collect::<Vec<_>>(),
                None => {
                    let sorted = crate::core::sort_accounts_flat(&accts);
                    sorted.iter().map(|&i| accts[i].id()).collect::<Vec<_>>()
                }
            };
            drop(accts);

            let new_order = move_account(&current_order, from, to);
            let _ = order_store.save(&new_order);
            *custom_order.borrow_mut() = Some(new_order);

            rebuild_account_list(
                &account_list,
                &accounts.borrow(),
                settings.borrow().category_display_enabled,
                custom_order.borrow().as_deref(),
                &conn_state_mgr.borrow(),
            );

            true
        }
    ));

    account_list.add_controller(drop_target);

    // -- Keyboard-accessible reordering (NFR-7): Ctrl+Up / Ctrl+Down --
    let key_controller = gtk::EventControllerKey::new();
    key_controller.connect_key_pressed(clone!(
        #[strong]
        accounts,
        #[strong]
        custom_order,
        #[strong]
        order_store,
        #[strong]
        settings,
        #[strong]
        conn_state_mgr,
        #[weak]
        account_list,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_, keyval, _keycode, modifiers| {
            let ctrl = modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK);
            if !ctrl {
                return glib::Propagation::Proceed;
            }

            let is_up = keyval == gtk4::gdk::Key::Up;
            let is_down = keyval == gtk4::gdk::Key::Down;
            if !is_up && !is_down {
                return glib::Propagation::Proceed;
            }

            let selected_row = match account_list.selected_row() {
                Some(r) => r,
                None => return glib::Propagation::Proceed,
            };

            if row_account_id(&selected_row).is_none() {
                return glib::Propagation::Proceed;
            }

            let from = selected_row.index() as usize;

            // Build current display order.
            let accts = accounts.borrow();
            let total_rows = {
                let mut count = 0usize;
                let mut child = account_list.first_child();
                while child.is_some() {
                    count += 1;
                    child = child.unwrap().next_sibling();
                }
                count
            };

            let to = if is_up {
                if from == 0 {
                    return glib::Propagation::Proceed;
                }
                from - 1
            } else {
                if from + 1 >= total_rows {
                    return glib::Propagation::Proceed;
                }
                from + 1
            };

            let current_order = match custom_order.borrow().as_ref() {
                Some(o) => apply_custom_order(&accts, o)
                    .iter()
                    .map(|&i| accts[i].id())
                    .collect::<Vec<_>>(),
                None => {
                    let sorted = crate::core::sort_accounts_flat(&accts);
                    sorted.iter().map(|&i| accts[i].id()).collect::<Vec<_>>()
                }
            };
            drop(accts);

            let new_order = move_account(&current_order, from, to);
            let _ = order_store.save(&new_order);
            *custom_order.borrow_mut() = Some(new_order);

            rebuild_account_list(
                &account_list,
                &accounts.borrow(),
                settings.borrow().category_display_enabled,
                custom_order.borrow().as_deref(),
                &conn_state_mgr.borrow(),
            );

            // Re-select the moved row.
            if let Some(new_row) = account_list.row_at_index(to as i32) {
                account_list.select_row(Some(&new_row));
            }

            glib::Propagation::Stop
        }
    ));
    account_list.add_controller(key_controller);

    // US-5: Revocation banner "Re-authorize" button handler.
    // Triggers the OAuth browser flow directly for one-click re-authorization.
    revocation_banner.connect_button_clicked(clone!(
        #[strong]
        banner_account_id,
        #[strong]
        accounts,
        #[strong]
        store,
        #[strong]
        credential_store,
        #[strong]
        settings,
        #[strong]
        custom_order,
        #[strong]
        conn_state_mgr,
        #[weak]
        revocation_banner,
        #[weak]
        toast_overlay,
        #[weak]
        account_list,
        move |_| {
            let account_id = match *banner_account_id.borrow() {
                Some(id) => id,
                None => return,
            };

            // Look up the account and find its OAuth config.
            let account = {
                let list = accounts.borrow();
                list.iter().find(|a| a.id() == account_id).cloned()
            };
            let account = match account {
                Some(a) => a,
                None => return,
            };

            let provider_db = ProviderDatabase::bundled();
            let oauth_config = match find_oauth_config_for_reauth(&account, &provider_db) {
                Some(cfg) => cfg,
                None => {
                    let toast = adw::Toast::new(&gettextrs::gettext(
                        "Could not find OAuth configuration for this account.",
                    ));
                    toast_overlay.add_toast(toast);
                    return;
                }
            };

            // Disable the banner button to prevent double-clicks.
            revocation_banner.set_button_label(Some(&gettextrs::gettext("Authorizing…")));

            // Run OAuth flow on a background thread, poll from main loop.
            let browser_pref = crate::services::oauth_service::load_browser_preference();
            let (tx, rx) =
                std::sync::mpsc::channel::<crate::ui::edit_account_dialog::ReauthOAuthMessage>();
            std::thread::spawn(move || {
                crate::ui::edit_account_dialog::run_reauth_oauth_thread(
                    oauth_config,
                    tx,
                    browser_pref,
                );
            });

            let store = store.clone();
            let accounts = accounts.clone();
            let credential_store = credential_store.clone();
            let settings = settings.clone();
            let custom_order = custom_order.clone();
            let conn_state_mgr = conn_state_mgr.clone();
            let banner = revocation_banner.clone();
            let toast_ref = toast_overlay.clone();
            let account_list = account_list.clone();
            let banner_id = banner_account_id.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                use crate::ui::edit_account_dialog::ReauthOAuthMessage;
                match rx.try_recv() {
                    Ok(ReauthOAuthMessage::Success {
                        access_token,
                        refresh_token,
                        expires_in,
                    }) => {
                        // Store new tokens (preserving all account state).
                        let validated = ValidatedTokenResponse {
                            access_token: access_token.clone(),
                            refresh_token,
                            expires_in,
                        };
                        if let Err(e) = crate::services::oauth_service::store_oauth_tokens(
                            &*credential_store,
                            account_id,
                            &validated,
                        ) {
                            eprintln!("Failed to store re-auth tokens: {e}");
                            let toast = adw::Toast::new(&format!(
                                "{} {e}",
                                gettextrs::gettext("Re-authorization failed:")
                            ));
                            toast_ref.add_toast(toast);
                            banner.set_button_label(Some(&gettextrs::gettext("Re-authorize")));
                            return glib::ControlFlow::Break;
                        }
                        // Update in-memory account credential and re-enable sync.
                        {
                            let mut list = accounts.borrow_mut();
                            if let Some(acct) = list.iter_mut().find(|a| a.id() == account_id) {
                                acct.update_credentials(access_token, AuthMethod::OAuth2);
                                acct.set_sync_enabled(true);
                                let _ = store.update(acct.clone());
                            }
                        }
                        // Hide the banner and clear the tracked account.
                        banner.set_revealed(false);
                        *banner_id.borrow_mut() = None;
                        banner.set_button_label(Some(&gettextrs::gettext("Re-authorize")));
                        // Show success toast.
                        let toast = adw::Toast::new(&gettextrs::gettext(
                            "Account re-authorized successfully.",
                        ));
                        toast_ref.add_toast(toast);
                        rebuild_account_list(
                            &account_list,
                            &accounts.borrow(),
                            settings.borrow().category_display_enabled,
                            custom_order.borrow().as_deref(),
                            &conn_state_mgr.borrow(),
                        );
                        glib::ControlFlow::Break
                    }
                    Ok(ReauthOAuthMessage::Error(err)) => {
                        banner.set_button_label(Some(&gettextrs::gettext("Re-authorize")));
                        let toast = adw::Toast::new(&format!(
                            "{} {}",
                            gettextrs::gettext("Re-authorization failed:"),
                            err
                        ));
                        toast_ref.add_toast(toast);
                        glib::ControlFlow::Break
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        banner.set_button_label(Some(&gettextrs::gettext("Re-authorize")));
                        glib::ControlFlow::Break
                    }
                }
            });
        }
    ));

    // "Add account" button handler.
    // FR-2: the wizard is the default path when adding an account.
    add_btn.connect_clicked(clone!(
        #[weak]
        window,
        #[strong]
        store,
        #[strong]
        accounts,
        #[strong]
        settings,
        #[strong]
        custom_order,
        #[strong]
        order_store,
        #[strong]
        conn_state_mgr,
        #[strong]
        credential_store,
        #[weak]
        account_list,
        move |_| {
            let store = store.clone();
            let accounts = accounts.clone();
            let settings = settings.clone();
            let custom_order = custom_order.clone();
            let order_store = order_store.clone();
            let conn_state_mgr = conn_state_mgr.clone();
            let credential_store = credential_store.clone();
            let account_list = account_list.clone();
            setup_wizard::show(
                &window,
                clone!(
                    #[weak]
                    window,
                    move |result| {
                        if let Some(setup_wizard::WizardAction::ManualSetup(data)) = result {
                            let store = store.clone();
                            let accounts = accounts.clone();
                            let settings = settings.clone();
                            let custom_order = custom_order.clone();
                            let order_store = order_store.clone();
                            let conn_state_mgr = conn_state_mgr.clone();
                            let credential_store = credential_store.clone();
                            let account_list = account_list.clone();
                            let categories = collect_categories(&accounts.borrow());
                            let window_for_identity = window.clone();
                            add_account_dialog::show_with_prefill(
                                &window,
                                categories,
                                add_account_dialog::PrefillData {
                                    display_name: data.display_name,
                                    email: data.email,
                                    password: data.password,
                                },
                                move |result| {
                                    if let Some(save_result) = result {
                                        let account = save_result.account;
                                        store_account_credentials(&*credential_store, &account);
                                        if let Err(e) = store.add(account.clone()) {
                                            eprintln!("Failed to persist new account: {e}");
                                            return;
                                        }
                                        let new_id = account.id();
                                        conn_state_mgr.borrow_mut().ensure_account(new_id);
                                        let create_identity = save_result.create_smtp_identity;
                                        let username = account.username().to_string();
                                        let credential = account.credential().to_string();
                                        let display_name = account.display_name().to_string();
                                        {
                                            let mut list = accounts.borrow_mut();
                                            list.push(account);
                                        }
                                        {
                                            let mut order = custom_order.borrow_mut();
                                            if let Some(ref mut o) = *order {
                                                o.push(new_id);
                                                let _ = order_store.save(o);
                                            }
                                        }
                                        rebuild_account_list(
                                            &account_list,
                                            &accounts.borrow(),
                                            settings.borrow().category_display_enabled,
                                            custom_order.borrow().as_deref(),
                                            &conn_state_mgr.borrow(),
                                        );
                                        // FR-43: navigate to SMTP identity dialog if opted in.
                                        if create_identity {
                                            let entry = smtp_identity_dialog::InboundAccountEntry {
                                                id: new_id.to_string(),
                                                display_name: display_name.clone(),
                                                username: username.clone(),
                                                password: credential.clone(),
                                            };
                                            smtp_identity_dialog::show(
                                                &window_for_identity,
                                                vec![entry],
                                                None,
                                                None,
                                                |_| {},
                                            );
                                        }
                                    }
                                },
                            );
                        }
                        // Future slices will wire Check action to provider detection / account creation.
                    }
                ),
            );
        }
    ));

    // FR-47: export button handler — open export dialog.
    export_btn.connect_clicked(clone!(
        #[weak]
        window,
        #[strong]
        accounts,
        move |_| {
            let accts = accounts.borrow().clone();
            export_dialog::show(&window, accts, |_success| {});
        }
    ));

    // FR-49: import button handler — open import dialog.
    import_btn.connect_clicked(clone!(
        #[weak]
        window,
        #[strong]
        store,
        #[strong]
        accounts,
        #[strong]
        settings,
        #[strong]
        custom_order,
        #[strong]
        order_store,
        #[strong]
        conn_state_mgr,
        #[strong]
        credential_store,
        #[weak]
        account_list,
        move |_| {
            let store = store.clone();
            let accounts_rc = accounts.clone();
            let settings = settings.clone();
            let custom_order = custom_order.clone();
            let order_store = order_store.clone();
            let conn_state_mgr = conn_state_mgr.clone();
            let credential_store = credential_store.clone();
            let account_list = account_list.clone();
            let accts = accounts.borrow().clone();
            import_dialog::show(&window, accts, move |result| {
                if let Some(updated_accounts) = result {
                    // Determine which accounts are new (not in the old list).
                    let old_ids: Vec<uuid::Uuid> =
                        accounts_rc.borrow().iter().map(|a| a.id()).collect();

                    // Replace the accounts list with the updated one.
                    *accounts_rc.borrow_mut() = updated_accounts.clone();

                    // Persist all accounts.
                    for acct in &updated_accounts {
                        store_account_credentials(&*credential_store, acct);
                        if old_ids.contains(&acct.id()) {
                            let _ = store.update(acct.clone());
                        } else {
                            let _ = store.add(acct.clone());
                            conn_state_mgr.borrow_mut().ensure_account(acct.id());
                            // Add new account to custom order if one exists.
                            let mut order = custom_order.borrow_mut();
                            if let Some(ref mut o) = *order {
                                o.push(acct.id());
                                let _ = order_store.save(o);
                            }
                        }
                    }

                    rebuild_account_list(
                        &account_list,
                        &accounts_rc.borrow(),
                        settings.borrow().category_display_enabled,
                        custom_order.borrow().as_deref(),
                        &conn_state_mgr.borrow(),
                    );
                }
            });
        }
    ));

    // FR-26, FR-28: import custom provider configuration handler.
    import_provider_btn.connect_clicked(clone!(
        #[weak]
        window,
        move |_| {
            import_provider_dialog::show(&window);
        }
    ));

    // Row activation handler: open edit dialog for the selected account.
    account_list.connect_row_activated(clone!(
        #[weak]
        window,
        #[strong]
        store,
        #[strong]
        accounts,
        #[strong]
        settings,
        #[strong]
        custom_order,
        #[strong]
        order_store,
        #[strong]
        conn_state_mgr,
        #[strong]
        credential_store,
        #[weak]
        account_list,
        move |_, row| {
            let acct_id = row_account_id(row);
            let account = {
                let list = accounts.borrow();
                match acct_id.and_then(|id| list.iter().find(|a| a.id() == id).cloned()) {
                    Some(a) => a,
                    None => return,
                }
            };

            // Capture connection diagnostics for the edit dialog (FR-45, FR-46).
            let conn_state;
            let conn_error;
            let conn_log;
            {
                let mgr = conn_state_mgr.borrow();
                conn_state = mgr.state(account.id());
                conn_error = mgr.error_detail(account.id()).map(String::from);
                conn_log = mgr.log_entries(account.id()).to_vec();
            }

            let store = store.clone();
            let accounts = accounts.clone();
            let settings = settings.clone();
            let custom_order = custom_order.clone();
            let conn_state_mgr = conn_state_mgr.clone();
            let account_list = account_list.clone();
            let categories = collect_categories(&accounts.borrow());
            let conn_diag = edit_account_dialog::ConnectionDiagnostics {
                state: conn_state,
                error: conn_error,
                log: conn_log,
                main_window: window.clone(),
            };
            let order_store = order_store.clone();
            let credential_store = credential_store.clone();
            let custom_order = custom_order.clone();
            // FR-44: capture old credential for password-change detection.
            let old_credential = account.credential().to_string();
            let edit_window = window.clone();
            edit_account_dialog::show(&window, account, categories, conn_diag, move |result| {
                match result {
                    Some(edit_account_dialog::EditDialogResult::Updated(updated)) => {
                        let updated = *updated;
                        store_account_credentials(&*credential_store, &updated);
                        if let Err(e) = store.update(updated.clone()) {
                            eprintln!("Failed to persist account update: {e}");
                            return;
                        }
                        // FR-44: if password changed, offer to propagate to identities.
                        if core::password_has_changed(&old_credential, updated.credential()) {
                            let identity_ids: Vec<i64> = store
                                .load_identities_for_account(updated.id())
                                .unwrap_or_default()
                                .iter()
                                .map(|r| r.id)
                                .collect();
                            if !identity_ids.is_empty() {
                                let new_password = updated.credential().to_string();
                                let cred_store = credential_store.clone();
                                show_password_propagation_dialog(
                                    &edit_window,
                                    identity_ids,
                                    new_password,
                                    cred_store,
                                );
                            }
                        }
                        {
                            let mut list = accounts.borrow_mut();
                            if let Some(pos) = list.iter().position(|a| a.id() == updated.id()) {
                                list[pos] = updated;
                            }
                            // FR-32: revoke primary if sync was disabled.
                            if let Some(revoked_id) = core::revoke_if_sync_disabled(&mut list) {
                                if let Some(a) = list.iter().find(|a| a.id() == revoked_id) {
                                    let _ = store.update(a.clone());
                                }
                            }
                        }
                        rebuild_account_list(
                            &account_list,
                            &accounts.borrow(),
                            settings.borrow().category_display_enabled,
                            custom_order.borrow().as_deref(),
                            &conn_state_mgr.borrow(),
                        );
                    }
                    Some(edit_account_dialog::EditDialogResult::Deleted(deleted_id)) => {
                        // FR-29, FR-30, AC-9: delete account and all associated data.
                        // Clear primary designation if this was the primary account.
                        {
                            let mut list = accounts.borrow_mut();
                            clear_primary_if_deleted(&mut list, deleted_id);
                        }
                        // Remove from persistent store.
                        if let Err(e) = store.delete(deleted_id) {
                            eprintln!("Failed to delete account: {e}");
                            return;
                        }
                        // Remove all credentials from the system keychain.
                        if let Err(e) = credential_store.delete_all_for_account(deleted_id) {
                            eprintln!("Warning: could not remove credentials from keychain: {e}");
                        }
                        // Remove from in-memory list.
                        {
                            let mut list = accounts.borrow_mut();
                            list.retain(|a| a.id() != deleted_id);
                        }
                        // FR-41: remove notification channel.
                        {
                            use crate::services::notification_channel::{
                                MockNotificationChannelManager, NotificationChannelManager,
                            };
                            let mgr = MockNotificationChannelManager;
                            let _ = mgr.unregister_channel(deleted_id);
                        }
                        // Remove from custom order if present.
                        {
                            let mut order = custom_order.borrow_mut();
                            if let Some(ref mut o) = *order {
                                remove_from_order(o, deleted_id);
                                let _ = order_store.save(o);
                            }
                        }
                        // Remove connection state.
                        conn_state_mgr.borrow_mut().remove(deleted_id);
                        // Rebuild account list immediately.
                        rebuild_account_list(
                            &account_list,
                            &accounts.borrow(),
                            settings.borrow().category_display_enabled,
                            custom_order.borrow().as_deref(),
                            &conn_state_mgr.borrow(),
                        );
                    }
                    Some(edit_account_dialog::EditDialogResult::Duplicated(duplicated)) => {
                        // FR-31, AC-10: duplicate creates a new independent account.
                        let duplicated = *duplicated;
                        store_account_credentials(&*credential_store, &duplicated);
                        if let Err(e) = store.add(duplicated.clone()) {
                            eprintln!("Failed to persist duplicated account: {e}");
                            return;
                        }
                        let new_id = duplicated.id();
                        // Initialise connection state for the new account.
                        conn_state_mgr.borrow_mut().ensure_account(new_id);
                        {
                            let mut list = accounts.borrow_mut();
                            list.push(duplicated);
                        }
                        // Add new account to custom order if one exists.
                        {
                            let mut order = custom_order.borrow_mut();
                            if let Some(ref mut o) = *order {
                                o.push(new_id);
                                let _ = order_store.save(o);
                            }
                        }
                        rebuild_account_list(
                            &account_list,
                            &accounts.borrow(),
                            settings.borrow().category_display_enabled,
                            custom_order.borrow().as_deref(),
                            &conn_state_mgr.borrow(),
                        );
                    }
                    Some(edit_account_dialog::EditDialogResult::Reauthorized {
                        account_id,
                        access_token,
                        refresh_token,
                        expires_in,
                    }) => {
                        // FR-25, US-18, US-19: store new OAuth tokens and update
                        // the account credential, preserving all other state.
                        let validated = crate::core::ValidatedTokenResponse {
                            access_token: access_token.clone(),
                            refresh_token,
                            expires_in,
                        };
                        if let Err(e) = crate::services::oauth_service::store_oauth_tokens(
                            &*credential_store,
                            account_id,
                            &validated,
                        ) {
                            eprintln!("Failed to store re-auth tokens: {e}");
                            return;
                        }
                        // Update the in-memory account credential.
                        {
                            let mut list = accounts.borrow_mut();
                            if let Some(acct) = list.iter_mut().find(|a| a.id() == account_id) {
                                acct.update_credentials(
                                    access_token,
                                    crate::core::AuthMethod::OAuth2,
                                );
                                acct.set_sync_enabled(true);
                                let _ = store.update(acct.clone());
                            }
                        }
                        rebuild_account_list(
                            &account_list,
                            &accounts.borrow(),
                            settings.borrow().category_display_enabled,
                            custom_order.borrow().as_deref(),
                            &conn_state_mgr.borrow(),
                        );
                    }
                    Some(edit_account_dialog::EditDialogResult::ConvertedToPassword {
                        account_id,
                        new_password,
                    }) => {
                        // FR-30: Convert OAuth → password. Remove old OAuth tokens,
                        // store the new password for both IMAP and SMTP, and update
                        // the account auth method. All other account state (folders,
                        // identities, etc.) is preserved.
                        let _ =
                            credential_store.delete(account_id, CredentialRole::OAuthRefreshToken);
                        let _ =
                            credential_store.delete(account_id, CredentialRole::OAuthTokenExpiry);
                        if let Err(e) = credential_store.write(
                            account_id,
                            CredentialRole::ImapPassword,
                            &SecretValue::new(new_password.clone()),
                        ) {
                            eprintln!("Failed to store new password: {e}");
                            return;
                        }
                        // Store the same password for SMTP so outgoing mail works.
                        if let Err(e) = credential_store.write(
                            account_id,
                            CredentialRole::SmtpPassword,
                            &SecretValue::new(new_password.clone()),
                        ) {
                            eprintln!("Failed to store SMTP password: {e}");
                        }
                        {
                            let mut list = accounts.borrow_mut();
                            if let Some(acct) = list.iter_mut().find(|a| a.id() == account_id) {
                                acct.switch_auth_type(new_password, crate::core::AuthMethod::Plain);
                                let _ = store.update(acct.clone());
                            }
                        }
                        rebuild_account_list(
                            &account_list,
                            &accounts.borrow(),
                            settings.borrow().category_display_enabled,
                            custom_order.borrow().as_deref(),
                            &conn_state_mgr.borrow(),
                        );
                    }
                    Some(edit_account_dialog::EditDialogResult::ConvertedToOAuth {
                        account_id,
                        access_token,
                        refresh_token,
                        expires_in,
                    }) => {
                        // FR-30: Convert password → OAuth. Store new OAuth tokens,
                        // remove the old SMTP password (OAuth uses the access token
                        // for both IMAP and SMTP), and update the account auth method.
                        // All other account state (folders, identities, etc.) is preserved.
                        let validated = crate::core::ValidatedTokenResponse {
                            access_token: access_token.clone(),
                            refresh_token,
                            expires_in,
                        };
                        if let Err(e) = crate::services::oauth_service::store_oauth_tokens(
                            &*credential_store,
                            account_id,
                            &validated,
                        ) {
                            eprintln!("Failed to store OAuth tokens: {e}");
                            return;
                        }
                        // Remove old SMTP password — OAuth uses the access token.
                        let _ = credential_store.delete(account_id, CredentialRole::SmtpPassword);
                        {
                            let mut list = accounts.borrow_mut();
                            if let Some(acct) = list.iter_mut().find(|a| a.id() == account_id) {
                                acct.switch_auth_type(
                                    access_token,
                                    crate::core::AuthMethod::OAuth2,
                                );
                                let _ = store.update(acct.clone());
                            }
                        }
                        rebuild_account_list(
                            &account_list,
                            &accounts.borrow(),
                            settings.borrow().category_display_enabled,
                            custom_order.borrow().as_deref(),
                            &conn_state_mgr.borrow(),
                        );
                    }
                    None => {}
                }
            });
        }
    ));

    window.present();

    // FR-1: on first launch with zero accounts, present the setup wizard automatically.
    if accounts.borrow().is_empty() {
        let store = store.clone();
        let accounts_first = accounts.clone();
        let settings_first = settings.clone();
        let custom_order_first = custom_order.clone();
        let order_store_first = order_store.clone();
        let conn_state_mgr_first = conn_state_mgr.clone();
        let credential_store_first = credential_store.clone();
        let account_list_first = account_list.clone();
        setup_wizard::show(
            &window,
            clone!(
                #[weak]
                window,
                move |result| {
                    if let Some(setup_wizard::WizardAction::ManualSetup(data)) = result {
                        let store = store.clone();
                        let accounts = accounts_first.clone();
                        let settings = settings_first.clone();
                        let custom_order = custom_order_first.clone();
                        let order_store = order_store_first.clone();
                        let conn_state_mgr = conn_state_mgr_first.clone();
                        let credential_store = credential_store_first.clone();
                        let account_list = account_list_first.clone();
                        let categories = collect_categories(&accounts.borrow());
                        let window_for_identity = window.clone();
                        add_account_dialog::show_with_prefill(
                            &window,
                            categories,
                            add_account_dialog::PrefillData {
                                display_name: data.display_name,
                                email: data.email,
                                password: data.password,
                            },
                            move |result| {
                                if let Some(save_result) = result {
                                    let account = save_result.account;
                                    store_account_credentials(&*credential_store, &account);
                                    if let Err(e) = store.add(account.clone()) {
                                        eprintln!("Failed to persist new account: {e}");
                                        return;
                                    }
                                    let new_id = account.id();
                                    conn_state_mgr.borrow_mut().ensure_account(new_id);
                                    let create_identity = save_result.create_smtp_identity;
                                    let username = account.username().to_string();
                                    let credential = account.credential().to_string();
                                    let display_name = account.display_name().to_string();
                                    {
                                        let mut list = accounts.borrow_mut();
                                        list.push(account);
                                    }
                                    {
                                        let mut order = custom_order.borrow_mut();
                                        if let Some(ref mut o) = *order {
                                            o.push(new_id);
                                            let _ = order_store.save(o);
                                        }
                                    }
                                    rebuild_account_list(
                                        &account_list,
                                        &accounts.borrow(),
                                        settings.borrow().category_display_enabled,
                                        custom_order.borrow().as_deref(),
                                        &conn_state_mgr.borrow(),
                                    );
                                    // FR-43: navigate to SMTP identity dialog if opted in.
                                    if create_identity {
                                        let entry = smtp_identity_dialog::InboundAccountEntry {
                                            id: new_id.to_string(),
                                            display_name: display_name.clone(),
                                            username: username.clone(),
                                            password: credential.clone(),
                                        };
                                        smtp_identity_dialog::show(
                                            &window_for_identity,
                                            vec![entry],
                                            None,
                                            None,
                                            |_| {},
                                        );
                                    }
                                }
                            },
                        );
                    }
                    // Future slices will wire Check action to provider detection / account creation.
                }
            ),
        );
    }
}

/// Retrieve the account UUID stored on a row widget via its `widget_name`.
/// Returns `None` for non-account rows (e.g. category headers).
fn row_account_id(row: &gtk::ListBoxRow) -> Option<uuid::Uuid> {
    let name = row.widget_name();
    uuid::Uuid::parse_str(name.as_str()).ok()
}

/// Remove all rows from the list box and rebuild from the accounts slice.
/// When `category_display` is true, accounts are grouped under category headers (FR-18).
/// When false, accounts are sorted using custom order if available, otherwise
/// default: primary first, then alphabetically (FR-19, FR-20).
fn rebuild_account_list(
    list_box: &gtk::ListBox,
    accounts: &[Account],
    category_display: bool,
    custom_order: Option<&[Uuid]>,
    conn_mgr: &ConnectionStateManager,
) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    if category_display {
        let groups = group_by_category(accounts);
        for group in &groups {
            // Category header row (non-activatable, non-selectable).
            let header_label = match &group.category {
                Some(name) => name.clone(),
                None => gettextrs::gettext("Uncategorized"),
            };
            let header_row = gtk::ListBoxRow::builder()
                .activatable(false)
                .selectable(false)
                .child(
                    &gtk::Label::builder()
                        .label(&header_label)
                        .css_classes(["heading"])
                        .halign(gtk::Align::Start)
                        .margin_top(8)
                        .margin_bottom(4)
                        .margin_start(12)
                        .build(),
                )
                .build();
            list_box.append(&header_row);

            for &idx in &group.accounts {
                let state = conn_mgr.state(accounts[idx].id());
                let row = make_account_row(&accounts[idx], state);
                list_box.append(&row);
            }
        }
    } else {
        let sorted = match custom_order {
            Some(order) => apply_custom_order(accounts, order),
            None => crate::core::sort_accounts_flat(accounts),
        };
        for idx in sorted {
            let state = conn_mgr.state(accounts[idx].id());
            let row = make_account_row(&accounts[idx], state);
            list_box.append(&row);
        }
    }
}

fn make_account_row(account: &Account, conn_state: ConnectionState) -> adw::ActionRow {
    use crate::core::Protocol;

    let subtitle = format!(
        "{} – {}:{} – {}",
        account.protocol(),
        account.host(),
        account.port(),
        conn_state
    );

    let row = adw::ActionRow::builder()
        .title(account.display_name())
        .subtitle(&subtitle)
        .activatable(true)
        .name(account.id().to_string())
        .build();

    // FR-44, AC-18: connection state indicator icon.
    let state_icon = gtk::Image::builder()
        .icon_name(conn_state.icon_name())
        .pixel_size(16)
        .valign(gtk::Align::Center)
        .tooltip_text(conn_state.to_string())
        .css_classes([conn_state.css_class()])
        .build();
    row.add_prefix(&state_icon);

    // FR-13, AC-6: display account avatar alongside the account name.
    if let Some(path) = account.avatar_path() {
        let avatar = gtk::Image::builder()
            .pixel_size(32)
            .valign(gtk::Align::Center)
            .build();
        avatar.set_from_file(Some(path));
        row.add_prefix(&avatar);
    }

    // FR-14: account colour stripe as a leading prefix indicator.
    if let Some(color) = account.color() {
        let hex = color.to_hex();
        let css_class = format!("account-color-{}", &hex[1..]);
        let stripe = gtk::Box::builder()
            .width_request(4)
            .height_request(32)
            .valign(gtk::Align::Center)
            .css_classes([css_class.as_str()])
            .build();
        let css = format!(".{css_class} {{ background-color: {hex}; border-radius: 2px; }}");
        let provider = gtk::CssProvider::new();
        provider.load_from_data(&css);
        gtk::style_context_add_provider_for_display(
            &stripe.display(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        row.add_prefix(&stripe);
    }

    // FR-6, US-27: visually indicate on-demand accounts.
    if account.on_demand() {
        let on_demand_icon = gtk::Image::builder()
            .icon_name("emblem-synchronizing-symbolic")
            .pixel_size(16)
            .valign(gtk::Align::Center)
            .tooltip_text(gettextrs::gettext("On-demand sync"))
            .build();
        row.add_suffix(&on_demand_icon);
    }

    // FR-39: visually indicate notification-enabled accounts.
    if account.notifications_enabled() {
        let notif_icon = gtk::Image::builder()
            .icon_name("preferences-system-notifications-symbolic")
            .pixel_size(16)
            .valign(gtk::Align::Center)
            .tooltip_text(gettextrs::gettext("Notifications enabled"))
            .build();
        row.add_suffix(&notif_icon);
    }

    // FR-6: visually indicate sync-disabled accounts with a paused icon.
    if !account.sync_enabled() {
        let paused = gtk::Image::builder()
            .icon_name("media-playback-pause-symbolic")
            .pixel_size(16)
            .valign(gtk::Align::Center)
            .tooltip_text(gettextrs::gettext("Synchronization disabled"))
            .build();
        row.add_suffix(&paused);
    }

    // FR-7, US-29: indicate unmetered-only constraint.
    if account.unmetered_only() {
        let icon = gtk::Image::builder()
            .icon_name("network-cellular-signal-excellent-symbolic")
            .pixel_size(16)
            .valign(gtk::Align::Center)
            .tooltip_text(gettextrs::gettext("Unmetered network only"))
            .build();
        row.add_suffix(&icon);
    }

    // FR-7, US-29, AC-13: indicate VPN-only constraint.
    if account.vpn_only() {
        let icon = gtk::Image::builder()
            .icon_name("network-vpn-symbolic")
            .pixel_size(16)
            .valign(gtk::Align::Center)
            .tooltip_text(gettextrs::gettext("VPN only"))
            .build();
        row.add_suffix(&icon);
    }

    // FR-7, US-30: indicate schedule exemption.
    if account.schedule_exempt() {
        let icon = gtk::Image::builder()
            .icon_name("alarm-symbolic")
            .pixel_size(16)
            .valign(gtk::Align::Center)
            .tooltip_text(gettextrs::gettext("Schedule exempt"))
            .build();
        row.add_suffix(&icon);
    }

    // FR-27: visually indicate the primary account with a star icon.
    if account.is_primary() {
        let star = gtk::Image::builder()
            .icon_name("starred-symbolic")
            .pixel_size(16)
            .valign(gtk::Align::Center)
            .tooltip_text(gettextrs::gettext("Primary account"))
            .build();
        row.add_suffix(&star);
    }

    // FR-11: visually distinguish POP3 accounts with a suffix badge.
    if account.protocol() == Protocol::Pop3 {
        let badge = gtk::Label::builder()
            .label("POP3")
            .css_classes(["caption", "accent"])
            .valign(gtk::Align::Center)
            .build();
        row.add_suffix(&badge);
    }

    row
}

/// FR-44: Show a dialog asking the user whether to propagate the new inbound
/// password to all associated SMTP identities.
fn show_password_propagation_dialog(
    parent: &adw::ApplicationWindow,
    identity_ids: Vec<i64>,
    new_password: String,
    cred_store: Rc<dyn crate::core::CredentialStore>,
) {
    let dialog = adw::AlertDialog::builder()
        .heading(gettextrs::gettext("Update identity passwords?"))
        .body(gettextrs::gettext(
            "The account password has changed. Do you want to update the SMTP password on all associated identities to match?",
        ))
        .build();

    dialog.add_response("decline", &gettextrs::gettext("No"));
    dialog.add_response("accept", &gettextrs::gettext("Yes"));
    dialog.set_default_response(Some("accept"));
    dialog.set_close_response("decline");

    dialog.connect_response(None, move |_dialog, response| {
        if response == "accept" {
            core::propagate_password_to_identities(&*cred_store, &identity_ids, &new_password);
        }
    });

    dialog.present(Some(parent));
}

#[allow(dead_code)]
/// US-5, AC-1, AC-2: Show a revocation notification on the given banner.
///
/// Displays the banner with the notification title and stores the account ID
/// so the banner's "Re-authorize" button handler knows which account to act on.
/// This function is called when a token refresh returns `NeedsReauthorization`.
pub(crate) fn show_revocation_notification(
    banner: &adw::Banner,
    banner_account_id: &Rc<RefCell<Option<Uuid>>>,
    event: &RevocationEvent,
) {
    let notification = build_revocation_notification(event);
    banner.set_title(&notification.title);
    banner.set_revealed(true);
    *banner_account_id.borrow_mut() = Some(event.account_id);
}
