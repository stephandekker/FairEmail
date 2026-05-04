use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::core::connection_test::{ConnectionTestRequest, ServerConnectionParams};
use crate::core::{Account, AuthMethod, EncryptionMode, Protocol, SmtpConfig, UpdateAccountParams};
use crate::services::connection_tester::{ConnectionTester, MockConnectionTester};

/// Result of the edit-account dialog: the updated Account, or `None` if cancelled.
pub(crate) type EditDialogResult = Option<Account>;

/// Build and show the "Edit Account" dialog pre-populated with the given account's values.
/// Calls `on_done` with the updated account on save, or `None` on cancel/close.
pub(crate) fn show(
    parent: &adw::ApplicationWindow,
    account: Account,
    on_done: impl Fn(EditDialogResult) + 'static,
) {
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

    vbox.append(&btn_box);

    clamp.set_child(Some(&vbox));
    scrolled.set_child(Some(&clamp));
    toast_overlay.set_child(Some(&scrolled));
    toolbar_view.set_content(Some(&toast_overlay));
    dialog.set_child(Some(&toolbar_view));

    let on_done = std::rc::Rc::new(on_done);
    let on_done_close = on_done.clone();
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
            };

            let mut acct = account.borrow_mut();
            match acct.update(params) {
                Ok(()) => {
                    on_done(Some(acct.clone()));
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
