use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::{self, collect_categories, group_by_category, sort_accounts_flat, Account};
use crate::services::{AccountStore, AppSettings, SettingsStore};
use crate::ui::add_account_dialog;
use crate::ui::edit_account_dialog;

/// Build the main application window with the account list and navigation pane.
pub(crate) fn build(
    app: &adw::Application,
    store: Rc<AccountStore>,
    settings_store: Rc<SettingsStore>,
) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(gettextrs::gettext("Alarm Clock – Accounts"))
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

    // FR-21: toggle button for category grouping in the navigation pane.
    let category_toggle = gtk::ToggleButton::builder()
        .icon_name("view-list-symbolic")
        .tooltip_text(gettextrs::gettext("Group by category"))
        .accessible_role(gtk::AccessibleRole::ToggleButton)
        .build();
    sidebar_header.pack_end(&category_toggle);

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

    window.set_content(Some(&split_view));

    // Shared mutable list of accounts for the sidebar model.
    let accounts: Rc<RefCell<Vec<Account>>> = Rc::new(RefCell::new(Vec::new()));

    // Load settings and initialise toggle state.
    let settings: Rc<RefCell<AppSettings>> =
        Rc::new(RefCell::new(settings_store.load().unwrap_or_default()));
    category_toggle.set_active(settings.borrow().category_display_enabled);

    // Populate from store on startup.
    {
        let loaded = store.load_all().unwrap_or_default();
        *accounts.borrow_mut() = loaded;
        rebuild_account_list(
            &account_list,
            &accounts.borrow(),
            settings.borrow().category_display_enabled,
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
        #[weak]
        account_list,
        move |btn| {
            let enabled = btn.is_active();
            {
                let mut s = settings.borrow_mut();
                s.category_display_enabled = enabled;
                let _ = settings_store.save(&s);
            }
            rebuild_account_list(&account_list, &accounts.borrow(), enabled);
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
                    );
                }
            }
            let _ = (widget, x);
        }
    ));
    account_list.add_controller(gesture);

    // "Add account" button handler.
    add_btn.connect_clicked(clone!(
        #[weak]
        window,
        #[strong]
        store,
        #[strong]
        accounts,
        #[strong]
        settings,
        #[weak]
        account_list,
        move |_| {
            let store = store.clone();
            let accounts = accounts.clone();
            let settings = settings.clone();
            let account_list = account_list.clone();
            let categories = collect_categories(&accounts.borrow());
            add_account_dialog::show(&window, categories, move |result| {
                if let Some(account) = result {
                    if let Err(e) = store.add(account.clone()) {
                        eprintln!("Failed to persist account: {e}");
                        return;
                    }
                    let new_id = account.id();
                    let became_primary;
                    {
                        let mut list = accounts.borrow_mut();
                        list.push(account);
                        // FR-28: auto-designate primary if none exists.
                        became_primary = core::auto_designate_on_add(&mut list, new_id);
                    }
                    // Persist if auto-designated primary.
                    if became_primary {
                        let list = accounts.borrow();
                        if let Some(a) = list.iter().find(|a| a.id() == new_id) {
                            let _ = store.update(a.clone());
                        }
                    }
                    rebuild_account_list(
                        &account_list,
                        &accounts.borrow(),
                        settings.borrow().category_display_enabled,
                    );
                }
            });
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
            let store = store.clone();
            let accounts = accounts.clone();
            let settings = settings.clone();
            let account_list = account_list.clone();
            let categories = collect_categories(&accounts.borrow());
            edit_account_dialog::show(&window, account, categories, move |result| {
                if let Some(updated) = result {
                    if let Err(e) = store.update(updated.clone()) {
                        eprintln!("Failed to persist account update: {e}");
                        return;
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
                    );
                }
            });
        }
    ));

    window.present();
}

/// Retrieve the account UUID stored on a row widget via its `widget_name`.
/// Returns `None` for non-account rows (e.g. category headers).
fn row_account_id(row: &gtk::ListBoxRow) -> Option<uuid::Uuid> {
    let name = row.widget_name();
    uuid::Uuid::parse_str(name.as_str()).ok()
}

/// Remove all rows from the list box and rebuild from the accounts slice.
/// When `category_display` is true, accounts are grouped under category headers (FR-18).
/// When false, accounts are sorted flat: primary first, then alphabetically (FR-19).
fn rebuild_account_list(list_box: &gtk::ListBox, accounts: &[Account], category_display: bool) {
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
                let row = make_account_row(&accounts[idx]);
                list_box.append(&row);
            }
        }
    } else {
        let sorted = sort_accounts_flat(accounts);
        for idx in sorted {
            let row = make_account_row(&accounts[idx]);
            list_box.append(&row);
        }
    }
}

fn make_account_row(account: &Account) -> adw::ActionRow {
    use crate::core::Protocol;

    let subtitle = format!(
        "{} – {}:{}",
        account.protocol(),
        account.host(),
        account.port()
    );

    let row = adw::ActionRow::builder()
        .title(account.display_name())
        .subtitle(&subtitle)
        .activatable(true)
        .name(account.id().to_string())
        .build();

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
