use std::cell::RefCell;
use std::rc::Rc;

use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::core::connection_test::{ConnectionTestRequest, ServerConnectionParams};
use crate::core::{
    Account, AccountColor, AuthMethod, ConnectionLogEntry, ConnectionState, EncryptionMode,
    Pop3Settings, Protocol, QuotaInfo, SmtpConfig, SwipeAction, SwipeDefaults, SystemFolders,
    UpdateAccountParams,
};
use crate::services::connection_tester::{ConnectionTester, MockConnectionTester};
use crate::ui::connection_log_dialog;

/// Result of the edit-account dialog.
#[derive(Debug, Clone)]
pub(crate) enum EditDialogResult {
    /// Account was updated (save).
    Updated(Box<Account>),
    /// Account should be deleted (confirmed by user).
    Deleted(uuid::Uuid),
    /// Account should be duplicated (FR-31, AC-10).
    Duplicated(Box<Account>),
}

/// Connection diagnostics passed to the edit dialog (FR-44, FR-45, FR-46).
pub(crate) struct ConnectionDiagnostics {
    pub state: ConnectionState,
    pub error: Option<String>,
    pub log: Vec<ConnectionLogEntry>,
    pub main_window: adw::ApplicationWindow,
}

/// Build and show the "Edit Account" dialog pre-populated with the given account's values.
/// Calls `on_done` with the updated account on save, or `None` on cancel/close.
/// `existing_categories` provides autocomplete suggestions for the category field (FR-23).
/// Connection diagnostics are displayed read-only to help the user diagnose
/// connectivity problems (FR-44, FR-45, FR-46).
pub(crate) fn show(
    parent: &adw::ApplicationWindow,
    account: Account,
    existing_categories: Vec<String>,
    conn_diag: ConnectionDiagnostics,
    on_done: impl Fn(Option<EditDialogResult>) + 'static,
) {
    let conn_state = conn_diag.state;
    let conn_error = conn_diag.error;
    let conn_log = conn_diag.log;
    let log_main_window = conn_diag.main_window;
    let dialog = adw::Dialog::builder()
        .title(gettextrs::gettext("Edit Account"))
        .content_width(460)
        .content_height(680)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let toast_overlay = adw::ToastOverlay::new();

    let scrolled = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();

    let clamp = adw::Clamp::builder()
        .maximum_size(500)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);

    // -- Display name --
    let name_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Display Name"))
        .build();
    let name_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Account name"))
        .build();
    name_row.set_tooltip_text(Some(&gettextrs::gettext(
        "A friendly name for this account",
    )));
    name_row.set_text(account.display_name());
    name_group.add(&name_row);
    vbox.append(&name_group);

    // -- Quota display (FR-42, FR-43, AC-17) --
    // Only shown when the server reports quota information.
    if let Some(quota) = account.quota() {
        let quota_group = adw::PreferencesGroup::builder()
            .title(gettextrs::gettext("Storage Quota"))
            .build();

        let used_str = QuotaInfo::format_bytes(quota.used_bytes);
        let limit_str = QuotaInfo::format_bytes(quota.limit_bytes);
        let pct = quota.usage_percent();

        let quota_row = adw::ActionRow::builder()
            .title(gettextrs::gettext("Usage"))
            .subtitle(format!("{used_str} / {limit_str} ({pct:.1}%)"))
            .build();

        let level_bar = gtk::LevelBar::builder()
            .min_value(0.0)
            .max_value(100.0)
            .value(pct.min(100.0))
            .valign(gtk::Align::Center)
            .width_request(120)
            .build();

        if quota.is_high_usage() {
            level_bar.add_css_class("quota-warning");
            quota_row.add_css_class("warning");

            let warning_icon = gtk::Image::builder()
                .icon_name("dialog-warning-symbolic")
                .valign(gtk::Align::Center)
                .tooltip_text(gettextrs::gettext("Storage quota is critically high"))
                .build();
            quota_row.add_suffix(&warning_icon);
        }

        quota_row.add_suffix(&level_bar);
        quota_group.add(&quota_row);
        vbox.append(&quota_group);
    }

    // -- Connection state and diagnostics (FR-44, FR-45, FR-46, AC-18) --
    {
        let conn_group = adw::PreferencesGroup::builder()
            .title(gettextrs::gettext("Connection"))
            .build();

        let state_row = adw::ActionRow::builder()
            .title(gettextrs::gettext("Status"))
            .subtitle(conn_state.to_string())
            .build();
        let state_icon = gtk::Image::builder()
            .icon_name(conn_state.icon_name())
            .pixel_size(16)
            .valign(gtk::Align::Center)
            .css_classes([conn_state.css_class()])
            .build();
        state_row.add_prefix(&state_icon);
        conn_group.add(&state_row);

        // FR-45: display error detail when account is in an error state.
        if let Some(ref error) = conn_error {
            let error_row = adw::ActionRow::builder()
                .title(gettextrs::gettext("Error"))
                .subtitle(error)
                .css_classes(["error"])
                .build();
            let error_icon = gtk::Image::builder()
                .icon_name("dialog-warning-symbolic")
                .pixel_size(16)
                .valign(gtk::Align::Center)
                .build();
            error_row.add_prefix(&error_icon);
            conn_group.add(&error_row);
        }

        // FR-46, US-42: button to view the full connection log.
        let log_btn = gtk::Button::builder()
            .label(gettextrs::gettext("View Connection Log"))
            .css_classes(["flat"])
            .build();
        let account_name = account.display_name().to_string();
        let log_conn_state = conn_state;
        let log_conn_error = conn_error.clone();
        let log_entries = conn_log;
        let log_window = log_main_window;
        log_btn.connect_clicked(move |_| {
            connection_log_dialog::show(
                &log_window,
                &account_name,
                log_conn_state,
                log_conn_error.as_deref(),
                &log_entries,
            );
        });
        conn_group.add(&adw::ActionRow::builder().child(&log_btn).build());

        vbox.append(&conn_group);
    }

    // -- Notifications toggle (FR-39, AC-19) --
    let notif_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Notifications"))
        .build();
    let notif_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Enable notifications"))
        .subtitle(gettextrs::gettext(
            "Receive alerts for new messages on this account",
        ))
        .active(account.notifications_enabled())
        .build();
    notif_group.add(&notif_row);
    vbox.append(&notif_group);

    // -- Synchronization toggle (FR-6, AC-11) --
    let sync_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Synchronization"))
        .build();
    let sync_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Enable synchronization"))
        .active(account.sync_enabled())
        .build();
    sync_group.add(&sync_row);

    // -- On-demand sync toggle (FR-6, US-27, AC-12) --
    let on_demand_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("On-demand sync only"))
        .subtitle(gettextrs::gettext(
            "Sync only when you explicitly request it",
        ))
        .active(account.on_demand())
        .build();
    sync_group.add(&on_demand_row);

    // -- Polling interval (FR-6, US-28) --
    let poll_default = account.polling_interval_minutes().map_or(0.0, f64::from);
    let polling_row = adw::SpinRow::builder()
        .title(gettextrs::gettext(
            "Polling interval (minutes, 0 = default)",
        ))
        .adjustment(&gtk::Adjustment::new(
            poll_default,
            0.0,
            1440.0,
            1.0,
            5.0,
            0.0,
        ))
        .build();
    sync_group.add(&polling_row);

    // -- Unmetered network only toggle (FR-7, US-29) --
    let unmetered_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Unmetered network only"))
        .subtitle(gettextrs::gettext("Suppress sync on metered connections"))
        .active(account.unmetered_only())
        .build();
    sync_group.add(&unmetered_row);

    // -- VPN only toggle (FR-7, US-29, AC-13) --
    let vpn_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("VPN only"))
        .subtitle(gettextrs::gettext("Suppress sync when no VPN is active"))
        .active(account.vpn_only())
        .build();
    sync_group.add(&vpn_row);

    // -- Schedule exemption toggle (FR-7, US-30) --
    let schedule_exempt_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Schedule exemption"))
        .subtitle(gettextrs::gettext("Continue syncing during off-hours"))
        .active(account.schedule_exempt())
        .build();
    sync_group.add(&schedule_exempt_row);

    vbox.append(&sync_group);

    // -- Account colour (FR-5, FR-12) --
    let color_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Account Colour"))
        .build();

    let color_row = adw::ActionRow::builder()
        .title(gettextrs::gettext("Colour"))
        .build();

    let color_dialog = gtk::ColorDialog::builder()
        .title(gettextrs::gettext("Choose account colour"))
        .with_alpha(false)
        .build();
    let color_btn = gtk::ColorDialogButton::builder()
        .dialog(&color_dialog)
        .valign(gtk::Align::Center)
        .tooltip_text(gettextrs::gettext("Pick a colour for this account"))
        .build();
    color_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Account colour picker",
    ))]);

    let has_color = account.color().is_some();
    let color_active = std::rc::Rc::new(std::cell::RefCell::new(has_color));
    if let Some(c) = account.color() {
        let rgba = gtk4::gdk::RGBA::new(c.red, c.green, c.blue, 1.0);
        color_btn.set_rgba(&rgba);
    }

    let clear_color_btn = gtk::Button::builder()
        .icon_name("edit-clear-symbolic")
        .tooltip_text(gettextrs::gettext("Clear account colour"))
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .sensitive(has_color)
        .build();
    clear_color_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Clear account colour",
    ))]);

    color_btn.connect_rgba_notify(clone!(
        #[strong]
        color_active,
        #[weak]
        clear_color_btn,
        move |_| {
            *color_active.borrow_mut() = true;
            clear_color_btn.set_sensitive(true);
        }
    ));

    clear_color_btn.connect_clicked(clone!(
        #[strong]
        color_active,
        #[weak]
        color_btn,
        move |btn| {
            *color_active.borrow_mut() = false;
            let transparent = gtk4::gdk::RGBA::new(0.5, 0.5, 0.5, 1.0);
            color_btn.set_rgba(&transparent);
            btn.set_sensitive(false);
        }
    ));

    color_row.add_suffix(&color_btn);
    color_row.add_suffix(&clear_color_btn);
    color_group.add(&color_row);
    vbox.append(&color_group);

    // -- Account avatar (FR-5, FR-13, US-15, US-16) --
    let avatar_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Account Avatar"))
        .build();

    let avatar_row = adw::ActionRow::builder()
        .title(gettextrs::gettext("Avatar"))
        .build();

    let avatar_image = gtk::Image::builder()
        .pixel_size(32)
        .valign(gtk::Align::Center)
        .build();
    avatar_image.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Account avatar preview",
    ))]);

    let has_avatar = account.avatar_path().is_some();
    let avatar_path: Rc<RefCell<Option<String>>> =
        Rc::new(RefCell::new(account.avatar_path().map(String::from)));
    if let Some(path) = account.avatar_path() {
        avatar_image.set_from_file(Some(path));
    } else {
        avatar_image.set_icon_name(Some("avatar-default-symbolic"));
    }

    let choose_avatar_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Choose Image…"))
        .valign(gtk::Align::Center)
        .tooltip_text(gettextrs::gettext("Pick an avatar image for this account"))
        .build();
    choose_avatar_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Choose avatar image",
    ))]);

    let clear_avatar_btn = gtk::Button::builder()
        .icon_name("edit-clear-symbolic")
        .tooltip_text(gettextrs::gettext("Clear account avatar"))
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .sensitive(has_avatar)
        .build();
    clear_avatar_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Clear account avatar",
    ))]);

    choose_avatar_btn.connect_clicked(clone!(
        #[strong]
        avatar_path,
        #[weak]
        avatar_image,
        #[weak]
        clear_avatar_btn,
        move |_| {
            let file_dialog = gtk::FileDialog::builder()
                .title(gettextrs::gettext("Choose Avatar Image"))
                .modal(true)
                .build();
            let filter = gtk::FileFilter::new();
            filter.set_name(Some(&gettextrs::gettext("Image files")));
            filter.add_mime_type("image/png");
            filter.add_mime_type("image/jpeg");
            filter.add_mime_type("image/svg+xml");
            filter.add_mime_type("image/webp");
            let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            file_dialog.set_filters(Some(&filters));

            let avatar_path = avatar_path.clone();
            let avatar_image = avatar_image.clone();
            let clear_avatar_btn = clear_avatar_btn.clone();
            file_dialog.open(
                None::<&gtk::Window>,
                gtk::gio::Cancellable::NONE,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            avatar_image.set_from_file(Some(&path_str));
                            *avatar_path.borrow_mut() = Some(path_str);
                            clear_avatar_btn.set_sensitive(true);
                        }
                    }
                },
            );
        }
    ));

    clear_avatar_btn.connect_clicked(clone!(
        #[strong]
        avatar_path,
        #[weak]
        avatar_image,
        move |btn| {
            *avatar_path.borrow_mut() = None;
            avatar_image.set_icon_name(Some("avatar-default-symbolic"));
            btn.set_sensitive(false);
        }
    ));

    avatar_row.add_prefix(&avatar_image);
    avatar_row.add_suffix(&choose_avatar_btn);
    avatar_row.add_suffix(&clear_avatar_btn);
    avatar_group.add(&avatar_row);
    vbox.append(&avatar_group);

    // -- Category label (FR-17, FR-22, FR-23, US-17, US-18) --
    let category_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Category"))
        .build();
    let category_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Category label"))
        .build();
    category_row.set_tooltip_text(Some(&gettextrs::gettext(
        "Optional label to organize accounts (e.g. Work, Personal)",
    )));
    if let Some(cat) = account.category() {
        category_row.set_text(cat);
    }
    category_group.add(&category_row);

    // Autocomplete suggestions from existing categories (FR-23).
    if !existing_categories.is_empty() {
        let suggestions_box = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();
        for cat in &existing_categories {
            let suggestion = adw::ActionRow::builder()
                .title(cat)
                .activatable(true)
                .build();
            suggestions_box.append(&suggestion);
        }
        suggestions_box.set_visible(false);

        category_row.connect_changed(clone!(
            #[weak]
            suggestions_box,
            #[strong]
            existing_categories,
            move |row| {
                let text = row.text().to_string();
                let mut any_visible = false;
                for (idx, cat) in existing_categories.iter().enumerate() {
                    if let Some(child) = suggestions_box.row_at_index(idx as i32) {
                        let visible = text.is_empty() || cat.contains(&text);
                        child.set_visible(visible);
                        if visible {
                            any_visible = true;
                        }
                    }
                }
                suggestions_box.set_visible(any_visible);
            }
        ));

        suggestions_box.connect_row_activated(clone!(
            #[weak]
            category_row,
            #[weak]
            suggestions_box,
            move |_, row| {
                if let Some(action_row) = row.downcast_ref::<adw::ActionRow>() {
                    category_row.set_text(&action_row.title());
                    suggestions_box.set_visible(false);
                }
            }
        ));

        category_group.add(&suggestions_box);
    }

    vbox.append(&category_group);

    // -- Incoming server settings --
    let server_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Incoming Server"))
        .build();

    let protocol_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Protocol"))
        .model(&gtk::StringList::new(&[
            &gettextrs::gettext("IMAP"),
            &gettextrs::gettext("POP3"),
        ]))
        .selected(protocol_to_combo(account.protocol()))
        .build();
    server_group.add(&protocol_row);

    let host_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Host"))
        .build();
    host_row.set_text(account.host());
    server_group.add(&host_row);

    let port_row = adw::SpinRow::builder()
        .title(gettextrs::gettext("Port"))
        .adjustment(&gtk::Adjustment::new(
            f64::from(account.port()),
            1.0,
            65535.0,
            1.0,
            10.0,
            0.0,
        ))
        .build();
    server_group.add(&port_row);

    let encryption_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Encryption"))
        .model(&gtk::StringList::new(&[
            &gettextrs::gettext("SSL/TLS"),
            &gettextrs::gettext("STARTTLS"),
            &gettextrs::gettext("None"),
        ]))
        .selected(encryption_to_combo(account.encryption()))
        .build();
    server_group.add(&encryption_row);

    // -- POP3 limitations banner (US-35, FR-10) --
    let pop3_banner = adw::Banner::builder()
        .title(gettextrs::gettext(
            "POP3 limitations: no server-side folders, no server-side search, no remote flag sync. Sent, Drafts, and Trash are local-only.",
        ))
        .revealed(account.protocol() == Protocol::Pop3)
        .build();
    vbox.append(&pop3_banner);

    // Show/hide the banner when the protocol selection changes.
    protocol_row.connect_selected_notify(clone!(
        #[weak]
        pop3_banner,
        move |row| {
            pop3_banner.set_revealed(row.selected() == 1);
        }
    ));

    vbox.append(&server_group);

    // -- Authentication --
    let auth_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Authentication"))
        .build();

    let auth_method_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Method"))
        .model(&gtk::StringList::new(&[
            &gettextrs::gettext("PLAIN"),
            &gettextrs::gettext("LOGIN"),
            &gettextrs::gettext("OAuth2"),
        ]))
        .selected(auth_to_combo(account.auth_method()))
        .build();
    auth_group.add(&auth_method_row);

    let username_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Username"))
        .build();
    username_row.set_text(account.username());
    auth_group.add(&username_row);

    let password_row = adw::PasswordEntryRow::builder()
        .title(gettextrs::gettext("Password / Token"))
        .build();
    password_row.set_text(account.credential());
    auth_group.add(&password_row);

    vbox.append(&auth_group);

    // -- POP3-specific settings (US-31, US-32, US-33, US-34, FR-9) --
    let existing_pop3 = account.pop3_settings();
    let pop3_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("POP3 Settings"))
        .visible(account.protocol() == Protocol::Pop3)
        .build();

    let leave_on_server_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Leave messages on server"))
        .active(existing_pop3.is_none_or(|s| s.leave_on_server))
        .build();
    pop3_group.add(&leave_on_server_row);

    let delete_from_server_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext(
            "Delete from server when deleted on device",
        ))
        .active(existing_pop3.is_some_and(|s| s.delete_from_server_when_deleted_on_device))
        .build();
    pop3_group.add(&delete_from_server_row);

    let keep_on_device_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext(
            "Keep on device when deleted from server",
        ))
        .active(existing_pop3.is_none_or(|s| s.keep_on_device_when_deleted_from_server))
        .build();
    pop3_group.add(&keep_on_device_row);

    let max_default = existing_pop3
        .and_then(|s| s.max_messages_to_download)
        .map_or(0.0, f64::from);
    let max_messages_row = adw::SpinRow::builder()
        .title(gettextrs::gettext(
            "Maximum messages to download (0 = unlimited)",
        ))
        .adjustment(&gtk::Adjustment::new(
            max_default,
            0.0,
            100_000.0,
            1.0,
            10.0,
            0.0,
        ))
        .build();
    pop3_group.add(&max_messages_row);

    vbox.append(&pop3_group);

    // -- IMAP system folder designation (FR-35, FR-36, US-36) --
    let system_folders_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("System Folders"))
        .description(gettextrs::gettext(
            "Designate which server folder serves each role",
        ))
        .visible(account.protocol() == Protocol::Imap)
        .build();

    let existing_sf = account.system_folders();

    let drafts_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Drafts"))
        .build();
    if let Some(v) = existing_sf.and_then(|sf| sf.drafts.as_deref()) {
        drafts_row.set_text(v);
    }
    system_folders_group.add(&drafts_row);

    let sent_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Sent"))
        .build();
    if let Some(v) = existing_sf.and_then(|sf| sf.sent.as_deref()) {
        sent_row.set_text(v);
    }
    system_folders_group.add(&sent_row);

    let archive_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Archive"))
        .build();
    if let Some(v) = existing_sf.and_then(|sf| sf.archive.as_deref()) {
        archive_row.set_text(v);
    }
    system_folders_group.add(&archive_row);

    let trash_folder_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Trash"))
        .build();
    if let Some(v) = existing_sf.and_then(|sf| sf.trash.as_deref()) {
        trash_folder_row.set_text(v);
    }
    system_folders_group.add(&trash_folder_row);

    let junk_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Junk"))
        .build();
    if let Some(v) = existing_sf.and_then(|sf| sf.junk.as_deref()) {
        junk_row.set_text(v);
    }
    system_folders_group.add(&junk_row);

    vbox.append(&system_folders_group);

    // Show/hide POP3 settings and system folders when protocol changes.
    protocol_row.connect_selected_notify(clone!(
        #[weak]
        pop3_group,
        #[weak]
        system_folders_group,
        move |row| {
            let is_pop3 = row.selected() == 1;
            pop3_group.set_visible(is_pop3);
            system_folders_group.set_visible(!is_pop3);
        }
    ));

    // -- Swipe and move defaults (FR-37, FR-38, US-37) --
    let swipe_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Swipe & Move Defaults"))
        .description(gettextrs::gettext(
            "Configure default swipe actions and move-to folder",
        ))
        .build();

    let existing_sd = account.swipe_defaults();

    let swipe_left_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Swipe left action"))
        .model(&swipe_action_string_list())
        .selected(existing_sd.map_or(0, |sd| swipe_action_to_combo(&sd.swipe_left)))
        .build();
    swipe_group.add(&swipe_left_row);

    let swipe_left_folder_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Swipe left folder"))
        .visible(
            existing_sd.is_some_and(|sd| matches!(sd.swipe_left, SwipeAction::MoveToFolder(_))),
        )
        .build();
    if let Some(SwipeAction::MoveToFolder(ref name)) = existing_sd.map(|sd| &sd.swipe_left).cloned()
    {
        swipe_left_folder_row.set_text(name);
    }
    swipe_group.add(&swipe_left_folder_row);

    swipe_left_row.connect_selected_notify(clone!(
        #[weak]
        swipe_left_folder_row,
        move |row| {
            swipe_left_folder_row.set_visible(row.selected() == 5);
        }
    ));

    let swipe_right_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Swipe right action"))
        .model(&swipe_action_string_list())
        .selected(existing_sd.map_or(0, |sd| swipe_action_to_combo(&sd.swipe_right)))
        .build();
    swipe_group.add(&swipe_right_row);

    let swipe_right_folder_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Swipe right folder"))
        .visible(
            existing_sd.is_some_and(|sd| matches!(sd.swipe_right, SwipeAction::MoveToFolder(_))),
        )
        .build();
    if let Some(SwipeAction::MoveToFolder(ref name)) =
        existing_sd.map(|sd| &sd.swipe_right).cloned()
    {
        swipe_right_folder_row.set_text(name);
    }
    swipe_group.add(&swipe_right_folder_row);

    swipe_right_row.connect_selected_notify(clone!(
        #[weak]
        swipe_right_folder_row,
        move |row| {
            swipe_right_folder_row.set_visible(row.selected() == 5);
        }
    ));

    let default_move_to_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Default move-to folder"))
        .build();
    default_move_to_row.set_tooltip_text(Some(&gettextrs::gettext(
        "Default destination folder for the move action",
    )));
    if let Some(v) = existing_sd.and_then(|sd| sd.default_move_to.as_deref()) {
        default_move_to_row.set_text(v);
    }
    swipe_group.add(&default_move_to_row);

    vbox.append(&swipe_group);

    // -- Outgoing (SMTP) server settings --
    let smtp_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Outgoing Server (SMTP)"))
        .build();

    let smtp = account.smtp();

    let smtp_host_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Host"))
        .build();
    if let Some(s) = smtp {
        smtp_host_row.set_text(&s.host);
    }
    smtp_group.add(&smtp_host_row);

    let smtp_port_default = smtp.map_or(587.0, |s| f64::from(s.port));
    let smtp_port_row = adw::SpinRow::builder()
        .title(gettextrs::gettext("Port"))
        .adjustment(&gtk::Adjustment::new(
            smtp_port_default,
            1.0,
            65535.0,
            1.0,
            10.0,
            0.0,
        ))
        .build();
    smtp_group.add(&smtp_port_row);

    let smtp_enc_selected = smtp.map_or(1, |s| encryption_to_combo(s.encryption));
    let smtp_encryption_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Encryption"))
        .model(&gtk::StringList::new(&[
            &gettextrs::gettext("SSL/TLS"),
            &gettextrs::gettext("STARTTLS"),
            &gettextrs::gettext("None"),
        ]))
        .selected(smtp_enc_selected)
        .build();
    smtp_group.add(&smtp_encryption_row);

    let smtp_auth_selected = smtp.map_or(0, |s| auth_to_combo(s.auth_method));
    let smtp_auth_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Method"))
        .model(&gtk::StringList::new(&[
            &gettextrs::gettext("PLAIN"),
            &gettextrs::gettext("LOGIN"),
            &gettextrs::gettext("OAuth2"),
        ]))
        .selected(smtp_auth_selected)
        .build();
    smtp_group.add(&smtp_auth_row);

    let smtp_username_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Username"))
        .build();
    if let Some(s) = smtp {
        smtp_username_row.set_text(&s.username);
    }
    smtp_group.add(&smtp_username_row);

    let smtp_password_row = adw::PasswordEntryRow::builder()
        .title(gettextrs::gettext("Password / Token"))
        .build();
    if let Some(s) = smtp {
        smtp_password_row.set_text(&s.credential);
    }
    smtp_group.add(&smtp_password_row);

    vbox.append(&smtp_group);

    // -- Action buttons --
    let btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .build();

    let test_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Test Connection"))
        .css_classes(["pill"])
        .build();
    btn_box.append(&test_btn);

    let save_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Save"))
        .css_classes(["suggested-action", "pill"])
        .build();
    btn_box.append(&save_btn);

    let duplicate_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Duplicate"))
        .css_classes(["pill"])
        .build();
    btn_box.append(&duplicate_btn);

    let delete_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Delete Account"))
        .css_classes(["destructive-action", "pill"])
        .build();
    btn_box.append(&delete_btn);

    vbox.append(&btn_box);

    clamp.set_child(Some(&vbox));
    scrolled.set_child(Some(&clamp));
    toast_overlay.set_child(Some(&scrolled));
    toolbar_view.set_content(Some(&toast_overlay));
    dialog.set_child(Some(&toolbar_view));

    let on_done = std::rc::Rc::new(on_done);
    let on_done_close = on_done.clone();
    let on_done_save = on_done.clone();
    let account = std::rc::Rc::new(std::cell::RefCell::new(account));

    // -- Test Connection button handler --
    test_btn.connect_clicked(clone!(
        #[weak]
        host_row,
        #[weak]
        port_row,
        #[weak]
        encryption_row,
        #[weak]
        auth_method_row,
        #[weak]
        username_row,
        #[weak]
        password_row,
        #[weak]
        protocol_row,
        #[weak]
        smtp_host_row,
        #[weak]
        smtp_port_row,
        #[weak]
        smtp_encryption_row,
        #[weak]
        smtp_auth_row,
        #[weak]
        smtp_username_row,
        #[weak]
        smtp_password_row,
        #[weak]
        toast_overlay,
        move |_| {
            let incoming_protocol = match protocol_row.selected() {
                0 => Protocol::Imap,
                _ => Protocol::Pop3,
            };

            let incoming = ServerConnectionParams {
                host: host_row.text().to_string(),
                port: port_row.value() as u16,
                encryption: combo_to_encryption(encryption_row.selected()),
                auth_method: combo_to_auth(auth_method_row.selected()),
                username: username_row.text().to_string(),
                credential: password_row.text().to_string(),
            };

            let smtp_host = smtp_host_row.text().to_string();
            let outgoing = if smtp_host.trim().is_empty() {
                None
            } else {
                Some(ServerConnectionParams {
                    host: smtp_host,
                    port: smtp_port_row.value() as u16,
                    encryption: combo_to_encryption(smtp_encryption_row.selected()),
                    auth_method: combo_to_auth(smtp_auth_row.selected()),
                    username: smtp_username_row.text().to_string(),
                    credential: smtp_password_row.text().to_string(),
                })
            };

            let request = ConnectionTestRequest {
                incoming,
                incoming_protocol,
                outgoing,
            };

            let tester = MockConnectionTester;
            match tester.test_connection(&request) {
                Ok(result) => {
                    let toast = adw::Toast::new(&result.summary());
                    toast_overlay.add_toast(toast);
                }
                Err(e) => {
                    let toast = adw::Toast::new(&e.to_string());
                    toast_overlay.add_toast(toast);
                }
            }
        }
    ));

    // -- Save button handler --
    save_btn.connect_clicked(clone!(
        #[weak]
        dialog,
        #[weak]
        name_row,
        #[weak]
        category_row,
        #[weak]
        host_row,
        #[weak]
        port_row,
        #[weak]
        encryption_row,
        #[weak]
        auth_method_row,
        #[weak]
        username_row,
        #[weak]
        password_row,
        #[weak]
        protocol_row,
        #[weak]
        smtp_host_row,
        #[weak]
        smtp_port_row,
        #[weak]
        smtp_encryption_row,
        #[weak]
        smtp_auth_row,
        #[weak]
        smtp_username_row,
        #[weak]
        smtp_password_row,
        #[weak]
        leave_on_server_row,
        #[weak]
        delete_from_server_row,
        #[weak]
        keep_on_device_row,
        #[weak]
        max_messages_row,
        #[weak]
        toast_overlay,
        #[weak]
        color_btn,
        #[weak]
        notif_row,
        #[weak]
        sync_row,
        #[weak]
        on_demand_row,
        #[weak]
        polling_row,
        #[weak]
        unmetered_row,
        #[weak]
        vpn_row,
        #[weak]
        schedule_exempt_row,
        #[weak]
        drafts_row,
        #[weak]
        sent_row,
        #[weak]
        archive_row,
        #[weak]
        trash_folder_row,
        #[weak]
        junk_row,
        #[weak]
        swipe_left_row,
        #[weak]
        swipe_left_folder_row,
        #[weak]
        swipe_right_row,
        #[weak]
        swipe_right_folder_row,
        #[weak]
        default_move_to_row,
        #[strong]
        color_active,
        #[strong]
        avatar_path,
        #[strong]
        account,
        move |_| {
            let protocol = match protocol_row.selected() {
                0 => Protocol::Imap,
                _ => Protocol::Pop3,
            };
            let encryption = combo_to_encryption(encryption_row.selected());
            let auth = combo_to_auth(auth_method_row.selected());

            let smtp_host = smtp_host_row.text().to_string();
            let smtp = if smtp_host.trim().is_empty() {
                None
            } else {
                Some(SmtpConfig {
                    host: smtp_host,
                    port: smtp_port_row.value() as u16,
                    encryption: combo_to_encryption(smtp_encryption_row.selected()),
                    auth_method: combo_to_auth(smtp_auth_row.selected()),
                    username: smtp_username_row.text().to_string(),
                    credential: smtp_password_row.text().to_string(),
                })
            };

            let pop3_settings = if protocol == Protocol::Pop3 {
                let max_val = max_messages_row.value() as u32;
                Some(Pop3Settings {
                    leave_on_server: leave_on_server_row.is_active(),
                    delete_from_server_when_deleted_on_device: delete_from_server_row.is_active(),
                    keep_on_device_when_deleted_from_server: keep_on_device_row.is_active(),
                    max_messages_to_download: if max_val == 0 { None } else { Some(max_val) },
                })
            } else {
                None
            };

            let system_folders = if protocol == Protocol::Imap {
                let sf = SystemFolders {
                    drafts: non_empty_text(&drafts_row),
                    sent: non_empty_text(&sent_row),
                    archive: non_empty_text(&archive_row),
                    trash: non_empty_text(&trash_folder_row),
                    junk: non_empty_text(&junk_row),
                };
                if sf.is_empty() {
                    None
                } else {
                    Some(sf)
                }
            } else {
                None
            };

            let color = if *color_active.borrow() {
                let rgba = color_btn.rgba();
                Some(AccountColor::new(rgba.red(), rgba.green(), rgba.blue()))
            } else {
                None
            };

            let avatar = avatar_path.borrow().clone();

            let category = {
                let text = category_row.text().to_string();
                let trimmed = text.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            };

            let swipe_defaults = {
                let sl = combo_to_swipe_action(swipe_left_row.selected(), &swipe_left_folder_row);
                let sr = combo_to_swipe_action(swipe_right_row.selected(), &swipe_right_folder_row);
                let mt = non_empty_text(&default_move_to_row);
                if sl == SwipeAction::None && sr == SwipeAction::None && mt.is_none() {
                    None
                } else {
                    Some(SwipeDefaults {
                        swipe_left: sl,
                        swipe_right: sr,
                        default_move_to: mt,
                    })
                }
            };

            let params = UpdateAccountParams {
                display_name: name_row.text().to_string(),
                protocol,
                host: host_row.text().to_string(),
                port: port_row.value() as u16,
                encryption,
                auth_method: auth,
                username: username_row.text().to_string(),
                credential: password_row.text().to_string(),
                smtp,
                pop3_settings,
                color,
                avatar_path: avatar,
                category,
                sync_enabled: sync_row.is_active(),
                on_demand: on_demand_row.is_active(),
                polling_interval_minutes: {
                    let v = polling_row.value() as u32;
                    if v == 0 {
                        None
                    } else {
                        Some(v)
                    }
                },
                unmetered_only: unmetered_row.is_active(),
                vpn_only: vpn_row.is_active(),
                schedule_exempt: schedule_exempt_row.is_active(),
                system_folders,
                swipe_defaults,
                notifications_enabled: notif_row.is_active(),
            };

            let mut acct = account.borrow_mut();
            match acct.update(params) {
                Ok(()) => {
                    on_done(Some(EditDialogResult::Updated(Box::new(acct.clone()))));
                    dialog.close();
                }
                Err(e) => {
                    let toast = adw::Toast::new(&e.to_string());
                    toast_overlay.add_toast(toast);
                }
            }
        }
    ));

    // -- Duplicate button handler (FR-31, AC-10) --
    let on_done_duplicate = on_done_save.clone();
    duplicate_btn.connect_clicked(clone!(
        #[weak]
        dialog,
        #[strong]
        account,
        move |_| {
            let source = account.borrow();
            match crate::core::duplicate_account(&source) {
                Ok(duplicated) => {
                    on_done_duplicate(Some(EditDialogResult::Duplicated(Box::new(duplicated))));
                    dialog.close();
                }
                Err(_e) => {
                    // Validation should not fail since we copy from a valid account,
                    // but handle gracefully just in case.
                }
            }
        }
    ));

    // -- Delete button handler (FR-29, FR-30, AC-9) --
    let on_done_delete = on_done_save.clone();
    let delete_account_id = account.borrow().id();
    let delete_account_name = account.borrow().display_name().to_string();
    delete_btn.connect_clicked(clone!(
        #[weak]
        dialog,
        move |_| {
            let confirm_dialog = adw::AlertDialog::builder()
                .heading(gettextrs::gettext("Delete Account?"))
                .body(format!(
                    "{} \"{}\" {}",
                    gettextrs::gettext("All data associated with"),
                    delete_account_name,
                    gettextrs::gettext("will be permanently removed: folders, messages, identities, pending operations, rules, and contacts. This cannot be undone.")
                ))
                .build();
            confirm_dialog.add_response("cancel", &gettextrs::gettext("Cancel"));
            confirm_dialog.add_response("delete", &gettextrs::gettext("Delete"));
            confirm_dialog.set_response_appearance(
                "delete",
                adw::ResponseAppearance::Destructive,
            );
            confirm_dialog.set_default_response(Some("cancel"));
            confirm_dialog.set_close_response("cancel");

            let on_done_delete = on_done_delete.clone();
            let dialog_ref = dialog.clone();
            confirm_dialog.connect_response(None, move |_confirm, response| {
                if response == "delete" {
                    on_done_delete(Some(EditDialogResult::Deleted(delete_account_id)));
                    dialog_ref.close();
                }
            });
            confirm_dialog.present(Some(&dialog));
        }
    ));

    dialog.connect_closed(move |_| {
        let _ = &on_done_close;
    });

    dialog.present(Some(parent));
}

