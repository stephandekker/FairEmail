//! SMTP Identity Configuration dialog (FR-45 through FR-49).

use std::cell::RefCell;
use std::rc::Rc;

use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::core::account::EncryptionMode;
use crate::core::port_autofill::{should_autofill, smtp_default_port};
use crate::core::provider::{
    MaxTlsVersion, Provider, ProviderEncryption, ServerConfig, UsernameType,
};
use crate::core::smtp_check::SmtpCheckError;
use crate::core::smtp_identity::validate_smtp_identity;
use crate::core::smtp_test_diagnostics::diagnose_smtp_error;
use crate::services::identity_store::IdentityRow;
use crate::services::smtp_checker::MockSmtpChecker;
use crate::services::smtp_checker::SmtpChecker;

/// An inbound account entry for the associated-account dropdown.
#[derive(Debug, Clone)]
pub struct InboundAccountEntry {
    pub id: String,
    pub display_name: String,
    pub username: String,
    pub password: String,
}

/// Result returned when the dialog is saved.
#[derive(Debug, Clone)]
pub struct SmtpIdentityDialogResult {
    pub identity: IdentityRow,
    /// The SMTP password (stored in the keychain, not the DB).
    pub password: String,
}

fn combo_to_encryption(selected: u32) -> EncryptionMode {
    match selected {
        0 => EncryptionMode::SslTls,
        1 => EncryptionMode::StartTls,
        _ => EncryptionMode::None,
    }
}

fn encryption_to_combo(enc: EncryptionMode) -> u32 {
    match enc {
        EncryptionMode::SslTls => 0,
        EncryptionMode::StartTls => 1,
        EncryptionMode::None => 2,
    }
}

fn encryption_from_str(s: &str) -> EncryptionMode {
    match s {
        "SslTls" => EncryptionMode::SslTls,
        "StartTls" => EncryptionMode::StartTls,
        _ => EncryptionMode::None,
    }
}

fn encryption_to_str(enc: EncryptionMode) -> &'static str {
    match enc {
        EncryptionMode::SslTls => "SslTls",
        EncryptionMode::StartTls => "StartTls",
        EncryptionMode::None => "None",
    }
}

fn provider_encryption_to_combo(enc: ProviderEncryption) -> u32 {
    match enc {
        ProviderEncryption::SslTls => 0,
        ProviderEncryption::StartTls => 1,
        ProviderEncryption::None => 2,
    }
}

