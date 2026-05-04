use std::rc::Rc;

use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::core::export_accounts::{ExportCategory, ExportEnvelope};
use crate::core::import_accounts::{DuplicateStrategy, ImportOptions};
use crate::core::Account;
use crate::services::import_service;

/// Show a file chooser, then the import options dialog (FR-49, FR-50, AC-15).
///
/// The callback receives the `ImportResult` outcomes summary on success,
/// or `None` on cancellation.
pub(crate) fn show(
    parent: &adw::ApplicationWindow,
    accounts: Vec<Account>,
    callback: impl Fn(Option<Vec<Account>>) + 'static,
) {
    let file_dialog = gtk::FileDialog::builder()
        .title(gettextrs::gettext("Import account configurations"))
        .build();

    let parent_clone = parent.clone();
    let callback = Rc::new(callback);

    file_dialog.open(
        Some(parent),
        None::<&gtk4::gio::Cancellable>,
        clone!(
            #[strong]
            parent_clone,
            #[strong]
            callback,
            move |result| {
                match result {
                    Ok(file) => {
                        if let Some(path) = file.path() {
                            handle_file_selected(
                                &parent_clone,
                                accounts.clone(),
                                path,
                                callback.clone(),
                            );
                        } else {
                            callback(None);
                        }
                    }
                    Err(_) => {
                        callback(None);
                    }
                }
            }
        ),
    );
}

/// After the user selects a file, check if it's encrypted and show the appropriate dialog.
fn handle_file_selected(
    parent: &adw::ApplicationWindow,
    accounts: Vec<Account>,
    path: std::path::PathBuf,
    callback: Rc<dyn Fn(Option<Vec<Account>>)>,
) {
    match import_service::is_file_encrypted(&path) {
        Ok(true) => {
            show_password_dialog(parent, accounts, path, callback);
        }
        Ok(false) => match import_service::read_import_file(&path, None) {
            Ok(envelope) => {
                show_import_options_dialog(parent, accounts, envelope, path, None, callback);
            }
            Err(e) => {
                let msg = gettextrs::gettext(format!("Failed to read import file: {e}"));
                let toast = adw::Toast::new(&msg);
                toast.set_timeout(5);
                show_toast(parent, &toast);
                callback(None);
            }
        },
        Err(e) => {
            let msg = gettextrs::gettext(format!("Failed to read import file: {e}"));
            let toast = adw::Toast::new(&msg);
            toast.set_timeout(5);
            show_toast(parent, &toast);
            callback(None);
        }
    }
}

/// Prompt for password for an encrypted import file.
fn show_password_dialog(
    parent: &adw::ApplicationWindow,
    accounts: Vec<Account>,
    path: std::path::PathBuf,
    callback: Rc<dyn Fn(Option<Vec<Account>>)>,
) {
    let dialog = adw::AlertDialog::builder()
        .heading(gettextrs::gettext("Password Required"))
        .body(gettextrs::gettext(
            "This file is password-protected. Enter the password to decrypt it.",
        ))
        .build();

    dialog.add_response("cancel", &gettextrs::gettext("Cancel"));
    dialog.add_response("decrypt", &gettextrs::gettext("Decrypt"));
    dialog.set_response_appearance("decrypt", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("decrypt"));
    dialog.set_close_response("cancel");

    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let password_entry = gtk::PasswordEntry::builder()
        .placeholder_text(gettextrs::gettext("Password"))
        .show_peek_icon(true)
        .build();
    content_box.append(&password_entry);

    dialog.set_extra_child(Some(&content_box));

    let parent_clone = parent.clone();
    dialog.connect_response(
        None,
        clone!(
            #[strong]
            parent_clone,
            move |_dialog, response| {
                if response != "decrypt" {
                    callback(None);
                    return;
                }

                let password = password_entry.text().to_string();
                if password.is_empty() {
                    let msg = gettextrs::gettext("Password cannot be empty");
                    let toast = adw::Toast::new(&msg);
                    toast.set_timeout(3);
                    show_toast(&parent_clone, &toast);
                    callback(None);
                    return;
                }

                match import_service::read_import_file(&path, Some(&password)) {
                    Ok(envelope) => {
                        show_import_options_dialog(
                            &parent_clone,
                            accounts.clone(),
                            envelope,
                            path.clone(),
                            Some(password),
                            callback.clone(),
                        );
                    }
                    Err(e) => {
                        let msg = gettextrs::gettext(format!("Decryption failed: {e}"));
                        let toast = adw::Toast::new(&msg);
                        toast.set_timeout(5);
                        show_toast(&parent_clone, &toast);
                        callback(None);
                    }
                }
            }
        ),
    );

    dialog.present(Some(parent));
}

