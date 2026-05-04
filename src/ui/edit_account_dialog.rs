use std::cell::RefCell;
use std::rc::Rc;

use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::core::connection_test::{ConnectionTestRequest, ServerConnectionParams};
use crate::core::{
    Account, AccountColor, AuthMethod, EncryptionMode, Pop3Settings, Protocol, SmtpConfig,
    UpdateAccountParams,
};
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

    // Show/hide POP3 settings when protocol changes.
    protocol_row.connect_selected_notify(clone!(
        #[weak]
        pop3_group,
        move |row| {
            pop3_group.set_visible(row.selected() == 1);
        }
    ));

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

            let color = if *color_active.borrow() {
                let rgba = color_btn.rgba();
                Some(AccountColor::new(rgba.red(), rgba.green(), rgba.blue()))
            } else {
                None
            };

            let avatar = avatar_path.borrow().clone();

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
                sync_enabled: account.borrow().sync_enabled(),
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