/// Show the SMTP Identity Configuration dialog.
///
/// `accounts` provides the list of existing inbound accounts for the dropdown.
/// If `existing` is `Some`, the dialog edits that identity; otherwise it creates a new one.
/// `existing_password` is the current SMTP password for editing (read from the keychain).
/// `on_done` is called with the result on save, or `None` on cancel.
#[allow(clippy::too_many_lines)]
pub(crate) fn show(
    parent: &adw::ApplicationWindow,
    accounts: Vec<InboundAccountEntry>,
    existing: Option<IdentityRow>,
    existing_password: Option<String>,
    on_done: impl Fn(Option<SmtpIdentityDialogResult>) + 'static,
) {
    let is_edit = existing.is_some();
    let title = if is_edit {
        gettextrs::gettext("Edit SMTP Identity")
    } else {
        gettextrs::gettext("New SMTP Identity")
    };

    let dialog = adw::Dialog::builder()
        .title(&title)
        .content_width(460)
        .content_height(600)
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

    // -- Identity fields (email, display name) --
    let identity_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Identity"))
        .build();

    let email_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Email address"))
        .build();
    identity_group.add(&email_row);

    let display_name_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Display name"))
        .build();
    identity_group.add(&display_name_row);

    // -- Associated inbound account dropdown --
    let account_names: Vec<String> = accounts.iter().map(|a| a.display_name.clone()).collect();
    let account_list =
        gtk::StringList::new(&account_names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let account_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Associated account"))
        .model(&account_list)
        .build();
    identity_group.add(&account_row);

    vbox.append(&identity_group);

    // -- SMTP server fields --
    let smtp_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Outgoing Server (SMTP)"))
        .build();

    // -- Domain-based auto-config (FR-49) --
    let domain_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Domain"))
        .build();
    domain_row.set_tooltip_text(Some(&gettextrs::gettext(
        "Enter domain to auto-detect SMTP server settings",
    )));

    let auto_config_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Auto-config"))
        .valign(gtk::Align::Center)
        .css_classes(["suggested-action"])
        .build();
    auto_config_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Auto-detect SMTP server settings from domain",
    ))]);

    let auto_config_spinner = gtk::Spinner::builder()
        .spinning(false)
        .visible(false)
        .valign(gtk::Align::Center)
        .build();

    domain_row.add_suffix(&auto_config_spinner);
    domain_row.add_suffix(&auto_config_btn);
    smtp_group.add(&domain_row);

    let host_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Host"))
        .build();
    smtp_group.add(&host_row);

    let port_row = adw::SpinRow::builder()
        .title(gettextrs::gettext("Port"))
        .adjustment(&gtk::Adjustment::new(587.0, 1.0, 65535.0, 1.0, 10.0, 0.0))
        .build();
    smtp_group.add(&port_row);

    let encryption_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Encryption"))
        .model(&gtk::StringList::new(&[
            &gettextrs::gettext("SSL/TLS"),
            &gettextrs::gettext("STARTTLS"),
            &gettextrs::gettext("None"),
        ]))
        .selected(1) // STARTTLS default
        .build();
    smtp_group.add(&encryption_row);

    let username_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Username"))
        .build();
    smtp_group.add(&username_row);

    let password_row = adw::PasswordEntryRow::builder()
        .title(gettextrs::gettext("Password"))
        .build();
    smtp_group.add(&password_row);

    vbox.append(&smtp_group);

    // -- Advanced / Security expander --
    let security_expander = adw::ExpanderRow::builder()
        .title(gettextrs::gettext("Advanced"))
        .show_enable_switch(false)
        .build();

    let dnssec_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("DNSSEC"))
        .subtitle(gettextrs::gettext(
            "Require DNSSEC validation for DNS lookups",
        ))
        .build();
    security_expander.add_row(&dnssec_row);

    let dane_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("DANE"))
        .subtitle(gettextrs::gettext(
            "Require DANE (TLSA) verification for TLS",
        ))
        .build();
    security_expander.add_row(&dane_row);

    // Client certificate selector.
    let client_cert_path: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    let client_cert_label = gtk::Label::builder()
        .label(gettextrs::gettext("None"))
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
        .sensitive(false)
        .build();

    let client_cert_row = adw::ActionRow::builder()
        .title(gettextrs::gettext("Client certificate"))
        .build();
    client_cert_row.add_suffix(&cert_select_btn);
    client_cert_row.add_suffix(&cert_clear_btn);
    client_cert_row.add_suffix(&client_cert_label);
    security_expander.add_row(&client_cert_row);

    let realm_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Authentication realm"))
        .build();
    security_expander.add_row(&realm_row);

    // -- EHLO options (FR-52, FR-53) --
    let use_ip_ehlo_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Use IP address in EHLO"))
        .subtitle(gettextrs::gettext(
            "Send the device IP address in the SMTP greeting",
        ))
        .active(true)
        .build();
    security_expander.add_row(&use_ip_ehlo_row);

    let custom_ehlo_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Custom EHLO hostname"))
        .visible(false)
        .build();
    security_expander.add_row(&custom_ehlo_row);

    // Show/hide custom EHLO field based on toggle.
    use_ip_ehlo_row.connect_active_notify(clone!(
        #[weak]
        custom_ehlo_row,
        move |row| {
            custom_ehlo_row.set_visible(!row.is_active());
        }
    ));

    // -- Login before send (FR-54) --
    let login_before_send_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Login before send"))
        .subtitle(gettextrs::gettext(
            "Verify the inbound account is accessible before sending",
        ))
        .build();
    security_expander.add_row(&login_before_send_row);

    let security_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Security"))
        .build();
    security_group.add(&security_expander);
    vbox.append(&security_group);

    // -- Pre-fill from existing identity if editing --
    let editing_id: i64 = if let Some(ref row) = existing {
        email_row.set_text(&row.email_address);
        display_name_row.set_text(&row.display_name);
        host_row.set_text(&row.smtp_host);
        port_row.set_value(row.smtp_port as f64);
        encryption_row.set_selected(encryption_to_combo(encryption_from_str(
            &row.smtp_encryption,
        )));
        username_row.set_text(&row.smtp_username);
        if let Some(ref pw) = existing_password {
            password_row.set_text(pw);
        }
        if let Some(realm) = Some(&row.smtp_realm).filter(|r| !r.is_empty()) {
            realm_row.set_text(realm);
        }
        dnssec_row.set_active(row.smtp_dnssec);
        dane_row.set_active(row.smtp_dane);
        if let Some(ref cert) = row.smtp_client_certificate {
            let name = std::path::Path::new(cert)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(cert);
            client_cert_label.set_label(name);
            *client_cert_path.borrow_mut() = Some(cert.clone());
            cert_clear_btn.set_sensitive(true);
        }
        // EHLO options.
        use_ip_ehlo_row.set_active(row.use_ip_in_ehlo);
        custom_ehlo_row.set_visible(!row.use_ip_in_ehlo);
        if let Some(ref ehlo) = row.custom_ehlo {
            custom_ehlo_row.set_text(ehlo);
        }
        // Login before send.
        login_before_send_row.set_active(row.login_before_send);
        // Select the matching account in the dropdown.
        if let Some(idx) = accounts.iter().position(|a| a.id == row.account_id) {
            account_row.set_selected(idx as u32);
        }
        row.id
    } else {
        // Default: pre-fill username/password from first account's credentials.
        if let Some(first) = accounts.first() {
            username_row.set_text(&first.username);
            password_row.set_text(&first.password);
        }
        0
    };

    // -- Wire up: encryption change auto-fills port --
    encryption_row.connect_selected_notify(clone!(
        #[weak]
        port_row,
        move |row| {
            let encryption = combo_to_encryption(row.selected());
            let current_port = port_row.value() as u16;
            let current = if current_port == 0 {
                None
            } else {
                Some(current_port)
            };
            if should_autofill(current) {
                port_row.set_value(smtp_default_port(encryption) as f64);
            }
        }
    ));

    // -- Auto-config button handler (FR-49) --
    auto_config_btn.connect_clicked(clone!(
        #[weak]
        domain_row,
        #[weak]
        auto_config_btn,
        #[weak]
        auto_config_spinner,
        #[weak]
        host_row,
        #[weak]
        port_row,
        #[weak]
        encryption_row,
        #[weak]
        toast_overlay,
        move |_| {
            let domain_text = domain_row.text().to_string();
            if domain_text.trim().is_empty() {
                let toast = adw::Toast::new(&gettextrs::gettext("Please enter a domain"));
                toast_overlay.add_toast(toast);
                return;
            }

            // Disable domain field and button during operation.
            domain_row.set_sensitive(false);
            auto_config_btn.set_visible(false);
            auto_config_spinner.set_visible(true);
            auto_config_spinner.set_spinning(true);

            // Run auto-config asynchronously (simulated with short delay).
            glib::timeout_add_local_once(
                std::time::Duration::from_millis(200),
                clone!(
                    #[weak]
                    domain_row,
                    #[weak]
                    auto_config_btn,
                    #[weak]
                    auto_config_spinner,
                    #[weak]
                    host_row,
                    #[weak]
                    port_row,
                    #[weak]
                    encryption_row,
                    #[weak]
                    toast_overlay,
                    move || {
                        let provider_db = crate::core::provider::ProviderDatabase::bundled();

                        // Use mock implementations matching the rest of the dialog.
                        struct NoopResolver;
                        impl crate::core::dns_discovery::DnsResolver for NoopResolver {
                            fn lookup_ns(
                                &self,
                                _: &str,
                            ) -> Result<Vec<String>, crate::core::dns_discovery::DnsError>
                            {
                                Err(crate::core::dns_discovery::DnsError::NoRecords)
                            }
                            fn lookup_mx(
                                &self,
                                _: &str,
                            ) -> Result<Vec<(u16, String)>, crate::core::dns_discovery::DnsError>
                            {
                                Err(crate::core::dns_discovery::DnsError::NoRecords)
                            }
                            fn lookup_srv(
                                &self,
                                _: &str,
                            ) -> Result<
                                Vec<crate::core::dns_discovery::SrvRecord>,
                                crate::core::dns_discovery::DnsError,
                            > {
                                Err(crate::core::dns_discovery::DnsError::NoRecords)
                            }
                        }
                        struct NoopHttp;
                        impl crate::core::ispdb_discovery::HttpClient for NoopHttp {
                            fn get(
                                &self,
                                _: &str,
                            ) -> Result<String, crate::core::ispdb_discovery::AutoconfigError>
                            {
                                Err(crate::core::ispdb_discovery::AutoconfigError::HttpFailed(
                                    "not available".to_string(),
                                ))
                            }
                        }
                        struct NoopProber;
                        impl crate::core::port_scanning::PortProber for NoopProber {
                            fn probe(&self, _: &str, _: u16) -> bool {
                                false
                            }
                        }

                        let result = crate::core::auto_config::discover_outbound(
                            &domain_text,
                            &provider_db,
                            &NoopResolver,
                            &NoopHttp,
                            &NoopProber,
                        );

                        // Re-enable controls.
                        domain_row.set_sensitive(true);
                        auto_config_btn.set_visible(true);
                        auto_config_spinner.set_spinning(false);
                        auto_config_spinner.set_visible(false);

                        match result {
                            Ok(config) => {
                                host_row.set_text(&config.hostname);
                                port_row.set_value(config.port as f64);
                                encryption_row
                                    .set_selected(provider_encryption_to_combo(config.encryption));

                                let toast = adw::Toast::new(&gettextrs::gettext(
                                    "SMTP settings auto-detected",
                                ));
                                toast_overlay.add_toast(toast);
                            }
                            Err(_) => {
                                let toast = adw::Toast::new(&gettextrs::gettext(
                                    "Could not auto-detect SMTP settings for this domain",
                                ));
                                toast_overlay.add_toast(toast);
                            }
                        }
                    }
                ),
            );
        }
    ));

    // -- Wire up: associated account change defaults username/password --
    let accounts_for_change = accounts.clone();
    account_row.connect_selected_notify(clone!(
        #[weak]
        username_row,
        #[weak]
        password_row,
        move |row| {
            let idx = row.selected() as usize;
            if let Some(acct) = accounts_for_change.get(idx) {
                // Only pre-fill if current values are empty (don't overwrite user edits).
                if username_row.text().is_empty() {
                    username_row.set_text(&acct.username);
                }
                if password_row.text().is_empty() {
                    password_row.set_text(&acct.password);
                }
            }
        }
    ));

    // -- Wire up: client certificate select/clear --
    cert_select_btn.connect_clicked(clone!(
        #[strong]
        client_cert_path,
        #[weak]
        client_cert_label,
        #[weak]
        cert_clear_btn,
        move |btn| {
            let file_dialog = gtk::FileDialog::builder()
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
            file_dialog.set_filters(Some(&filters));

            let window = btn.root().and_then(|r| r.downcast::<gtk::Window>().ok());
            let path_ref = client_cert_path.clone();
            let label_ref = client_cert_label.clone();
            let clear_ref = cert_clear_btn.clone();
            file_dialog.open(
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
                        }
                    }
                },
            );
        }
    ));

    cert_clear_btn.connect_clicked(clone!(
        #[strong]
        client_cert_path,
        #[weak]
        client_cert_label,
        move |btn| {
            *client_cert_path.borrow_mut() = None;
            client_cert_label.set_label(&gettextrs::gettext("None"));
            btn.set_sensitive(false);
        }
    ));

    // -- Test Connection --
    let test_group = adw::PreferencesGroup::new();

    let test_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Test Connection"))
        .css_classes(["pill"])
        .build();

    let test_spinner = gtk::Spinner::new();
    test_spinner.set_visible(false);

    let test_btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Center)
        .build();
    test_btn_box.append(&test_btn);
    test_btn_box.append(&test_spinner);

    test_group.add(&test_btn_box);
    vbox.append(&test_group);

    // -- Test Results (hidden initially) --
    let test_results_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Test Results"))
        .visible(false)
        .build();
    let test_results_label = gtk::Label::builder()
        .wrap(true)
        .selectable(true)
        .xalign(0.0)
        .build();
    test_results_group.add(&test_results_label);

    let store_size_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Store as size limit"))
        .css_classes(["pill"])
        .visible(false)
        .build();
    test_results_group.add(&store_size_btn);

    vbox.append(&test_results_group);

    // Shared state for detected max message size.
    let detected_max_size: Rc<RefCell<Option<u64>>> = Rc::new(RefCell::new(None));
    let stored_max_size: Rc<RefCell<Option<u64>>> = Rc::new(RefCell::new(None));

    // -- Action buttons --
    let btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .build();

    let cancel_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Cancel"))
        .build();

    let save_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Save"))
        .css_classes(["suggested-action"])
        .build();

    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);
    vbox.append(&btn_box);

    clamp.set_child(Some(&vbox));
    scrolled.set_child(Some(&clamp));
    toast_overlay.set_child(Some(&scrolled));
    toolbar_view.set_content(Some(&toast_overlay));
    dialog.set_child(Some(&toolbar_view));

    // -- "Store as size limit" button handler --
    store_size_btn.connect_clicked(clone!(
        #[strong]
        detected_max_size,
        #[strong]
        stored_max_size,
        #[weak]
        store_size_btn,
        #[weak]
        toast_overlay,
        move |_| {
            if let Some(size) = *detected_max_size.borrow() {
                *stored_max_size.borrow_mut() = Some(size);
                store_size_btn.set_visible(false);
                let size_mb = size / (1024 * 1024);
                toast_overlay.add_toast(adw::Toast::new(&format!(
                    "{} {} MiB",
                    gettextrs::gettext("Size limit stored:"),
                    size_mb
                )));
            }
        }
    ));

    // -- Test Connection button handler --
    test_btn.connect_clicked(clone!(
        #[strong]
        detected_max_size,
        #[weak]
        host_row,
        #[weak]
        port_row,
        #[weak]
        encryption_row,
        #[weak]
        username_row,
        #[weak]
        password_row,
        #[weak]
        email_row,
        #[weak]
        toast_overlay,
        #[weak]
        test_btn,
        #[weak]
        save_btn,
        #[weak]
        cancel_btn,
        #[weak]
        test_spinner,
        #[weak]
        test_results_group,
        #[weak]
        test_results_label,
        #[weak]
        store_size_btn,
        #[weak]
        display_name_row,
        #[weak]
        account_row,
        #[weak]
        domain_row,
        #[weak]
        auto_config_btn,
        #[weak]
        realm_row,
        #[weak]
        dane_row,
        #[weak]
        dnssec_row,
        #[weak]
        use_ip_ehlo_row,
        #[weak]
        custom_ehlo_row,
        #[weak]
        login_before_send_row,
        #[strong]
        client_cert_path,
        move |_| {
            let host_val = host_row.text().to_string();
            let user_val = username_row.text().to_string();
            let pass_val = password_row.text().to_string();
            let email_val = email_row.text().to_string();
            let has_cert = client_cert_path.borrow().is_some();

            // Validate before testing.
            let validation =
                validate_smtp_identity(&host_val, &user_val, &pass_val, &email_val, has_cert);
            if !validation.is_valid() {
                let msg = validation
                    .errors
                    .iter()
                    .map(|e| e.message())
                    .collect::<Vec<_>>()
                    .join("; ");
                toast_overlay.add_toast(adw::Toast::new(&msg));
                return;
            }

            // Disable all input fields during test.
            set_smtp_form_sensitive(
                &email_row,
                &display_name_row,
                &account_row,
                &domain_row,
                &auto_config_btn,
                &host_row,
                &port_row,
                &encryption_row,
                &username_row,
                &password_row,
                &realm_row,
                &dane_row,
                &dnssec_row,
                &use_ip_ehlo_row,
                &custom_ehlo_row,
                &login_before_send_row,
                &test_btn,
                &save_btn,
                &cancel_btn,
                false,
            );

            // Show spinner.
            test_spinner.set_visible(true);
            test_spinner.set_spinning(true);
            test_results_group.set_visible(false);
            store_size_btn.set_visible(false);

            let encryption = combo_to_encryption(encryption_row.selected());

            // Run the test with a short delay to let the UI update.
            glib::timeout_add_local_once(
                std::time::Duration::from_millis(200),
                clone!(
                    #[strong]
                    detected_max_size,
                    #[weak]
                    host_row,
                    #[weak]
                    port_row,
                    #[weak]
                    encryption_row,
                    #[weak]
                    username_row,
                    #[weak]
                    password_row,
                    #[weak]
                    email_row,
                    #[weak]
                    toast_overlay,
                    #[weak]
                    test_btn,
                    #[weak]
                    save_btn,
                    #[weak]
                    cancel_btn,
                    #[weak]
                    test_spinner,
                    #[weak]
                    test_results_group,
                    #[weak]
                    test_results_label,
                    #[weak]
                    store_size_btn,
                    #[weak]
                    display_name_row,
                    #[weak]
                    account_row,
                    #[weak]
                    domain_row,
                    #[weak]
                    auto_config_btn,
                    #[weak]
                    realm_row,
                    #[weak]
                    dane_row,
                    #[weak]
                    dnssec_row,
                    #[weak]
                    use_ip_ehlo_row,
                    #[weak]
                    custom_ehlo_row,
                    #[weak]
                    login_before_send_row,
                    move || {
                        let provider = build_smtp_provider(
                            host_val.trim(),
                            port_row.value() as u16,
                            encryption,
                        );
                        let checker = MockSmtpChecker;
                        let result = checker.check_smtp(&user_val, &pass_val, &provider, None);

                        // Stop spinner.
                        test_spinner.set_spinning(false);
                        test_spinner.set_visible(false);

                        // Re-enable form.
                        set_smtp_form_sensitive(
                            &email_row,
                            &display_name_row,
                            &account_row,
                            &domain_row,
                            &auto_config_btn,
                            &host_row,
                            &port_row,
                            &encryption_row,
                            &username_row,
                            &password_row,
                            &realm_row,
                            &dane_row,
                            &dnssec_row,
                            &use_ip_ehlo_row,
                            &custom_ehlo_row,
                            &login_before_send_row,
                            &test_btn,
                            &save_btn,
                            &cancel_btn,
                            true,
                        );

                        let mut text = String::new();

                        match result {
                            Ok(success) => {
                                text.push_str(&gettextrs::gettext("SMTP: OK"));
                                text.push_str(&format!(
                                    "\n{} {}",
                                    gettextrs::gettext("Authenticated as:"),
                                    success.authenticated_username
                                ));
                                if let Some(size) = success.max_message_size {
                                    let size_mb = size / (1024 * 1024);
                                    text.push_str(&format!(
                                        "\n{} {} MiB ({} bytes)",
                                        gettextrs::gettext("Max message size:"),
                                        size_mb,
                                        size
                                    ));
                                    *detected_max_size.borrow_mut() = Some(size);
                                    store_size_btn.set_visible(true);
                                } else {
                                    *detected_max_size.borrow_mut() = None;
                                    text.push_str(&format!(
                                        "\n{}",
                                        gettextrs::gettext(
                                            "Server did not advertise a maximum message size"
                                        )
                                    ));
                                }

                                toast_overlay.add_toast(adw::Toast::new(&gettextrs::gettext(
                                    "Connection successful",
                                )));
                            }
                            Err(ref e) => {
                                let provider_db =
                                    crate::core::provider::ProviderDatabase::new(vec![]);
                                let diag =
                                    diagnose_smtp_error(e, Some(host_val.trim()), &provider_db);
                                text.push_str(&diag.display_text());

                                if let SmtpCheckError::UntrustedCertificate(ref info) = e {
                                    text.push_str("\n\n");
                                    text.push_str(&gettextrs::gettext("Certificate fingerprint:"));
                                    text.push('\n');
                                    text.push_str(&info.fingerprint);
                                }

                                toast_overlay.add_toast(adw::Toast::new(&gettextrs::gettext(
                                    "Connection test completed with errors",
                                )));
                            }
                        }

                        test_results_label.set_text(&text);
                        test_results_group.set_visible(true);
                    }
                ),
            );
        }
    ));

    // -- Cancel --
    let on_done = Rc::new(on_done);
    let on_done_cancel = on_done.clone();
    cancel_btn.connect_clicked(clone!(
        #[weak]
        dialog,
        move |_| {
            on_done_cancel(None);
            dialog.close();
        }
    ));

    let on_done_close = on_done.clone();
    dialog.connect_closed(move |_| {
        on_done_close(None);
    });

    // -- Save --
    let accounts_for_save = accounts;
    save_btn.connect_clicked(clone!(
        #[strong]
        stored_max_size,
        #[weak]
        dialog,
        #[weak]
        host_row,
        #[weak]
        port_row,
        #[weak]
        encryption_row,
        #[weak]
        username_row,
        #[weak]
        password_row,
        #[weak]
        email_row,
        #[weak]
        display_name_row,
        #[weak]
        account_row,
        #[weak]
        realm_row,
        #[weak]
        dane_row,
        #[weak]
        dnssec_row,
        #[weak]
        use_ip_ehlo_row,
        #[weak]
        custom_ehlo_row,
        #[weak]
        login_before_send_row,
        #[weak]
        toast_overlay,
        #[strong]
        client_cert_path,
        move |_| {
            let host_val = host_row.text().to_string();
            let user_val = username_row.text().to_string();
            let pass_val = password_row.text().to_string();
            let email_val = email_row.text().to_string();
            let has_cert = client_cert_path.borrow().is_some();

            let validation =
                validate_smtp_identity(&host_val, &user_val, &pass_val, &email_val, has_cert);

            if !validation.is_valid() {
                let msg = validation
                    .errors
                    .iter()
                    .map(|e| e.message())
                    .collect::<Vec<_>>()
                    .join("; ");
                toast_overlay.add_toast(adw::Toast::new(&msg));
                return;
            }

            // Show password warnings as non-blocking toasts.
            for w in &validation.password_warnings {
                toast_overlay.add_toast(adw::Toast::new(w.message()));
            }

            let selected_account_idx = account_row.selected() as usize;
            let account_id = accounts_for_save
                .get(selected_account_idx)
                .map(|a| a.id.clone())
                .unwrap_or_default();

            let encryption = combo_to_encryption(encryption_row.selected());

            let identity = IdentityRow {
                id: editing_id,
                account_id,
                email_address: email_val.trim().to_string(),
                display_name: display_name_row.text().trim().to_string(),
                smtp_host: host_val.trim().to_string(),
                smtp_port: port_row.value() as u16,
                smtp_encryption: encryption_to_str(encryption).to_string(),
                smtp_username: user_val.trim().to_string(),
                smtp_realm: realm_row.text().trim().to_string(),
                use_ip_in_ehlo: use_ip_ehlo_row.is_active(),
                custom_ehlo: {
                    let val = custom_ehlo_row.text().trim().to_string();
                    if val.is_empty() {
                        None
                    } else {
                        Some(val)
                    }
                },
                login_before_send: login_before_send_row.is_active(),
                max_message_size_cache: *stored_max_size.borrow(),
                smtp_client_certificate: client_cert_path.borrow().clone(),
                smtp_dane: dane_row.is_active(),
                smtp_dnssec: dnssec_row.is_active(),
            };

            on_done(Some(SmtpIdentityDialogResult {
                identity,
                password: pass_val,
            }));
            dialog.close();
        }
    ));

    dialog.present(Some(parent));
}

