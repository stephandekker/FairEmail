use std::cell::RefCell;
use std::rc::Rc;

use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::core::auth_conversion::{can_convert_to_password, find_oauth_config_for_conversion};
use crate::core::inbound_test::{InboundTestError, InboundTestParams};
use crate::core::oauth_flow::OAuthSession;
use crate::core::provider::{
    MaxTlsVersion, ProviderEncryption, ServerConfig, UsernameType,
};
use crate::core::provider_dropdown;
use crate::core::reauth::find_oauth_config_for_reauth;
use crate::core::save_auto_test;
use crate::core::{
    Account, AccountColor, AuthMethod, ConnectionLogEntry, ConnectionState, DateHeaderPreference,
    EncryptionMode, FetchSettings, KeepAliveSettings, Pop3Settings, Protocol, QuotaInfo,
    SmtpConfig, SwipeAction, SwipeDefaults, SystemFolders, UpdateAccountParams,
};
use crate::services::inbound_tester::{InboundTester, MockInboundTester};
use crate::services::oauth_service;
use crate::services::smtp_checker::{MockSmtpChecker, SmtpChecker};
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
    /// Account was re-authorized via OAuth (FR-25, US-18, US-19).
    /// The caller must store the new tokens in the credential store
    /// and clear the needs-reauth flag.
    Reauthorized {
        account_id: uuid::Uuid,
        access_token: String,
        refresh_token: String,
        expires_in: Option<u64>,
    },
    /// Account was converted from OAuth to password authentication (FR-30).
    /// The caller must update the account credential, store the new password
    /// in the credential store, and remove the old OAuth tokens.
    ConvertedToPassword {
        account_id: uuid::Uuid,
        new_password: String,
    },
    /// Account was converted from password to OAuth authentication (FR-30).
    /// The caller must update the account credential/auth_method, store the
    /// new OAuth tokens, and remove the old password credential.
    ConvertedToOAuth {
        account_id: uuid::Uuid,
        access_token: String,
        refresh_token: String,
        expires_in: Option<u64>,
    },
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

    // -- Provider dropdown (FR-29, FR-30, FR-31) --
    let provider_db_for_dropdown =
        crate::services::user_provider_service::load_merged_provider_database();
    let dropdown_entries = provider_dropdown::build_dropdown_entries(&provider_db_for_dropdown);
    let provider_labels: Vec<String> = dropdown_entries
        .iter()
        .map(|e| {
            if e.id.is_empty() {
                gettextrs::gettext("Custom")
            } else {
                e.label.clone()
            }
        })
        .collect();
    let provider_label_refs: Vec<&str> = provider_labels.iter().map(|s| s.as_str()).collect();
    let provider_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Provider"))
        .model(&gtk::StringList::new(&provider_label_refs))
        .selected(0) // Default to Custom for existing accounts
        .build();
    server_group.add(&provider_row);

    // Guidance label for provider-specific help text (FR-31).
    let provider_guidance_label = gtk::Label::builder()
        .wrap(true)
        .xalign(0.0)
        .css_classes(["caption", "dim-label"])
        .visible(false)
        .margin_start(12)
        .margin_end(12)
        .build();

    let dropdown_entries_rc = Rc::new(dropdown_entries);

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

    // -- Provider dropdown selection handler (FR-29, FR-30, FR-31) --
    provider_row.connect_selected_notify(clone!(
        #[strong]
        dropdown_entries_rc,
        #[weak]
        host_row,
        #[weak]
        port_row,
        #[weak]
        encryption_row,
        #[weak]
        provider_guidance_label,
        move |row| {
            let idx = row.selected() as usize;
            if idx >= dropdown_entries_rc.len() {
                return;
            }
            let entry = &dropdown_entries_rc[idx];

            // Pre-fill from provider database.
            let db = crate::services::user_provider_service::load_merged_provider_database();
            if let Some(prefill) = provider_dropdown::prefill_for_provider(&db, &entry.id) {
                host_row.set_text(&prefill.hostname);
                port_row.set_value(f64::from(prefill.port));
                encryption_row.set_selected(provider_encryption_to_combo(prefill.encryption));
            }
            // "Custom" (idx 0) leaves fields as-is.

            // Update guidance text.
            if let Some(guidance) = provider_dropdown::provider_guidance(&db, &entry.id) {
                provider_guidance_label.set_text(&guidance);
                provider_guidance_label.set_visible(true);
            } else {
                provider_guidance_label.set_visible(false);
            }
        }
    ));

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
    vbox.append(&provider_guidance_label);

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
            &gettextrs::gettext("Certificate"),
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

    // -- Re-authorize button for OAuth accounts (FR-25, US-18, US-19) --
    let provider_db = crate::services::user_provider_service::load_merged_provider_database();
    let oauth_config_for_reauth = find_oauth_config_for_reauth(&account, &provider_db)
        .map(|config| config.with_tenant(account.oauth_tenant()));
    let reauth_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Re-authorize (OAuth)"))
        .css_classes(["suggested-action", "pill"])
        .tooltip_text(gettextrs::gettext(
            "Re-run the OAuth sign-in flow to obtain fresh tokens",
        ))
        .visible(oauth_config_for_reauth.is_some())
        .build();
    let reauth_row = adw::ActionRow::builder()
        .child(&reauth_btn)
        .visible(oauth_config_for_reauth.is_some())
        .build();
    auth_group.add(&reauth_row);

    // -- Auth conversion buttons (FR-30, US-17, AC-11) --
    let oauth_config_for_conversion = find_oauth_config_for_conversion(&account, &provider_db)
        .map(|config| config.with_tenant(account.oauth_tenant()));
    let convert_to_oauth_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Convert to OAuth"))
        .css_classes(["pill"])
        .tooltip_text(gettextrs::gettext(
            "Convert this account from password to OAuth authentication",
        ))
        .visible(oauth_config_for_conversion.is_some())
        .build();
    let convert_to_oauth_row = adw::ActionRow::builder()
        .child(&convert_to_oauth_btn)
        .visible(oauth_config_for_conversion.is_some())
        .build();
    auth_group.add(&convert_to_oauth_row);

    let show_convert_to_password = can_convert_to_password(&account);
    let convert_to_password_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Convert to Password"))
        .css_classes(["pill"])
        .tooltip_text(gettextrs::gettext(
            "Convert this account from OAuth to password authentication",
        ))
        .visible(show_convert_to_password)
        .build();
    let convert_to_password_row = adw::ActionRow::builder()
        .child(&convert_to_password_btn)
        .visible(show_convert_to_password)
        .build();
    auth_group.add(&convert_to_password_row);

    // -- Shared mailbox (FR-40, N-8) --
    let shared_mailbox_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Shared Mailbox"))
        .build();
    let shared_mailbox_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Shared mailbox (optional)"))
        .build();
    shared_mailbox_row.set_tooltip_text(Some(&gettextrs::gettext(
        "Enter the email address of a shared mailbox you have access to, or leave blank",
    )));
    if let Some(sm) = account.shared_mailbox() {
        shared_mailbox_row.set_text(sm);
    }
    shared_mailbox_group.add(&shared_mailbox_row);

    vbox.append(&auth_group);
    vbox.append(&shared_mailbox_group);

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

    // Initial sensitivity: disabled when leave_on_server is on (AC-4).
    let leave_on_init = existing_pop3.is_none_or(|s| s.leave_on_server);
    let delete_from_server_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext(
            "Delete from server when deleted on device",
        ))
        .active(existing_pop3.is_some_and(|s| s.delete_from_server_when_deleted_on_device))
        .sensitive(!leave_on_init)
        .build();
    pop3_group.add(&delete_from_server_row);

    // Toggle delete_from_server_row sensitivity based on leave_on_server state.
    leave_on_server_row.connect_active_notify(clone!(
        #[weak]
        delete_from_server_row,
        move |row| {
            delete_from_server_row.set_sensitive(!row.is_active());
        }
    ));

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

    let apop_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Use APOP authentication"))
        .subtitle(gettextrs::gettext(
            "Enable for servers that require APOP (uses MD5)",
        ))
        .active(existing_pop3.is_some_and(|s| s.apop_enabled))
        .build();
    pop3_group.add(&apop_row);

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
        .title(gettextrs::gettext("Swipe &amp; Move Defaults"))
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
            &gettextrs::gettext("Certificate"),
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

    // -- Advanced Security Settings (FR-4, FR-53, US-8, US-9, US-10) --
    let existing_sec = account.security_settings();
    let security_expander = adw::ExpanderRow::builder()
        .title(gettextrs::gettext("Advanced"))
        .show_enable_switch(false)
        .build();

    let dnssec_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("DNSSEC"))
        .subtitle(gettextrs::gettext(
            "Require DNSSEC validation for DNS lookups",
        ))
        .active(existing_sec.is_some_and(|s| s.dnssec))
        .build();
    security_expander.add_row(&dnssec_row);

    let dane_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("DANE"))
        .subtitle(gettextrs::gettext(
            "Require DANE (TLSA) verification for TLS",
        ))
        .active(existing_sec.is_some_and(|s| s.dane))
        .build();
    security_expander.add_row(&dane_row);

    let insecure_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Allow insecure connections"))
        .subtitle(gettextrs::gettext(
            "Skip certificate verification for this account",
        ))
        .active(existing_sec.is_some_and(|s| s.insecure))
        .build();
    security_expander.add_row(&insecure_row);

    // FR-11/FR-12: when insecure is enabled, DANE is disabled and cannot be toggled.
    dane_row.set_sensitive(!insecure_row.is_active());
    insecure_row.connect_active_notify(clone!(
        #[weak]
        dane_row,
        move |row| {
            if row.is_active() {
                dane_row.set_active(false);
            }
            dane_row.set_sensitive(!row.is_active());
        }
    ));

    let cert_fingerprint_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Certificate fingerprint (SHA-256)"))
        .build();
    if let Some(fp) = existing_sec.and_then(|s| s.certificate_fingerprint.as_deref()) {
        cert_fingerprint_row.set_text(fp);
    }
    security_expander.add_row(&cert_fingerprint_row);

    // Client certificate selector (FR-9, FR-19).
    let client_cert_path: std::rc::Rc<std::cell::RefCell<Option<String>>> = std::rc::Rc::new(
        std::cell::RefCell::new(existing_sec.and_then(|s| s.client_certificate.clone())),
    );

    let client_cert_label = gtk::Label::builder()
        .label(
            client_cert_path
                .borrow()
                .as_deref()
                .and_then(|p| std::path::Path::new(p).file_name().and_then(|n| n.to_str()))
                .unwrap_or(&gettextrs::gettext("None")),
        )
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(gtk::pango::EllipsizeMode::Middle)
        .build();

    let cert_select_btn = gtk::Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text(gettextrs::gettext("Select certificate"))
        .valign(gtk::Align::Center)
        .build();

    let cert_clear_btn = gtk::Button::builder()
        .icon_name("edit-clear-symbolic")
        .tooltip_text(gettextrs::gettext("Clear certificate"))
        .valign(gtk::Align::Center)
        .sensitive(client_cert_path.borrow().is_some())
        .build();

    let client_cert_row = adw::ActionRow::builder()
        .title(gettextrs::gettext("Client certificate"))
        .build();
    client_cert_row.add_suffix(&cert_select_btn);
    client_cert_row.add_suffix(&cert_clear_btn);
    client_cert_row.add_suffix(&client_cert_label);
    security_expander.add_row(&client_cert_row);

    // Wire up Select button to open file chooser.
    cert_select_btn.connect_clicked(clone!(
        #[strong]
        client_cert_path,
        #[weak]
        client_cert_label,
        #[weak]
        cert_clear_btn,
        #[weak]
        password_row,
        move |btn| {
            let dialog = gtk::FileDialog::builder()
                .title(gettextrs::gettext("Select Client Certificate"))
                .modal(true)
                .build();

            let filter = gtk::FileFilter::new();
            filter.add_pattern("*.p12");
            filter.add_pattern("*.pfx");
            filter.add_pattern("*.pem");
            filter.set_name(Some(&gettextrs::gettext(
                "Certificate files (*.p12, *.pfx, *.pem)",
            )));
            let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));

            let window = btn.root().and_then(|r| r.downcast::<gtk::Window>().ok());
            let path_ref = client_cert_path.clone();
            let label_ref = client_cert_label.clone();
            let clear_ref = cert_clear_btn.clone();
            let pw_ref = password_row.clone();
            dialog.open(
                window.as_ref(),
                gtk::gio::Cancellable::NONE,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&path_str);
                            label_ref.set_label(name);
                            *path_ref.borrow_mut() = Some(path_str);
                            clear_ref.set_sensitive(true);
                            // FR-19: password not required when client cert selected.
                            pw_ref.set_css_classes(&[]);
                        }
                    }
                },
            );
        }
    ));

    // Wire up Clear button (US-8-clear).
    cert_clear_btn.connect_clicked(clone!(
        #[strong]
        client_cert_path,
        #[weak]
        client_cert_label,
        #[weak]
        auth_method_row,
        move |btn| {
            *client_cert_path.borrow_mut() = None;
            client_cert_label.set_label(&gettextrs::gettext("None"));
            btn.set_sensitive(false);
            // If auth method is Certificate, reset to Plain so the user is not
            // left with a broken configuration (US-8-clear, AC-4).
            if combo_to_auth(auth_method_row.selected()) == AuthMethod::Certificate {
                auth_method_row.set_selected(auth_to_combo(AuthMethod::Plain));
            }
        }
    ));

    let auth_realm_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Authentication realm"))
        .build();
    if let Some(realm) = existing_sec.and_then(|s| s.auth_realm.as_deref()) {
        auth_realm_row.set_text(realm);
    }
    security_expander.add_row(&auth_realm_row);

    let security_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Security"))
        .build();
    security_group.add(&security_expander);
    vbox.append(&security_group);

    // -- Advanced Fetch & Keep-Alive Settings (FR-51, FR-52, FR-53) --
    let existing_fetch = account.fetch_settings();
    let existing_ka = account.keep_alive_settings();

    let fetch_expander = adw::ExpanderRow::builder()
        .title(gettextrs::gettext("Advanced"))
        .show_enable_switch(false)
        .build();

    let partial_fetch_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Partial fetch"))
        .subtitle(gettextrs::gettext(
            "Use body structure fetch for large messages",
        ))
        .active(existing_fetch.is_some_and(|s| s.partial_fetch))
        .build();
    fetch_expander.add_row(&partial_fetch_row);

    let raw_fetch_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Raw fetch"))
        .subtitle(gettextrs::gettext(
            "Fetch raw message data instead of parsed MIME",
        ))
        .active(existing_fetch.is_some_and(|s| s.raw_fetch))
        .build();
    fetch_expander.add_row(&raw_fetch_row);

    let ignore_size_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Ignore size limits"))
        .subtitle(gettextrs::gettext(
            "Ignore server-reported size limits when fetching",
        ))
        .active(existing_fetch.is_some_and(|s| s.ignore_size_limits))
        .build();
    fetch_expander.add_row(&ignore_size_row);

    let date_pref_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Date source"))
        .subtitle(gettextrs::gettext(
            "Which timestamp to display for messages",
        ))
        .model(&gtk::StringList::new(&[
            &gettextrs::gettext("Server time"),
            &gettextrs::gettext("Date header"),
            &gettextrs::gettext("Received header"),
        ]))
        .selected(match existing_fetch.map(|s| s.date_header_preference) {
            Some(DateHeaderPreference::DateHeader) => 1,
            Some(DateHeaderPreference::ReceivedHeader) => 2,
            _ => 0,
        })
        .build();
    fetch_expander.add_row(&date_pref_row);

    let utf8_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("UTF-8 support"))
        .subtitle(gettextrs::gettext(
            "Enable IMAP UTF8=ACCEPT for this account",
        ))
        .active(existing_fetch.is_some_and(|s| s.utf8_support))
        .build();
    fetch_expander.add_row(&utf8_row);

    let noop_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Use NOOP instead of IDLE"))
        .subtitle(gettextrs::gettext(
            "Send NOOP commands for keep-alive instead of IMAP IDLE",
        ))
        .active(existing_ka.is_some_and(|s| s.use_noop_instead_of_idle))
        .build();
    fetch_expander.add_row(&noop_row);

    let fetch_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Fetch & Keep-Alive"))
        .build();
    fetch_group.add(&fetch_expander);
    vbox.append(&fetch_group);

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
    let on_done_reauth = on_done.clone();
    let on_done_convert_oauth = on_done.clone();
    let on_done_convert_password = on_done.clone();
    let account = std::rc::Rc::new(std::cell::RefCell::new(account));

    // -- Session-level flag: has a successful connection test been run? (FR-42, US-22) --
    let test_passed_in_session: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

    // -- Detected fingerprint from last failed test (FR-15, US-17) --
    let detected_fingerprint: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // -- Test results area (hidden until test completes) --
    let test_results_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Test Results"))
        .visible(false)
        .build();
    let test_results_label = gtk::Label::builder()
        .wrap(true)
        .xalign(0.0)
        .css_classes(["body"])
        .selectable(true)
        .build();
    test_results_group.add(&adw::ActionRow::builder().child(&test_results_label).build());
    // "Trust this certificate" button — shown only on untrusted-cert errors (FR-15, US-17).
    let trust_cert_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Trust this certificate"))
        .css_classes(["suggested-action"])
        .visible(false)
        .build();
    test_results_group.add(&adw::ActionRow::builder().child(&trust_cert_btn).build());
    vbox.append(&test_results_group);

    // -- Progress spinner (hidden until test starts) --
    let test_spinner = gtk::Spinner::builder()
        .spinning(false)
        .visible(false)
        .halign(gtk::Align::Center)
        .margin_top(6)
        .margin_bottom(6)
        .build();
    vbox.append(&test_spinner);

    // -- Test Connection button handler --
    test_btn.connect_clicked(clone!(
        #[strong]
        test_passed_in_session,
        #[strong]
        detected_fingerprint,
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
        toast_overlay,
        #[weak]
        test_btn,
        #[weak]
        save_btn,
        #[weak]
        name_row,
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
        test_spinner,
        #[weak]
        test_results_group,
        #[weak]
        test_results_label,
        #[weak]
        duplicate_btn,
        #[weak]
        delete_btn,
        #[weak]
        insecure_row,
        #[weak]
        dane_row,
        #[weak]
        dnssec_row,
        #[weak]
        cert_fingerprint_row,
        #[weak]
        trust_cert_btn,
        #[weak]
        auth_realm_row,
        #[strong]
        client_cert_path,
        move |_| {
            let params = InboundTestParams {
                host: host_row.text().to_string(),
                port: port_row.value() as u16,
                encryption: combo_to_encryption(encryption_row.selected()),
                auth_method: combo_to_auth(auth_method_row.selected()),
                username: username_row.text().to_string(),
                credential: password_row.text().to_string(),
                protocol: match protocol_row.selected() {
                    0 => Protocol::Imap,
                    _ => Protocol::Pop3,
                },
                insecure: insecure_row.is_active(),
                accepted_fingerprint: {
                    let fp = cert_fingerprint_row.text().trim().to_string();
                    if fp.is_empty() {
                        None
                    } else {
                        Some(fp)
                    }
                },
                client_certificate: client_cert_path.borrow().clone(),
                dane: dane_row.is_active(),
                dnssec: dnssec_row.is_active(),
                auth_realm: {
                    let r = auth_realm_row.text().trim().to_string();
                    if r.is_empty() {
                        None
                    } else {
                        Some(r)
                    }
                },
            };

            // Disable all input fields and buttons during test.
            set_edit_form_sensitive(
                &name_row,
                &host_row,
                &port_row,
                &encryption_row,
                &auth_method_row,
                &username_row,
                &password_row,
                &protocol_row,
                &smtp_host_row,
                &smtp_port_row,
                &smtp_encryption_row,
                &smtp_auth_row,
                &smtp_username_row,
                &smtp_password_row,
                &test_btn,
                &save_btn,
                &duplicate_btn,
                &delete_btn,
                false,
            );

            // Show spinner.
            test_spinner.set_visible(true);
            test_spinner.set_spinning(true);
            test_results_group.set_visible(false);

            // Run the test asynchronously (simulated with a short delay).
            glib::timeout_add_local_once(
                std::time::Duration::from_millis(200),
                clone!(
                    #[strong]
                    test_passed_in_session,
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
                    toast_overlay,
                    #[weak]
                    test_btn,
                    #[weak]
                    save_btn,
                    #[weak]
                    name_row,
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
                    test_spinner,
                    #[weak]
                    test_results_group,
                    #[weak]
                    test_results_label,
                    #[weak]
                    duplicate_btn,
                    #[weak]
                    delete_btn,
                    #[weak]
                    trust_cert_btn,
                    #[strong]
                    detected_fingerprint,
                    move || {
                        let tester = MockInboundTester;
                        let inbound_result = tester.test_inbound(&params);

                        // Run SMTP test if SMTP host is configured.
                        let smtp_host_val = smtp_host_row.text().to_string();
                        let smtp_result = if !smtp_host_val.trim().is_empty() {
                            let smtp_port_val = smtp_port_row.value() as u16;
                            let smtp_enc = combo_to_encryption(smtp_encryption_row.selected());
                            let smtp_user = smtp_username_row.text().to_string();
                            let smtp_pass = smtp_password_row.text().to_string();
                            let provider =
                                build_smtp_provider(smtp_host_val.trim(), smtp_port_val, smtp_enc);
                            let checker = MockSmtpChecker;
                            Some(checker.check_smtp(&smtp_user, &smtp_pass, &provider, None))
                        } else {
                            None
                        };

                        // Stop spinner.
                        test_spinner.set_spinning(false);
                        test_spinner.set_visible(false);

                        // Re-enable form.
                        set_edit_form_sensitive(
                            &name_row,
                            &host_row,
                            &port_row,
                            &encryption_row,
                            &auth_method_row,
                            &username_row,
                            &password_row,
                            &protocol_row,
                            &smtp_host_row,
                            &smtp_port_row,
                            &smtp_encryption_row,
                            &smtp_auth_row,
                            &smtp_username_row,
                            &smtp_password_row,
                            &test_btn,
                            &save_btn,
                            &duplicate_btn,
                            &delete_btn,
                            true,
                        );

                        let mut text = String::new();
                        let mut any_error = false;
                        // Hide the trust button by default; only show on untrusted cert.
                        trust_cert_btn.set_visible(false);
                        *detected_fingerprint.borrow_mut() = None;

                        // Display inbound results.
                        match inbound_result {
                            Ok(success) => {
                                if params.protocol == Protocol::Pop3 {
                                    // POP3 has no server-side folders or IMAP
                                    // capabilities; just confirm success.
                                    text.push_str(&gettextrs::gettext(
                                        "POP3 connection successful",
                                    ));
                                } else {
                                    if !success.folders.is_empty() {
                                        text.push_str(&gettextrs::gettext("Folders:"));
                                        text.push('\n');
                                        text.push_str(&success.format_folder_list());
                                        text.push_str("\n\n");
                                    }
                                    text.push_str(&success.format_capabilities());

                                    if !success.idle_supported {
                                        let warning_toast = adw::Toast::new(&gettextrs::gettext(
                                        "Server does not support IDLE — polling fallback will be used",
                                    ));
                                        toast_overlay.add_toast(warning_toast);
                                    }
                                }
                            }
                            Err(InboundTestError::TlsHandshakeFailed {
                                ref message,
                                fingerprint: Some(ref fp),
                            }) => {
                                any_error = true;
                                text.push_str(message);
                                text.push_str("\n\n");
                                text.push_str(&gettextrs::gettext("Certificate fingerprint:"));
                                text.push('\n');
                                text.push_str(fp);
                                // Show the "Trust this certificate" action (FR-15, US-17).
                                *detected_fingerprint.borrow_mut() = Some(fp.clone());
                                trust_cert_btn.set_visible(true);
                            }
                            Err(e) => {
                                any_error = true;
                                text.push_str(&e.to_string());
                            }
                        }

                        // Display SMTP results.
                        if let Some(smtp_res) = smtp_result {
                            if !text.is_empty() {
                                text.push_str("\n\n");
                            }
                            match smtp_res {
                                Ok(smtp_success) => {
                                    text.push_str(&gettextrs::gettext("SMTP: OK"));
                                    if let Some(size) = smtp_success.max_message_size {
                                        let size_mb = size / (1024 * 1024);
                                        text.push_str(&format!(
                                            "\n{} {} ({} bytes)",
                                            gettextrs::gettext("Max message size:"),
                                            gettextrs::gettext(format!("{size_mb} MiB")),
                                            size
                                        ));
                                    }
                                }
                                Err(smtp_err) => {
                                    any_error = true;
                                    text.push_str(&format!(
                                        "{} {}",
                                        gettextrs::gettext("SMTP test failed:"),
                                        smtp_err
                                    ));
                                }
                            }
                        }

                        test_results_label.set_text(&text);
                        test_results_group.set_visible(true);

                        if any_error {
                            let toast = adw::Toast::new(&gettextrs::gettext(
                                "Connection test completed with errors",
                            ));
                            toast_overlay.add_toast(toast);
                        } else {
                            // Mark that a successful test was run in this session (FR-42, US-22).
                            *test_passed_in_session.borrow_mut() = true;
                            let toast =
                                adw::Toast::new(&gettextrs::gettext("Connection successful"));
                            toast_overlay.add_toast(toast);
                        }
                    }
                ),
            );
        }
    ));

    // -- "Trust this certificate" button handler (FR-15, US-17) --
    trust_cert_btn.connect_clicked(clone!(
        #[strong]
        detected_fingerprint,
        #[weak]
        cert_fingerprint_row,
        #[weak]
        trust_cert_btn,
        #[weak]
        toast_overlay,
        move |_| {
            if let Some(fp) = detected_fingerprint.borrow().clone() {
                cert_fingerprint_row.set_text(&fp);
                trust_cert_btn.set_visible(false);
                let toast = adw::Toast::new(&gettextrs::gettext(
                    "Certificate pinned — re-run test to verify",
                ));
                toast_overlay.add_toast(toast);
            }
        }
    ));

    // -- Save button handler --
    save_btn.connect_clicked(clone!(
        #[strong]
        test_passed_in_session,
        #[weak]
        dialog,
        #[weak]
        name_row,
        #[weak]
        category_row,
        #[weak]
        shared_mailbox_row,
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
        #[weak]
        dnssec_row,
        #[weak]
        dane_row,
        #[weak]
        insecure_row,
        #[weak]
        cert_fingerprint_row,
        #[strong]
        client_cert_path,
        #[weak]
        auth_realm_row,
        #[weak]
        partial_fetch_row,
        #[weak]
        raw_fetch_row,
        #[weak]
        ignore_size_row,
        #[weak]
        date_pref_row,
        #[weak]
        utf8_row,
        #[weak]
        noop_row,
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
                    apop_enabled: apop_row.is_active(),
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
                security_settings: {
                    let dnssec = dnssec_row.is_active();
                    let dane = dane_row.is_active();
                    let insecure_val = insecure_row.is_active();
                    let fp_text = cert_fingerprint_row.text().trim().to_string();
                    let fingerprint = if fp_text.is_empty() {
                        None
                    } else {
                        Some(fp_text)
                    };
                    let client_cert = client_cert_path.borrow().clone();
                    let realm_text = auth_realm_row.text().trim().to_string();
                    let realm = if realm_text.is_empty() {
                        None
                    } else {
                        Some(realm_text)
                    };
                    if dnssec
                        || dane
                        || insecure_val
                        || fingerprint.is_some()
                        || client_cert.is_some()
                        || realm.is_some()
                    {
                        Some(crate::core::SecuritySettings {
                            dnssec,
                            dane,
                            insecure: insecure_val,
                            certificate_fingerprint: fingerprint,
                            client_certificate: client_cert,
                            auth_realm: realm,
                            allow_insecure_auth: false,
                            max_tls_version: None,
                            disable_ip_connections: false,
                        })
                    } else {
                        None
                    }
                },
                fetch_settings: {
                    let partial = partial_fetch_row.is_active();
                    let raw = raw_fetch_row.is_active();
                    let ignore_size = ignore_size_row.is_active();
                    let date_pref = match date_pref_row.selected() {
                        1 => DateHeaderPreference::DateHeader,
                        2 => DateHeaderPreference::ReceivedHeader,
                        _ => DateHeaderPreference::ServerTime,
                    };
                    let utf8 = utf8_row.is_active();
                    if partial
                        || raw
                        || ignore_size
                        || date_pref != DateHeaderPreference::ServerTime
                        || utf8
                    {
                        Some(FetchSettings {
                            partial_fetch: partial,
                            raw_fetch: raw,
                            ignore_size_limits: ignore_size,
                            date_header_preference: date_pref,
                            utf8_support: utf8,
                        })
                    } else {
                        None
                    }
                },
                keep_alive_settings: {
                    let noop = noop_row.is_active();
                    if noop {
                        Some(KeepAliveSettings {
                            use_noop_instead_of_idle: true,
                        })
                    } else {
                        None
                    }
                },
                oauth_tenant: None,
                shared_mailbox: {
                    let text = shared_mailbox_row.text().trim().to_string();
                    if text.is_empty() {
                        None
                    } else {
                        Some(text)
                    }
                },
            };

            // FR-42, NFR-7, US-22: auto-test connection before save when
            // sync is enabled, no prior successful test in this session, and
            // connection-relevant parameters have actually changed.
            let acct_ref = account.borrow();
            if save_auto_test::should_auto_test_existing_account(
                &acct_ref,
                &params,
                *test_passed_in_session.borrow(),
            ) {
                let test_params = InboundTestParams {
                    host: params.host.clone(),
                    port: params.port,
                    encryption: params.encryption,
                    auth_method: params.auth_method,
                    username: params.username.clone(),
                    credential: params.credential.clone(),
                    protocol: params.protocol,
                    insecure: params
                        .security_settings
                        .as_ref()
                        .is_some_and(|s| s.insecure),
                    accepted_fingerprint: params
                        .security_settings
                        .as_ref()
                        .and_then(|s| s.certificate_fingerprint.clone()),
                    client_certificate: params
                        .security_settings
                        .as_ref()
                        .and_then(|s| s.client_certificate.clone()),
                    dane: params.security_settings.as_ref().is_some_and(|s| s.dane),
                    dnssec: params.security_settings.as_ref().is_some_and(|s| s.dnssec),
                    auth_realm: params
                        .security_settings
                        .as_ref()
                        .and_then(|s| s.auth_realm.clone()),
                };
                let tester = MockInboundTester;
                let result = tester.test_inbound(&test_params);
                if let Err(e) = result {
                    drop(acct_ref);
                    let toast = adw::Toast::new(&format!(
                        "{} {}",
                        gettextrs::gettext("Connection check failed:"),
                        e
                    ));
                    toast_overlay.add_toast(toast);
                    return;
                }
            }
            drop(acct_ref);

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

    // -- Re-authorize button handler (FR-25, US-18, US-19) --
    if let Some(oauth_config) = oauth_config_for_reauth {
        let reauth_account_id = account.borrow().id();
        reauth_btn.connect_clicked(clone!(
            #[weak]
            dialog,
            #[weak]
            toast_overlay,
            #[weak]
            reauth_btn,
            move |_| {
                reauth_btn.set_sensitive(false);

                // Show progress toast.
                let progress_toast = adw::Toast::builder()
                    .title(gettextrs::gettext("Opening browser for re-authorization…"))
                    .build();
                toast_overlay.add_toast(progress_toast);

                // Run OAuth flow on a background thread, poll from main loop.
                let oauth_config_clone = oauth_config.clone();
                let browser_pref = crate::services::oauth_service::load_browser_preference();
                let (tx, rx) = std::sync::mpsc::channel::<ReauthOAuthMessage>();
                std::thread::spawn(move || {
                    run_reauth_oauth_thread(oauth_config_clone, tx, browser_pref);
                });

                let on_done_reauth = on_done_reauth.clone();
                let dialog_ref = dialog.clone();
                let toast_overlay_ref = toast_overlay.clone();
                let reauth_btn_ref = reauth_btn.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    match rx.try_recv() {
                        Ok(ReauthOAuthMessage::Success {
                            access_token,
                            refresh_token,
                            expires_in,
                        }) => {
                            on_done_reauth(Some(EditDialogResult::Reauthorized {
                                account_id: reauth_account_id,
                                access_token,
                                refresh_token,
                                expires_in,
                            }));
                            dialog_ref.close();
                            glib::ControlFlow::Break
                        }
                        Ok(ReauthOAuthMessage::Error(err)) => {
                            reauth_btn_ref.set_sensitive(true);
                            let toast = adw::Toast::new(&format!(
                                "{} {}",
                                gettextrs::gettext("Re-authorization failed:"),
                                err
                            ));
                            toast_overlay_ref.add_toast(toast);
                            glib::ControlFlow::Break
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            reauth_btn_ref.set_sensitive(true);
                            glib::ControlFlow::Break
                        }
                    }
                });
            }
        ));
    }

    // -- Convert to OAuth button handler (FR-30, US-17) --
    if let Some(oauth_config) = oauth_config_for_conversion {
        let convert_account_id = account.borrow().id();
        convert_to_oauth_btn.connect_clicked(clone!(
            #[weak]
            dialog,
            #[weak]
            toast_overlay,
            #[weak]
            convert_to_oauth_btn,
            move |_| {
                // Show confirmation dialog before starting the OAuth flow.
                let confirm = adw::AlertDialog::builder()
                    .heading(gettextrs::gettext("Convert to OAuth?"))
                    .body(gettextrs::gettext(
                        "This will replace password authentication with OAuth. \
                         All account settings, folders, and messages will be preserved.",
                    ))
                    .build();
                confirm.add_response("cancel", &gettextrs::gettext("Cancel"));
                confirm.add_response("convert", &gettextrs::gettext("Convert"));
                confirm.set_response_appearance("convert", adw::ResponseAppearance::Suggested);

                let oauth_config = oauth_config.clone();
                let on_done_convert_oauth = on_done_convert_oauth.clone();
                let dialog_ref = dialog.clone();
                let toast_overlay_ref = toast_overlay.clone();
                let convert_to_oauth_btn_ref = convert_to_oauth_btn.clone();

                confirm.connect_response(None, move |_, response| {
                    if response != "convert" {
                        return;
                    }
                    convert_to_oauth_btn_ref.set_sensitive(false);

                    let progress_toast = adw::Toast::builder()
                        .title(gettextrs::gettext("Opening browser for OAuth sign-in…"))
                        .build();
                    toast_overlay_ref.add_toast(progress_toast);

                    let oauth_config_clone = oauth_config.clone();
                    let browser_pref = crate::services::oauth_service::load_browser_preference();
                    let (tx, rx) = std::sync::mpsc::channel::<ReauthOAuthMessage>();
                    std::thread::spawn(move || {
                        run_reauth_oauth_thread(oauth_config_clone, tx, browser_pref);
                    });

                    let on_done = on_done_convert_oauth.clone();
                    let dialog_ref2 = dialog_ref.clone();
                    let toast_ref2 = toast_overlay_ref.clone();
                    let btn_ref2 = convert_to_oauth_btn_ref.clone();
                    glib::timeout_add_local(
                        std::time::Duration::from_millis(100),
                        move || match rx.try_recv() {
                            Ok(ReauthOAuthMessage::Success {
                                access_token,
                                refresh_token,
                                expires_in,
                            }) => {
                                on_done(Some(EditDialogResult::ConvertedToOAuth {
                                    account_id: convert_account_id,
                                    access_token,
                                    refresh_token,
                                    expires_in,
                                }));
                                dialog_ref2.close();
                                glib::ControlFlow::Break
                            }
                            Ok(ReauthOAuthMessage::Error(err)) => {
                                btn_ref2.set_sensitive(true);
                                let toast = adw::Toast::new(&format!(
                                    "{} {}",
                                    gettextrs::gettext("OAuth conversion failed:"),
                                    err
                                ));
                                toast_ref2.add_toast(toast);
                                glib::ControlFlow::Break
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => {
                                glib::ControlFlow::Continue
                            }
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                btn_ref2.set_sensitive(true);
                                glib::ControlFlow::Break
                            }
                        },
                    );
                });
                confirm.present(Some(&dialog));
            }
        ));
    }

    // -- Convert to Password button handler (FR-30, US-17) --
    if show_convert_to_password {
        let convert_pw_account_id = account.borrow().id();
        convert_to_password_btn.connect_clicked(clone!(
            #[weak]
            dialog,
            #[weak]
            toast_overlay,
            move |_| {
                // Build a dialog asking for the new password with confirmation.
                let pw_dialog = adw::AlertDialog::builder()
                    .heading(gettextrs::gettext("Convert to Password Authentication?"))
                    .body(gettextrs::gettext(
                        "Enter the password for this account. All account settings, \
                         folders, and messages will be preserved. The existing OAuth \
                         tokens will be removed.",
                    ))
                    .build();
                pw_dialog.add_response("cancel", &gettextrs::gettext("Cancel"));
                pw_dialog.add_response("convert", &gettextrs::gettext("Convert"));
                pw_dialog.set_response_appearance("convert", adw::ResponseAppearance::Suggested);

                let pw_entry = adw::PasswordEntryRow::builder()
                    .title(gettextrs::gettext("New password"))
                    .build();
                let extra_content = gtk::Box::new(gtk::Orientation::Vertical, 6);
                extra_content.append(&adw::PreferencesGroup::builder().build());
                let pw_group = adw::PreferencesGroup::builder().build();
                pw_group.add(&pw_entry);
                extra_content.append(&pw_group);
                pw_dialog.set_extra_child(Some(&extra_content));

                let on_done = on_done_convert_password.clone();
                let dialog_ref = dialog.clone();
                let toast_ref = toast_overlay.clone();
                pw_dialog.connect_response(None, move |_, response| {
                    if response != "convert" {
                        return;
                    }
                    let new_password = pw_entry.text().to_string();
                    if new_password.trim().is_empty() {
                        let toast =
                            adw::Toast::new(&gettextrs::gettext("Password must not be empty."));
                        toast_ref.add_toast(toast);
                        return;
                    }
                    on_done(Some(EditDialogResult::ConvertedToPassword {
                        account_id: convert_pw_account_id,
                        new_password,
                    }));
                    dialog_ref.close();
                });
                pw_dialog.present(Some(&dialog));
            }
        ));
    }

    dialog.connect_closed(move |_| {
        let _ = &on_done_close;
    });

    dialog.present(Some(parent));
}

