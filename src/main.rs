pub mod core;
pub mod services;
#[cfg(feature = "ui")]
mod ui;

#[cfg(feature = "ui")]
fn main() {
    use glib::clone;
    use gtk4::prelude::*;
    use libadwaita as adw;
    use std::rc::Rc;

    use crate::services::{AccountStore, OrderStore, SettingsStore};

    fn data_dir() -> std::path::PathBuf {
        let base = if let Ok(custom) = std::env::var("FAIRMAIL_DATA_DIR") {
            std::path::PathBuf::from(custom)
        } else {
            glib::user_data_dir().join("fairmail")
        };
        std::fs::create_dir_all(&base).expect("could not create data directory");
        base
    }

    /// Migrate accounts from the legacy JSON file into the SQLite store.
    /// Idempotent: skips accounts already present. On success, renames the
    /// JSON file to `.migrated`. On failure, leaves the file in place.
    fn migrate_json_accounts(dir: &std::path::Path, store: &AccountStore) {
        let json_path = dir.join("accounts.json");
        if !json_path.exists() {
            return;
        }

        let data = match std::fs::read_to_string(&json_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Warning: could not read accounts.json for migration: {e}");
                return;
            }
        };

        let accounts: Vec<crate::core::Account> = match serde_json::from_str(&data) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Warning: could not parse accounts.json for migration: {e}");
                return;
            }
        };

        if accounts.is_empty() {
            // Empty file — just rename it.
            let _ = std::fs::rename(&json_path, dir.join("accounts.json.migrated"));
            return;
        }

        match store.import_from_json(&accounts) {
            Ok(_) => {
                // Success — rename to .migrated.
                if let Err(e) = std::fs::rename(&json_path, dir.join("accounts.json.migrated")) {
                    eprintln!("Warning: could not rename accounts.json to .migrated: {e}");
                }
            }
            Err(e) => {
                eprintln!("Warning: JSON migration failed, leaving accounts.json in place: {e}");
            }
        }
    }

    let app = adw::Application::builder()
        .application_id("com.example.Fairmail")
        .build();

    app.connect_activate(clone!(move |app| {
        let dir = data_dir();
        let store = Rc::new(
            AccountStore::new(dir.join("fairmail.db")).expect("could not open account database"),
        );
        migrate_json_accounts(&dir, &store);
        let settings_store = Rc::new(SettingsStore::new(dir.join("settings.json")));
        let order_store = Rc::new(OrderStore::new(dir.join("order.json")));
        ui::window::build(app, store, settings_store, order_store);
    }));

    app.run();
}

#[cfg(not(feature = "ui"))]
fn main() {
    eprintln!("This binary requires the 'ui' feature. Build with: cargo build --features ui");
}
