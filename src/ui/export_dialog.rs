use std::rc::Rc;

use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::core::{Account, ExportCategory, ExportOptions};
use crate::services::export_to_file;

/// Show the export dialog (FR-47, FR-48, FR-50, US-43, US-45).
///
/// The dialog allows the user to:
/// - Select which accounts to export (FR-50).
/// - Select which data categories to include (FR-50, US-45).
/// - Optionally set a password for encryption (FR-48).
/// - Choose an output file path.
///
/// The callback receives `true` on success, `false` on cancellation or failure.
pub(crate) fn show(
    parent: &adw::ApplicationWindow,
    accounts: Vec<Account>,
    callback: impl Fn(bool) + 'static,
) {
    let dialog = adw::AlertDialog::builder()
        .heading(gettextrs::gettext("Export Account Configurations"))
        .body(gettextrs::gettext(
            "Select which accounts and data to export.",
        ))
        .build();

    dialog.add_response("cancel", &gettextrs::gettext("Cancel"));
    dialog.add_response("export", &gettextrs::gettext("Export"));
    dialog.set_response_appearance("export", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("export"));
    dialog.set_close_response("cancel");

    // Build the content area with account checkboxes, category checkboxes, and password field.
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

    let account_checks: Vec<(uuid::Uuid, gtk::CheckButton)> = accounts
        .iter()
        .map(|acct| {
            let check = gtk::CheckButton::builder()
                .label(acct.display_name())
                .active(true)
                .build();
            (acct.id(), check)
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

    // -- Password field (optional encryption) --
    let password_label = gtk::Label::builder()
        .label(gettextrs::gettext("Password (optional, for encryption)"))
        .css_classes(["heading"])
        .halign(gtk::Align::Start)
        .margin_top(8)
        .build();
    content_box.append(&password_label);

    let password_entry = gtk::PasswordEntry::builder()
        .placeholder_text(gettextrs::gettext("Leave empty for no encryption"))
        .show_peek_icon(true)
        .build();
    content_box.append(&password_entry);

    dialog.set_extra_child(Some(&content_box));

    let accounts_for_export = accounts;
    let parent_clone = parent.clone();
    let callback = Rc::new(callback);

    dialog.connect_response(
        None,
        clone!(
            #[strong]
            parent_clone,
            move |_dialog, response| {
                if response != "export" {
                    callback(false);
                    return;
                }

                // Gather selected account IDs.
                let selected_ids: Vec<uuid::Uuid> = account_checks
                    .iter()
                    .filter(|(_, check)| check.is_active())
                    .map(|(id, _)| *id)
                    .collect();

                // If all are selected, pass empty vec (meaning "all").
                let account_ids = if selected_ids.len() == accounts_for_export.len() {
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

                // If all are selected, pass empty vec (meaning "all").
                let categories = if selected_cats.len() == ExportCategory::all().len() {
                    vec![]
                } else {
                    selected_cats
                };

                let password = {
                    let text = password_entry.text();
                    let trimmed = text.trim().to_string();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    }
                };

                let options = ExportOptions {
                    account_ids,
                    categories,
                    password,
                };

                // Show file chooser for output path.
                let file_dialog = gtk::FileDialog::builder()
                    .title(gettextrs::gettext("Export to file"))
                    .initial_name("fairmail-accounts-export.json")
                    .build();

                let accounts_clone = accounts_for_export.clone();
                let cb = callback.clone();
                file_dialog.save(
                    Some(&parent_clone),
                    None::<&gtk4::gio::Cancellable>,
                    clone!(
                        #[strong]
                        parent_clone,
                        move |result| {
                            match result {
                                Ok(file) => {
                                    if let Some(path) = file.path() {
                                        match export_to_file(&accounts_clone, &options, &path) {
                                            Ok(export_result) => {
                                                let msg = if export_result.encrypted {
                                                    gettextrs::gettext(format!(
                                                        "Exported {} account(s) (encrypted)",
                                                        export_result.account_count
                                                    ))
                                                } else {
                                                    gettextrs::gettext(format!(
                                                        "Exported {} account(s)",
                                                        export_result.account_count
                                                    ))
                                                };
                                                let toast = adw::Toast::new(&msg);
                                                toast.set_timeout(3);
                                                show_toast(&parent_clone, &toast);
                                                cb(true);
                                            }
                                            Err(e) => {
                                                let msg = gettextrs::gettext(format!(
                                                    "Export failed: {e}"
                                                ));
                                                let toast = adw::Toast::new(&msg);
                                                toast.set_timeout(5);
                                                show_toast(&parent_clone, &toast);
                                                cb(false);
                                            }
                                        }
                                    } else {
                                        cb(false);
                                    }
                                }
                                Err(_) => {
                                    // User cancelled the file chooser.
                                    cb(false);
                                }
                            }
                        }
                    ),
                );
            }
        ),
    );

    dialog.present(Some(parent));
}

/// Attempt to show an `AdwToast` on the window by wrapping it in an overlay.
fn show_toast(window: &adw::ApplicationWindow, toast: &adw::Toast) {
    // Walk the widget tree to find an AdwToastOverlay, or print to stderr as fallback.
    if let Some(content) = window.content() {
        let overlay = adw::ToastOverlay::new();
        overlay.set_child(Some(&content));
        window.set_content(Some(&overlay));
        overlay.add_toast(toast.clone());
    }
}
