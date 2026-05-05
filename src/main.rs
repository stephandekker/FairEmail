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

    use crate::core::credential_store::{CredentialRole, CredentialStore, SecretValue};
    use crate::services::{
        AccountStore, LibsecretCredentialStore, SqliteOrderStore, SqliteSettingsStore,
    };

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

    /// Migrate settings from the legacy JSON file into the SQLite store.
    /// Idempotent. On success, renames `settings.json` to `settings.json.migrated`.
    fn migrate_json_settings(dir: &std::path::Path, store: &SqliteSettingsStore) {
        let json_path = dir.join("settings.json");
        if !json_path.exists() {
            return;
        }

        let data = match std::fs::read_to_string(&json_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Warning: could not read settings.json for migration: {e}");
                return;
            }
        };

        let settings: crate::services::AppSettings = match serde_json::from_str(&data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: could not parse settings.json for migration: {e}");
                return;
            }
        };

        match store.import_from_json(&settings) {
            Ok(_) => {
                if let Err(e) = std::fs::rename(&json_path, dir.join("settings.json.migrated")) {
                    eprintln!("Warning: could not rename settings.json to .migrated: {e}");
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: settings migration failed, leaving settings.json in place: {e}"
                );
            }
        }
    }

    /// Migrate account order from the legacy JSON file into the SQLite store.
    /// Idempotent. On success, renames `order.json` to `order.json.migrated`.
    fn migrate_json_order(dir: &std::path::Path, store: &SqliteOrderStore) {
        let json_path = dir.join("order.json");
        if !json_path.exists() {
            return;
        }

        let data = match std::fs::read_to_string(&json_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Warning: could not read order.json for migration: {e}");
                return;
            }
        };

        let order: Vec<uuid::Uuid> = match serde_json::from_str(&data) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Warning: could not parse order.json for migration: {e}");
                return;
            }
        };

        if order.is_empty() {
            let _ = std::fs::rename(&json_path, dir.join("order.json.migrated"));
            return;
        }

        match store.import_from_json(&order) {
            Ok(_) => {
                if let Err(e) = std::fs::rename(&json_path, dir.join("order.json.migrated")) {
                    eprintln!("Warning: could not rename order.json to .migrated: {e}");
                }
            }
            Err(e) => {
                eprintln!("Warning: order migration failed, leaving order.json in place: {e}");
            }
        }
    }

    /// Migrate plaintext credentials from the database into the system keychain.
    /// Idempotent: only processes accounts that still have a non-empty credential column.
    /// On success, clears the credential columns in the database.
    fn migrate_credentials_to_keychain(store: &AccountStore, cred_store: &dyn CredentialStore) {
        let plaintext_creds = match store.read_plaintext_credentials() {
            Ok(creds) => creds,
            Err(e) => {
                eprintln!("Warning: could not read plaintext credentials for migration: {e}");
                return;
            }
        };

        if plaintext_creds.is_empty() {
            return;
        }

        let mut all_ok = true;
        for (account_id, imap_cred, smtp_cred_opt) in &plaintext_creds {
            if !imap_cred.is_empty() {
                if let Err(e) = cred_store.write(
                    *account_id,
                    CredentialRole::ImapPassword,
                    &SecretValue::new(imap_cred.clone()),
                ) {
                    eprintln!(
                        "Warning: could not migrate credential to keychain for account {account_id}: {e}"
                    );
                    all_ok = false;
                }
            }
            if let Some(smtp_cred) = smtp_cred_opt {
                if let Err(e) = cred_store.write(
                    *account_id,
                    CredentialRole::SmtpPassword,
                    &SecretValue::new(smtp_cred.clone()),
                ) {
                    eprintln!(
                        "Warning: could not migrate SMTP credential to keychain for account {account_id}: {e}"
                    );
                    all_ok = false;
                }
            }
        }

        if all_ok {
            if let Err(e) = store.clear_plaintext_credentials() {
                eprintln!("Warning: could not clear plaintext credentials from database: {e}");
            }
        }
    }

    let app = adw::Application::builder()
        .application_id("com.example.Fairmail")
        .build();

    app.connect_activate(clone!(move |app| {
        let dir = data_dir();
        let db_path = dir.join("fairmail.db");
        let store =
            Rc::new(AccountStore::new(db_path.clone()).expect("could not open account database"));
        migrate_json_accounts(&dir, &store);
        let cred_store: Rc<dyn crate::core::CredentialStore> =
            Rc::new(LibsecretCredentialStore::new());
        migrate_credentials_to_keychain(&store, cred_store.as_ref());
        let settings_store = Rc::new(
            SqliteSettingsStore::new(db_path.clone()).expect("could not open settings database"),
        );
        migrate_json_settings(&dir, &settings_store);
        let order_store =
            Rc::new(SqliteOrderStore::new(db_path).expect("could not open order database"));
        migrate_json_order(&dir, &order_store);
        ui::window::build(app, store, settings_store, order_store, cred_store);
    }));

    app.run();
}

#[cfg(not(feature = "ui"))]
fn main() {
    eprintln!("This binary requires the 'ui' feature. Build with: cargo build --features ui");
}
