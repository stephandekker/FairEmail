use std::cell::RefCell;
use std::rc::Rc;

use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::core::connection_test::{ConnectionTestRequest, ServerConnectionParams};
use crate::core::{
    Account, AccountColor, AuthMethod, EncryptionMode, NewAccountParams, Pop3Settings, Protocol,
    SmtpConfig, SwipeAction, SwipeDefaults, SystemFolders,
};
use crate::services::connection_tester::{ConnectionTester, MockConnectionTester};

/// Result of the add-account dialog: either a validated Account or the user cancelled.
pub(crate) type DialogResult = Option<Account>;

/// Pre-fill data carried over from the quick-setup wizard (FR-36).
#[derive(Debug, Clone, Default)]
pub(crate) struct PrefillData {
    pub display_name: String,
    pub email: String,
    pub password: String,
}

/// Build and show the "Add Account" dialog with optional pre-filled data from the wizard.
/// `existing_categories` provides autocomplete suggestions for the category field (FR-23).
pub(crate) fn show_with_prefill(
    parent: &adw::ApplicationWindow,
    existing_categories: Vec<String>,
    prefill: PrefillData,
    on_done: impl Fn(DialogResult) + 'static,
) {
    show_inner(parent, existing_categories, Some(prefill), on_done);
}

/// Build and show the "Add Account" dialog. Calls `on_done` with the result.
/// `existing_categories` provides autocomplete suggestions for the category field (FR-23).
pub(crate) fn show(
    parent: &adw::ApplicationWindow,
    existing_categories: Vec<String>,
    on_done: impl Fn(DialogResult) + 'static,
) {
    show_inner(parent, existing_categories, None, on_done);
}

fn show_inner(
    parent: &adw::ApplicationWindow,
    existing_categories: Vec<String>,
    prefill: Option<PrefillData>,
    on_done: impl Fn(DialogResult) + 'static,
) {
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

    let color_active = Rc::new(RefCell::new(false));

    let clear_color_btn = gtk::Button::builder()
        .icon_name("edit-clear-symbolic")
        .tooltip_text(gettextrs::gettext("Clear account colour"))
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .sensitive(false)
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
        .icon_name("avatar-default-symbolic")
        .pixel_size(32)
        .valign(gtk::Align::Center)
        .build();
    avatar_image.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Account avatar preview",
    ))]);

    let avatar_path: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

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
        .sensitive(false)
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

        // Show/hide and filter suggestions as the user types.
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

        // Clicking a suggestion fills the entry.
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

    // -- FR-16: prominent warning when "None" encryption is selected --
    let no_encryption_banner = adw::Banner::builder()
        .title(gettextrs::gettext(
            "⚠ No encryption: credentials and messages will be sent in plain text. This is insecure and not recommended.",
        ))
        .revealed(false)
        .build();
    no_encryption_banner.add_css_class("error");
    vbox.append(&no_encryption_banner);

    encryption_row.connect_selected_notify(clone!(
        #[weak]
        no_encryption_banner,
        move |row| {
            // Index 2 = "None"
            no_encryption_banner.set_revealed(row.selected() == 2);
        }
    ));

    // -- POP3 limitations banner (US-35, FR-10) --
    let pop3_banner = adw::Banner::builder()
        .title(gettextrs::gettext(
            "POP3 limitations: no server-side folders, no server-side search, no remote flag sync. Sent, Drafts, and Trash are local-only.",
        ))
        .revealed(false)
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

    // -- POP3-specific settings (US-31, US-32, US-33, US-34, FR-9) --
    let pop3_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("POP3 Settings"))
        .visible(false)
        .build();

    let leave_on_server_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext("Leave messages on server"))
        .active(true)
        .build();
    pop3_group.add(&leave_on_server_row);

    let delete_from_server_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext(
            "Delete from server when deleted on device",
        ))
        .active(false)
        .build();
    pop3_group.add(&delete_from_server_row);

    let keep_on_device_row = adw::SwitchRow::builder()
        .title(gettextrs::gettext(
            "Keep on device when deleted from server",
        ))
        .active(true)
        .build();
    pop3_group.add(&keep_on_device_row);

    let max_messages_row = adw::SpinRow::builder()
        .title(gettextrs::gettext(
            "Maximum messages to download (0 = unlimited)",
        ))
        .adjustment(&gtk::Adjustment::new(0.0, 0.0, 100_000.0, 1.0, 10.0, 0.0))
        .build();
    pop3_group.add(&max_messages_row);

    vbox.append(&pop3_group);

    // -- IMAP system folder designation (FR-35, FR-36, US-36) --
    let system_folders_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("System Folders"))
        .description(gettextrs::gettext(
            "Designate which server folder serves each role",
        ))
        .visible(true) // IMAP is default protocol
        .build();

    let drafts_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Drafts"))
        .build();
    system_folders_group.add(&drafts_row);

    let sent_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Sent"))
        .build();
    system_folders_group.add(&sent_row);

    let archive_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Archive"))
        .build();
    system_folders_group.add(&archive_row);

    let trash_folder_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Trash"))
        .build();
    system_folders_group.add(&trash_folder_row);

    let junk_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Junk"))
        .build();
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

    let swipe_left_row = adw::ComboRow::builder()
        .title(gettextrs::gettext("Swipe left action"))
        .model(&swipe_action_string_list())
        .selected(0)
        .build();
    swipe_group.add(&swipe_left_row);

    let swipe_left_folder_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Swipe left folder"))
        .visible(false)
        .build();
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
        .selected(0)
        .build();
    swipe_group.add(&swipe_right_row);

    let swipe_right_folder_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Swipe right folder"))
        .visible(false)
        .build();
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
    swipe_group.add(&default_move_to_row);

    vbox.append(&swipe_group);

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

            // FR-58: default display name to username/email if left blank.
            let display_name = {
                let name = name_row.text().to_string();
                if name.trim().is_empty() {
                    username_row.text().to_string()
                } else {
                    name
                }
            };

            match Account::new(NewAccountParams {
                display_name,
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
                sync_enabled: true,
                on_demand: false,
                polling_interval_minutes: None,
                unmetered_only: false,
                vpn_only: false,
                schedule_exempt: false,
                system_folders,
                swipe_defaults,
                notifications_enabled: true,
                security_settings: None,
                fetch_settings: None,
                keep_alive_settings: None,
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

    // Pre-fill fields from wizard data (FR-36).
    if let Some(data) = prefill {
        if !data.display_name.is_empty() {
            name_row.set_text(&data.display_name);
        }
        if !data.email.is_empty() {
            username_row.set_text(&data.email);
        }
        if !data.password.is_empty() {
            password_row.set_text(&data.password);
        }
    }

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
