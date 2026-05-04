use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use uuid::Uuid;

use crate::core::{
    self, apply_custom_order, collect_categories, group_by_category, move_account, Account,
};
use crate::services::{AccountStore, AppSettings, OrderStore, SettingsStore};
use crate::ui::add_account_dialog;
use crate::ui::edit_account_dialog;

/// Build the main application window with the account list and navigation pane.
pub(crate) fn build(
    app: &adw::Application,
    store: Rc<AccountStore>,
    settings_store: Rc<SettingsStore>,
    order_store: Rc<OrderStore>,
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

    window.set_content(Some(&split_view));

    // Shared mutable list of accounts for the sidebar model.
    let accounts: Rc<RefCell<Vec<Account>>> = Rc::new(RefCell::new(Vec::new()));

    // Shared mutable custom order (FR-20, AC-8).
    let custom_order: Rc<RefCell<Option<Vec<Uuid>>>> = Rc::new(RefCell::new(None));

    // Load settings and initialise toggle state.
    let settings: Rc<RefCell<AppSettings>> =
        Rc::new(RefCell::new(settings_store.load().unwrap_or_default()));
    category_toggle.set_active(settings.borrow().category_display_enabled);

    // Populate from store on startup.
    {
        let loaded = store.load_all().unwrap_or_default();
        *accounts.borrow_mut() = loaded;
        // Load persisted order (AC-8).
        *custom_order.borrow_mut() = order_store.load().unwrap_or(None);
        rebuild_account_list(
            &account_list,
            &accounts.borrow(),
            settings.borrow().category_display_enabled,
            custom_order.borrow().as_deref(),
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
            );

            // Re-select the moved row.
            if let Some(new_row) = account_list.row_at_index(to as i32) {
                account_list.select_row(Some(&new_row));
            }

            glib::Propagation::Stop
        }
    ));
    account_list.add_controller(key_controller);

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
        #[strong]
        custom_order,
        #[strong]
        order_store,
        #[weak]
        account_list,
        move |_| {
            let store = store.clone();
            let accounts = accounts.clone();
            let settings = settings.clone();
            let custom_order = custom_order.clone();
            let order_store = order_store.clone();
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
        #[strong]
        custom_order,
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
            let custom_order = custom_order.clone();
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
                        custom_order.borrow().as_deref(),
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
/// When false, accounts are sorted using custom order if available, otherwise
/// default: primary first, then alphabetically (FR-19, FR-20).
fn rebuild_account_list(
    list_box: &gtk::ListBox,
    accounts: &[Account],
    category_display: bool,
    custom_order: Option<&[Uuid]>,
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
                let row = make_account_row(&accounts[idx]);
                list_box.append(&row);
            }
        }
    } else {
        let sorted = match custom_order {
            Some(order) => apply_custom_order(accounts, order),
            None => crate::core::sort_accounts_flat(accounts),
        };
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
