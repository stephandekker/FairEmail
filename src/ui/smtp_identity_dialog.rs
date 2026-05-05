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
use crate::core::smtp_identity::validate_smtp_identity;
use crate::services::identity_store::IdentityRow;

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
                use_ip_in_ehlo: false,
                custom_ehlo: None,
                login_before_send: false,
                max_message_size_cache: None,
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