fn combo_to_encryption(selected: u32) -> EncryptionMode {
    match selected {
        0 => EncryptionMode::SslTls,
        1 => EncryptionMode::StartTls,
        _ => EncryptionMode::None,
    }
}

fn combo_to_auth(selected: u32) -> AuthMethod {
    match selected {
        0 => AuthMethod::Plain,
        1 => AuthMethod::Login,
        _ => AuthMethod::OAuth2,
    }
}

fn protocol_to_combo(p: Protocol) -> u32 {
    match p {
        Protocol::Imap => 0,
        Protocol::Pop3 => 1,
    }
}

fn encryption_to_combo(e: EncryptionMode) -> u32 {
    match e {
        EncryptionMode::SslTls => 0,
        EncryptionMode::StartTls => 1,
        EncryptionMode::None => 2,
    }
}

fn auth_to_combo(a: AuthMethod) -> u32 {
    match a {
        AuthMethod::Plain => 0,
        AuthMethod::Login => 1,
        AuthMethod::OAuth2 => 2,
    }
}

fn non_empty_text(row: &adw::EntryRow) -> Option<String> {
    let text = row.text().to_string();
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn swipe_action_string_list() -> gtk::StringList {
    let labels = [
        gettextrs::gettext("None"),
        gettextrs::gettext("Archive"),
        gettextrs::gettext("Delete"),
        gettextrs::gettext("Mark as read"),
        gettextrs::gettext("Mark as unread"),
        gettextrs::gettext("Move to folder…"),
    ];
    let refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
    gtk::StringList::new(&refs)
}

fn combo_to_swipe_action(selected: u32, folder_row: &adw::EntryRow) -> SwipeAction {
    match selected {
        1 => SwipeAction::Archive,
        2 => SwipeAction::Delete,
        3 => SwipeAction::MarkRead,
        4 => SwipeAction::MarkUnread,
        5 => {
            let text = folder_row.text().to_string();
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                SwipeAction::None
            } else {
                SwipeAction::MoveToFolder(trimmed)
            }
        }
        _ => SwipeAction::None,
    }
}

fn swipe_action_to_combo(action: &SwipeAction) -> u32 {
    match action {
        SwipeAction::None => 0,
        SwipeAction::Archive => 1,
        SwipeAction::Delete => 2,
        SwipeAction::MarkRead => 3,
        SwipeAction::MarkUnread => 4,
        SwipeAction::MoveToFolder(_) => 5,
    }
}