/// Messages from the re-authorization OAuth background thread.
pub(crate) enum ReauthOAuthMessage {
    Success {
        access_token: String,
        refresh_token: String,
        expires_in: Option<u64>,
    },
    Error(String),
}

/// Run the OAuth flow for re-authorization on a background thread.
///
/// Reuses the same core OAuth flow as the setup wizard (story 2) but sends
/// the result back via a channel for token replacement on the existing account.
pub(crate) fn run_reauth_oauth_thread(
    oauth_config: crate::core::provider::OAuthConfig,
    tx: std::sync::mpsc::Sender<ReauthOAuthMessage>,
    oauth_browser_preference: Option<String>,
) {
    // Step 1: Bind redirect listener, falling back to jump page if unavailable (FR-11).
    let (listener, port, redirect_method) = match oauth_service::bind_redirect_listener() {
        Ok((l, p)) => (l, p, crate::core::oauth_flow::RedirectMethod::LocalServer),
        Err(bind_err) => match oauth_service::bind_redirect_listener() {
            Ok((l, p)) => (
                l,
                p,
                crate::core::oauth_flow::RedirectMethod::JumpPage {
                    jump_url: crate::core::oauth_flow::DEFAULT_JUMP_PAGE_URL.to_string(),
                },
            ),
            Err(_) => {
                let _ = tx.send(ReauthOAuthMessage::Error(bind_err.to_string()));
                return;
            }
        },
    };

    // Step 2: Build session and open browser.
    let session = OAuthSession::new_with_redirect(oauth_config, port, redirect_method);
    let url = session.authorization_url();
    if let Err(e) =
        oauth_service::open_browser_with_selection(&url, oauth_browser_preference.as_deref())
    {
        let _ = tx.send(ReauthOAuthMessage::Error(e.to_string()));
        return;
    }

    // Step 3: Wait for callback.
    let callback = match oauth_service::wait_for_callback(listener) {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(ReauthOAuthMessage::Error(e.to_string()));
            return;
        }
    };

    // Step 4: Validate state.
    if let Err(e) = session.validate_state(Some(&callback.state)) {
        let _ = tx.send(ReauthOAuthMessage::Error(e.to_string()));
        return;
    }

    // Step 5: Exchange code for tokens.
    let exchange_params = session.token_exchange_params(&callback.code);
    let token_response = match oauth_service::exchange_code_for_tokens(exchange_params) {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(ReauthOAuthMessage::Error(e.to_string()));
            return;
        }
    };

    // Step 6: Validate (must include refresh token).
    let validated = match crate::core::oauth_flow::validate_token_response(token_response) {
        Ok(v) => v,
        Err(e) => {
            let _ = tx.send(ReauthOAuthMessage::Error(e.to_string()));
            return;
        }
    };

    let _ = tx.send(ReauthOAuthMessage::Success {
        access_token: validated.access_token,
        refresh_token: validated.refresh_token,
        expires_in: validated.expires_in,
    });
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
        2 => AuthMethod::OAuth2,
        _ => AuthMethod::Certificate,
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
        AuthMethod::Certificate => 3,
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

