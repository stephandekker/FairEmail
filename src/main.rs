pub mod core;
pub mod services;
#[cfg(feature = "ui")]
mod ui;

#[cfg(feature = "ui")]
fn main() {
    // Check for CLI flags before starting the UI.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--dev-fetch") {
        dev_fetch::run_dev_fetch(&args);
        return;
    }
    if args.iter().any(|a| a == "--rebuild-index") {
        rebuild_index_cli::run_rebuild_index();
        return;
    }

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

/// Rebuild-index CLI handler: reconstructs the SQLite index from on-disk `.eml` files.
#[cfg(feature = "ui")]
mod rebuild_index_cli {
    use crate::services::database::open_and_migrate;
    use crate::services::rebuild_index::rebuild_index;

    pub fn run_rebuild_index() {
        let data_dir = if let Ok(custom) = std::env::var("FAIRMAIL_DATA_DIR") {
            std::path::PathBuf::from(custom)
        } else {
            glib::user_data_dir().join("fairmail")
        };

        let db_path = data_dir.join("fairmail.db");
        let conn = open_and_migrate(&db_path).expect("could not open database");

        let content_root = data_dir.join("messages");
        eprintln!("Rebuilding index from {}", content_root.display());

        match rebuild_index(&conn, &content_root) {
            Ok(result) => {
                eprintln!("Rebuild complete:");
                eprintln!("  Files scanned:     {}", result.files_scanned);
                eprintln!("  Messages inserted: {}", result.messages_inserted);
                eprintln!("  Skipped (existing):{}", result.skipped_existing);
                eprintln!("  Skipped (errors):  {}", result.skipped_errors);
            }
            Err(e) => {
                eprintln!("Rebuild failed: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// Dev-only fetch module for sanity-testing the message fetch pipeline.
#[cfg(feature = "ui")]
mod dev_fetch {
    use crate::core::credential_store::{CredentialRole, CredentialStore};
    use crate::services::imap_client::ImapConnectParams;
    use crate::services::{
        count_messages, database::open_and_migrate, fetch_and_store_folder, FsContentStore,
        LibsecretCredentialStore,
    };

    pub fn run_dev_fetch(args: &[String]) {
        // Usage: --dev-fetch <account_id> <folder_name>
        let fetch_idx = args.iter().position(|a| a == "--dev-fetch").unwrap();
        let account_id = args
            .get(fetch_idx + 1)
            .expect("Usage: --dev-fetch <account_id> <folder_name>");
        let folder_name = args
            .get(fetch_idx + 2)
            .expect("Usage: --dev-fetch <account_id> <folder_name>");

        let data_dir = if let Ok(custom) = std::env::var("FAIRMAIL_DATA_DIR") {
            std::path::PathBuf::from(custom)
        } else {
            glib::user_data_dir().join("fairmail")
        };

        let db_path = data_dir.join("fairmail.db");
        let conn = open_and_migrate(&db_path).expect("could not open database");

        // Load account from database to get connection params.
        let acct_store =
            crate::services::AccountStore::new(db_path).expect("could not open account store");
        let accounts = acct_store.load_all().expect("could not load accounts");
        let account = accounts
            .iter()
            .find(|a| a.id().to_string() == *account_id)
            .unwrap_or_else(|| panic!("Account not found: {account_id}"));

        // Read IMAP credentials from keychain.
        let cred_store = LibsecretCredentialStore::new();
        let secret = cred_store
            .read(account.id(), CredentialRole::ImapPassword)
            .expect("could not read IMAP password from keychain");
        let password = secret.expose();

        let params = ImapConnectParams {
            host: account.host().to_string(),
            port: account.port(),
            encryption: account.encryption(),
            username: account.username().to_string(),
            password: password.to_string(),
            accepted_fingerprint: account
                .security_settings()
                .and_then(|s| s.certificate_fingerprint.clone()),
            insecure: account
                .security_settings()
                .map(|s| s.insecure)
                .unwrap_or(false),
            account_id: account.id().to_string(),
            client_certificate: account
                .security_settings()
                .and_then(|s| s.client_certificate.clone()),
            dane: account.security_settings().map(|s| s.dane).unwrap_or(false),
            dnssec: account
                .security_settings()
                .map(|s| s.dnssec)
                .unwrap_or(false),
        };

        let content_root = data_dir.join("messages");
        let content_store = FsContentStore::new(content_root);

        eprintln!("Fetching folder '{folder_name}' for account {account_id}...");

        match fetch_and_store_folder(&conn, &content_store, &params, folder_name) {
            Ok(result) => {
                let total = count_messages(&conn, account_id).unwrap_or(0);
                eprintln!("Fetch complete:");
                eprintln!("  Messages fetched: {}", result.messages_fetched);
                eprintln!("  UIDVALIDITY: {}", result.uidvalidity);
                eprintln!(
                    "  HIGHESTMODSEQ: {}",
                    result
                        .highestmodseq
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "N/A".to_string())
                );
                eprintln!("  Total messages in DB for account: {total}");
            }
            Err(e) => {
                eprintln!("Fetch failed: {e}");
                std::process::exit(1);
            }
        }
    }
}

#[cfg(not(feature = "ui"))]
fn main() {
    eprintln!("This binary requires the 'ui' feature. Build with: cargo build --features ui");
}
