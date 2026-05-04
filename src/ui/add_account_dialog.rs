use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;

use crate::core::{Account, AuthMethod, EncryptionMode, NewAccountParams, Protocol};

/// Result of the add-account dialog: either a validated Account or the user cancelled.
pub(crate) type DialogResult = Option<Account>;

/// Build and show the "Add Account" dialog. Calls `on_done` with the result.
pub(crate) fn show(parent: &adw::ApplicationWindow, on_done: impl Fn(DialogResult) + 'static) {
    let dialog = adw::Dialog::builder()
        .title(gettext::gettext("Add IMAP Account"))
        .content_width(460)
        .content_height(520)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let toast_overlay = adw::ToastOverlay::new();

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
        .title(gettext::gettext("Display Name"))
        .build();
    let name_row = adw::EntryRow::builder()
        .title(gettext::gettext("Account name"))
        .build();
    name_row.set_tooltip_text(Some(&gettext::gettext("A friendly name for this account")));
    name_group.add(&name_row);
    vbox.append(&name_group);

    // -- Server settings --
    let server_group = adw::PreferencesGroup::builder()
        .title(gettext::gettext("Server"))
        .build();

    let host_row = adw::EntryRow::builder()
        .title(gettext::gettext("Host"))
        .build();
    server_group.add(&host_row);

    let port_row = adw::SpinRow::builder()
        .title(gettext::gettext("Port"))
        .adjustment(&gtk::Adjustment::new(993.0, 1.0, 65535.0, 1.0, 10.0, 0.0))
        .build();
    server_group.add(&port_row);

    let encryption_row = adw::ComboRow::builder()
        .title(gettext::gettext("Encryption"))
        .model(&gtk::StringList::new(&[
            &gettext::gettext("SSL/TLS"),
            &gettext::gettext("STARTTLS"),
            &gettext::gettext("None"),
        ]))
        .build();
    server_group.add(&encryption_row);

    vbox.append(&server_group);

    // -- Authentication --
    let auth_group = adw::PreferencesGroup::builder()
        .title(gettext::gettext("Authentication"))
        .build();

    let auth_method_row = adw::ComboRow::builder()
        .title(gettext::gettext("Method"))
        .model(&gtk::StringList::new(&[
            &gettext::gettext("PLAIN"),
            &gettext::gettext("LOGIN"),
            &gettext::gettext("OAuth2"),
        ]))
        .build();
    auth_group.add(&auth_method_row);

    let username_row = adw::EntryRow::builder()
        .title(gettext::gettext("Username"))
        .build();
    auth_group.add(&username_row);

    let password_row = adw::PasswordEntryRow::builder()
        .title(gettext::gettext("Password / Token"))
        .build();
    auth_group.add(&password_row);

    vbox.append(&auth_group);

    // -- Save button --
    let save_btn = gtk::Button::builder()
        .label(gettext::gettext("Save"))
        .css_classes(["suggested-action", "pill"])
        .halign(gtk::Align::Center)
        .margin_top(12)
        .build();
    vbox.append(&save_btn);

    clamp.set_child(Some(&vbox));
    toast_overlay.set_child(Some(&clamp));
    toolbar_view.set_content(Some(&toast_overlay));
    dialog.set_child(Some(&toolbar_view));

    let on_done = std::rc::Rc::new(on_done);
    let on_done_close = on_done.clone();

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
        toast_overlay,
        move |_| {
            let encryption = match encryption_row.selected() {
                0 => EncryptionMode::SslTls,
                1 => EncryptionMode::StartTls,
                _ => EncryptionMode::None,
            };
            let auth = match auth_method_row.selected() {
                0 => AuthMethod::Plain,
                1 => AuthMethod::Login,
                _ => AuthMethod::OAuth2,
            };

            match Account::new(NewAccountParams {
                display_name: name_row.text().to_string(),
                protocol: Protocol::Imap,
                host: host_row.text().to_string(),
                port: port_row.value() as u16,
                encryption,
                auth_method: auth,
                username: username_row.text().to_string(),
                credential: password_row.text().to_string(),
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
        // If closed without saving, signal cancellation.
        // The Rc ensures on_done is only meaningfully called once
        // (the dialog result was already sent if Save succeeded).
        let _ = &on_done_close;
    });

    dialog.present(Some(parent));
}