/// Show the import options dialog with account selection, category selection,
/// and duplicate handling (FR-49, FR-50, US-45).
fn show_import_options_dialog(
    parent: &adw::ApplicationWindow,
    existing_accounts: Vec<Account>,
    envelope: ExportEnvelope,
    path: std::path::PathBuf,
    password: Option<String>,
    callback: Rc<dyn Fn(Option<Vec<Account>>)>,
) {
    let dialog = adw::AlertDialog::builder()
        .heading(gettextrs::gettext("Import Account Configurations"))
        .body(gettextrs::gettext(
            "Select which accounts and data to import.",
        ))
        .build();

    dialog.add_response("cancel", &gettextrs::gettext("Cancel"));
    dialog.add_response("import", &gettextrs::gettext("Import"));
    dialog.set_response_appearance("import", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("import"));
    dialog.set_close_response("cancel");

    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // -- Account selection --
    let accounts_label = gtk::Label::builder()
        .label(gettextrs::gettext("Accounts"))
        .css_classes(["heading"])
        .halign(gtk::Align::Start)
        .build();
    content_box.append(&accounts_label);

    let existing_ids: Vec<uuid::Uuid> = existing_accounts.iter().map(|a| a.id()).collect();

    let account_checks: Vec<(uuid::Uuid, gtk::CheckButton)> = envelope
        .accounts
        .iter()
        .map(|exported| {
            let is_duplicate = existing_ids.contains(&exported.id);
            let label = if is_duplicate {
                format!(
                    "{} ({})",
                    exported.display_name,
                    gettextrs::gettext("duplicate")
                )
            } else {
                exported.display_name.clone()
            };
            let check = gtk::CheckButton::builder()
                .label(label)
                .active(true)
                .build();
            (exported.id, check)
        })
        .collect();

    for (_, check) in &account_checks {
        content_box.append(check);
    }

    // -- Category selection --
    let categories_label = gtk::Label::builder()
        .label(gettextrs::gettext("Data categories"))
        .css_classes(["heading"])
        .halign(gtk::Align::Start)
        .margin_top(8)
        .build();
    content_box.append(&categories_label);

    let category_checks: Vec<(ExportCategory, gtk::CheckButton)> = ExportCategory::all()
        .iter()
        .map(|cat| {
            let check = gtk::CheckButton::builder()
                .label(cat.to_string())
                .active(true)
                .build();
            (*cat, check)
        })
        .collect();

    for (_, check) in &category_checks {
        content_box.append(check);
    }

    // -- Duplicate strategy --
    let duplicate_label = gtk::Label::builder()
        .label(gettextrs::gettext("When an account already exists"))
        .css_classes(["heading"])
        .halign(gtk::Align::Start)
        .margin_top(8)
        .build();
    content_box.append(&duplicate_label);

    let skip_radio = gtk::CheckButton::builder()
        .label(gettextrs::gettext("Skip duplicate"))
        .active(true)
        .build();
    content_box.append(&skip_radio);

    let update_radio = gtk::CheckButton::builder()
        .label(gettextrs::gettext("Update existing"))
        .build();
    update_radio.set_group(Some(&skip_radio));
    content_box.append(&update_radio);

    dialog.set_extra_child(Some(&content_box));

    let parent_clone = parent.clone();
    let total_in_file = envelope.accounts.len();
    dialog.connect_response(
        None,
        clone!(
            #[strong]
            parent_clone,
            move |_dialog, response| {
                if response != "import" {
                    callback(None);
                    return;
                }

                // Gather selected account IDs.
                let selected_ids: Vec<uuid::Uuid> = account_checks
                    .iter()
                    .filter(|(_, check)| check.is_active())
                    .map(|(id, _)| *id)
                    .collect();

                let account_ids = if selected_ids.len() == total_in_file {
                    vec![]
                } else {
                    selected_ids
                };

                // Gather selected categories.
                let selected_cats: Vec<ExportCategory> = category_checks
                    .iter()
                    .filter(|(_, check)| check.is_active())
                    .map(|(cat, _)| *cat)
                    .collect();

                let categories = if selected_cats.len() == ExportCategory::all().len() {
                    vec![]
                } else {
                    selected_cats
                };

                let duplicate_strategy = if update_radio.is_active() {
                    DuplicateStrategy::Update
                } else {
                    DuplicateStrategy::Skip
                };

                let options = ImportOptions {
                    account_ids,
                    categories,
                    duplicate_strategy,
                    password: password.clone(),
                };

                let mut accounts_to_mutate = existing_accounts.clone();
                match import_service::import_from_file(&mut accounts_to_mutate, &path, &options) {
                    Ok(result) => {
                        let mut parts = Vec::new();
                        if result.created_count() > 0 {
                            parts.push(gettextrs::gettext(format!(
                                "{} imported",
                                result.created_count()
                            )));
                        }
                        if result.updated_count() > 0 {
                            parts.push(gettextrs::gettext(format!(
                                "{} updated",
                                result.updated_count()
                            )));
                        }
                        if result.skipped_count() > 0 {
                            parts.push(gettextrs::gettext(format!(
                                "{} skipped",
                                result.skipped_count()
                            )));
                        }
                        if result.failed_count() > 0 {
                            parts.push(gettextrs::gettext(format!(
                                "{} failed",
                                result.failed_count()
                            )));
                        }
                        let msg = if parts.is_empty() {
                            gettextrs::gettext("No accounts to import")
                        } else {
                            parts.join(", ")
                        };
                        let toast = adw::Toast::new(&msg);
                        toast.set_timeout(3);
                        show_toast(&parent_clone, &toast);
                        callback(Some(accounts_to_mutate));
                    }
                    Err(e) => {
                        let msg = gettextrs::gettext(format!("Import failed: {e}"));
                        let toast = adw::Toast::new(&msg);
                        toast.set_timeout(5);
                        show_toast(&parent_clone, &toast);
                        callback(None);
                    }
                }
            }
        ),
    );

    dialog.present(Some(parent));
}

/// Attempt to show an `AdwToast` on the window.
fn show_toast(window: &adw::ApplicationWindow, toast: &adw::Toast) {
    if let Some(content) = window.content() {
        let overlay = adw::ToastOverlay::new();
        overlay.set_child(Some(&content));
        window.set_content(Some(&overlay));
        overlay.add_toast(toast.clone());
    }
}
