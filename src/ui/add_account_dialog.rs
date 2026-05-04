use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::core::connection_test::{ConnectionTestRequest, ServerConnectionParams};
use crate::core::{Account, AuthMethod, EncryptionMode, NewAccountParams, Protocol, SmtpConfig};
use crate::services::connection_tester::{ConnectionTester, MockConnectionTester};

/// Result of the add-account dialog: either a validated Account or the user cancelled.
pub(crate) type DialogResult = Option<Account>;

/// Build and show the "Add Account" dialog. Calls `on_done` with the result.
pub(crate) fn show(parent: &adw::ApplicationWindow, on_done: impl Fn(DialogResult) + 'static) {
    let dialog = adw::Dialog::builder()
        .title(gettextrs::gettext("Add Account"))
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
    name_group.add(&name_row);
    vbox.append(&name_group);

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
        .build();
    server_group.add(&protocol_row);

    let host_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Host"))
        .build();
    server_group.add(&host_row);

    let port_row = adw::SpinRow::builder()
        .title(gettextrs::gettext("Port"))
        .adjustment(&gtk::Adjustment::new(993.0, 1.0, 65535.0, 1.0, 10.0, 0.0))
        .build();
    server_group.add(&port_row);

    let encryption_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Encryption"))
        .model(&gtk::StringList::new(&[
            &gettextrs::gettext("SSL/TLS"),
            &gettextrs::gettext("STARTTLS"),
            &gettextrs::gettext("None"),
        ]))
        .build();
    server_group.add(&encryption_row);

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
        .build();
    auth_group.add(&auth_method_row);

    let username_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Username"))
        .build();
    auth_group.add(&username_row);

    let password_row = adw::PasswordEntryRow::builder()
        .title(gettextrs::gettext("Password / Token"))
        .build();
    auth_group.add(&password_row);

    vbox.append(&auth_group);

    // -- Outgoing (SMTP) server settings --
    let smtp_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Outgoing Server (SMTP)"))
        .build();

    let smtp_host_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Host"))
        .build();
    smtp_group.add(&smtp_host_row);

    let smtp_port_row = adw::SpinRow::builder()
        .title(gettextrs::gettext("Port"))
        .adjustment(&gtk::Adjustment::new(587.0, 1.0, 65535.0, 1.0, 10.0, 0.0))
        .build();
    smtp_group.add(&smtp_port_row);

    let smtp_encryption_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Encryption"))
        .model(&gtk::StringList::new(&[
            &gettextrs::gettext("SSL/TLS"),
            &gettextrs::gettext("STARTTLS"),
            &gettextrs::gettext("None"),
        ]))
        .selected(1) // STARTTLS is common for SMTP port 587
        .build();
    smtp_group.add(&smtp_encryption_row);

    let smtp_auth_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Method"))
        .model(&gtk::StringList::new(&[
            &gettextrs::gettext("PLAIN"),
            &gettextrs::gettext("LOGIN"),
            &gettextrs::gettext("OAuth2"),
        ]))
        .build();
    smtp_group.add(&smtp_auth_row);

    let smtp_username_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Username"))
        .build();
    smtp_group.add(&smtp_username_row);

    let smtp_password_row = adw::PasswordEntryRow::builder()
        .title(gettextrs::gettext("Password / Token"))
        .build();
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

    vbox.append(&btn_box);

    clamp.set_child(Some(&vbox));
    scrolled.set_child(Some(&clamp));
    toast_overlay.set_child(Some(&scrolled));
    toolbar_view.set_content(Some(&toast_overlay));
    dialog.set_child(Some(&toolbar_view));

    let on_done = std::rc::Rc::new(on_done);
    let on_done_close = on_done.clone();

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

            match Account::new(NewAccountParams {
                display_name: name_row.text().to_string(),
                protocol,
                host: host_row.text().to_string(),
                port: port_row.value() as u16,
                encryption,
                auth_method: auth,
                username: username_row.text().to_string(),
                credential: password_row.text().to_string(),
                smtp,
            }) {
                Ok(account) => {
                    on_done(Some(account));
                    dialog.close();
                }
                Err(e) => {
                    let toast = adw::Toast::new(&e.to_string());
                    toast_overlay.add_toast(toast);
                }
            }
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