#[allow(clippy::too_many_arguments)]
fn set_smtp_form_sensitive(
    email_row: &adw::EntryRow,
    display_name_row: &adw::EntryRow,
    account_row: &adw::ComboRow,
    domain_row: &adw::EntryRow,
    auto_config_btn: &gtk::Button,
    host_row: &adw::EntryRow,
    port_row: &adw::SpinRow,
    encryption_row: &adw::ComboRow,
    username_row: &adw::EntryRow,
    password_row: &adw::PasswordEntryRow,
    realm_row: &adw::EntryRow,
    dane_row: &adw::SwitchRow,
    dnssec_row: &adw::SwitchRow,
    use_ip_ehlo_row: &adw::SwitchRow,
    custom_ehlo_row: &adw::EntryRow,
    login_before_send_row: &adw::SwitchRow,
    test_btn: &gtk::Button,
    save_btn: &gtk::Button,
    cancel_btn: &gtk::Button,
    sensitive: bool,
) {
    email_row.set_sensitive(sensitive);
    display_name_row.set_sensitive(sensitive);
    account_row.set_sensitive(sensitive);
    domain_row.set_sensitive(sensitive);
    auto_config_btn.set_sensitive(sensitive);
    host_row.set_sensitive(sensitive);
    port_row.set_sensitive(sensitive);
    encryption_row.set_sensitive(sensitive);
    username_row.set_sensitive(sensitive);
    password_row.set_sensitive(sensitive);
    realm_row.set_sensitive(sensitive);
    dane_row.set_sensitive(sensitive);
    dnssec_row.set_sensitive(sensitive);
    use_ip_ehlo_row.set_sensitive(sensitive);
    custom_ehlo_row.set_sensitive(sensitive);
    login_before_send_row.set_sensitive(sensitive);
    test_btn.set_sensitive(sensitive);
    save_btn.set_sensitive(sensitive);
    cancel_btn.set_sensitive(sensitive);
}

fn encryption_to_provider(enc: EncryptionMode) -> ProviderEncryption {
    match enc {
        EncryptionMode::SslTls => ProviderEncryption::SslTls,
        EncryptionMode::StartTls => ProviderEncryption::StartTls,
        EncryptionMode::None => ProviderEncryption::None,
    }
}

fn build_smtp_provider(host: &str, port: u16, encryption: EncryptionMode) -> Provider {
    Provider {
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
