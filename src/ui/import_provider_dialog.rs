//! UI dialog for importing a custom OAuth provider configuration file (FR-26, FR-28).

use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use glib::clone;

/// Show a file chooser for importing a custom provider configuration file.
///
/// On success, shows a toast with the number of providers imported.
/// On failure, shows a toast with the error message.
pub(crate) fn show(parent: &adw::ApplicationWindow) {
    let json_filter = gtk::FileFilter::new();
    json_filter.set_name(Some(&gettextrs::gettext("JSON files")));
    json_filter.add_pattern("*.json");

    let all_filter = gtk::FileFilter::new();
    all_filter.set_name(Some(&gettextrs::gettext("All files")));
    all_filter.add_pattern("*");

    let filters = gtk4::gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&json_filter);
    filters.append(&all_filter);

    let file_dialog = gtk::FileDialog::builder()
        .title(gettextrs::gettext("Import provider configuration"))
        .filters(&filters)
        .build();

    let parent_clone = parent.clone();

    file_dialog.open(
        Some(parent),
        None::<&gtk4::gio::Cancellable>,
        clone!(
            #[strong]
            parent_clone,
            move |result| {
                match result {
                    Ok(file) => {
                        if let Some(path) = file.path() {
                            handle_import(&parent_clone, &path);
                        }
                    }
                    Err(_) => {
                        // User cancelled — nothing to do.
                    }
                }
            }
        ),
    );
}

/// Attempt the import and show result as a toast.
fn handle_import(parent: &adw::ApplicationWindow, path: &std::path::Path) {
    match crate::services::user_provider_service::import_provider_file(path) {
        Ok(count) => {
            let msg = gettextrs::ngettext(
                "Imported %d provider configuration",
                "Imported %d provider configurations",
                count as u32,
            )
            .replace("%d", &count.to_string());
            show_result_dialog(parent, &msg);
        }
        Err(e) => {
            let msg = gettextrs::gettext("Failed to import provider configuration: %s")
                .replace("%s", &e.to_string());
            show_result_dialog(parent, &msg);
        }
    }
}

/// Show an alert dialog with the import result message.
fn show_result_dialog(parent: &adw::ApplicationWindow, message: &str) {
    let dialog = adw::AlertDialog::builder()
        .heading(gettextrs::gettext("Provider import"))
        .body(message)
        .build();
    dialog.add_response("ok", &gettextrs::gettext("OK"));
    dialog.present(Some(parent));
}