/// Enable or disable the main form widgets during a connection test.
#[allow(clippy::too_many_arguments)]
fn set_edit_form_sensitive(
    name_row: &adw::EntryRow,
    host_row: &adw::EntryRow,
    port_row: &adw::SpinRow,
    encryption_row: &adw::ComboRow,
    auth_method_row: &adw::ComboRow,
    username_row: &adw::EntryRow,
    password_row: &adw::PasswordEntryRow,
    protocol_row: &adw::ComboRow,
    smtp_host_row: &adw::EntryRow,
    smtp_port_row: &adw::SpinRow,
    smtp_encryption_row: &adw::ComboRow,
    smtp_auth_row: &adw::ComboRow,
    smtp_username_row: &adw::EntryRow,
    smtp_password_row: &adw::PasswordEntryRow,
    test_btn: &gtk::Button,
    save_btn: &gtk::Button,
    duplicate_btn: &gtk::Button,
    delete_btn: &gtk::Button,
    sensitive: bool,
) {
    name_row.set_sensitive(sensitive);
    host_row.set_sensitive(sensitive);
    port_row.set_sensitive(sensitive);
    encryption_row.set_sensitive(sensitive);
    auth_method_row.set_sensitive(sensitive);
    username_row.set_sensitive(sensitive);
    password_row.set_sensitive(sensitive);
    protocol_row.set_sensitive(sensitive);
    smtp_host_row.set_sensitive(sensitive);
    smtp_port_row.set_sensitive(sensitive);
    smtp_encryption_row.set_sensitive(sensitive);
    smtp_auth_row.set_sensitive(sensitive);
    smtp_username_row.set_sensitive(sensitive);
    smtp_password_row.set_sensitive(sensitive);
    test_btn.set_sensitive(sensitive);
    save_btn.set_sensitive(sensitive);
    duplicate_btn.set_sensitive(sensitive);
    delete_btn.set_sensitive(sensitive);
}

fn encryption_to_provider(enc: EncryptionMode) -> ProviderEncryption {
    match enc {
        EncryptionMode::SslTls => ProviderEncryption::SslTls,
        EncryptionMode::StartTls => ProviderEncryption::StartTls,
        EncryptionMode::None => ProviderEncryption::None,
    }
}

/// Build a minimal `Provider` from SMTP UI fields for use with `SmtpChecker`.
fn build_smtp_provider(
    host: &str,
    port: u16,
    encryption: EncryptionMode,
) -> crate::core::provider::Provider {
    crate::core::provider::Provider {
        id: String::new(),
        display_name: String::new(),
        domain_patterns: vec![],
        mx_patterns: vec![],
        incoming: ServerConfig {
            hostname: String::new(),
            port: 0,
            encryption: ProviderEncryption::None,
        },
        outgoing: ServerConfig {
            hostname: host.to_string(),
            port,
            encryption: encryption_to_provider(encryption),
        },
        username_type: UsernameType::EmailAddress,
        keep_alive_interval: 0,
        noop_keep_alive: false,
        partial_fetch: false,
        max_tls_version: MaxTlsVersion::Tls1_3,
        app_password_required: false,
        disable_ip_connections: false,
        requires_manual_enablement: false,
        documentation_url: None,
        localized_docs: vec![],
        oauth: None,
        display_order: 0,
        enabled: false,
        supports_shared_mailbox: false,
        subtitle: None,
        registration_url: None,
        app_password_url: None,
        graph: None,
        debug_only: false,
        variant_of: None,
    }
}

fn provider_encryption_to_combo(enc: ProviderEncryption) -> u32 {
    match enc {
        ProviderEncryption::SslTls => 0,
        ProviderEncryption::StartTls => 1,
        ProviderEncryption::None => 2,
    }
}
