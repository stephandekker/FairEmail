use std::cell::RefCell;
use std::path::PathBuf;

use rusqlite::Connection;

use crate::services::database;
use crate::services::settings_store::{AppSettings, SettingsError};

/// SQLite-backed settings store. Drop-in replacement for the JSON-backed
/// `SettingsStore` with the same public method signatures.
#[derive(Debug)]
pub struct SqliteSettingsStore {
    conn: RefCell<Connection>,
}

impl SqliteSettingsStore {
    /// Open the database at `db_path`, run migrations, and return a ready store.
    pub fn new(db_path: PathBuf) -> Result<Self, SettingsError> {
        let conn = database::open_and_migrate(&db_path).map_err(|e| match e {
            database::DatabaseError::Sqlite(e) => {
                SettingsError::Io(std::io::Error::other(e.to_string()))
            }
            database::DatabaseError::Io(e) => SettingsError::Io(e),
        })?;
        Ok(Self {
            conn: RefCell::new(conn),
        })
    }

    /// Load settings from the database. Returns defaults if no keys exist.
    pub fn load(&self) -> Result<AppSettings, SettingsError> {
        let conn = self.conn.borrow();
        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = ?1")
            .map_err(|e| SettingsError::Io(std::io::Error::other(e.to_string())))?;

        let category_display_enabled: bool = stmt
            .query_row(["category_display_enabled"], |row| {
                let val: String = row.get(0)?;
                Ok(val)
            })
            .ok()
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(false);

        let oauth_browser: Option<String> = stmt
            .query_row(["oauth_browser"], |row| {
                let val: String = row.get(0)?;
                Ok(val)
            })
            .ok()
            .and_then(|v| serde_json::from_str(&v).ok());

        let mechanism_toggles: crate::core::auth_mechanism::MechanismToggles = stmt
            .query_row(["mechanism_toggles"], |row| {
                let val: String = row.get(0)?;
                Ok(val)
            })
            .ok()
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default();

        let expunge_mode: crate::services::settings_store::ExpungeMode = stmt
            .query_row(["expunge_mode"], |row| {
                let val: String = row.get(0)?;
                Ok(val)
            })
            .ok()
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default();

        Ok(AppSettings {
            category_display_enabled,
            oauth_browser,
            mechanism_toggles,
            expunge_mode,
        })
    }

    /// Save settings to the database.
    pub fn save(&self, settings: &AppSettings) -> Result<(), SettingsError> {
        let conn = self.conn.borrow();
        let value = serde_json::to_string(&settings.category_display_enabled)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params!["category_display_enabled", value],
        )
        .map_err(|e| SettingsError::Io(std::io::Error::other(e.to_string())))?;

        let browser_value = serde_json::to_string(&settings.oauth_browser)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params!["oauth_browser", browser_value],
        )
        .map_err(|e| SettingsError::Io(std::io::Error::other(e.to_string())))?;

        let toggles_value = serde_json::to_string(&settings.mechanism_toggles)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params!["mechanism_toggles", toggles_value],
        )
        .map_err(|e| SettingsError::Io(std::io::Error::other(e.to_string())))?;

        let expunge_value = serde_json::to_string(&settings.expunge_mode)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params!["expunge_mode", expunge_value],
        )
        .map_err(|e| SettingsError::Io(std::io::Error::other(e.to_string())))?;
        Ok(())
    }

    /// Import settings from an `AppSettings` struct in a single transaction.
    /// Idempotent: existing keys are overwritten.
    pub fn import_from_json(&self, settings: &AppSettings) -> Result<(), SettingsError> {
        self.save(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, SqliteSettingsStore) {
        let dir = TempDir::new().unwrap();
        let store = SqliteSettingsStore::new(dir.path().join("fairmail.db")).unwrap();
        (dir, store)
    }

    #[test]
    fn load_returns_defaults_on_empty_db() {
        let (_dir, store) = make_store();
        let settings = store.load().unwrap();
        assert!(!settings.category_display_enabled);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (_dir, store) = make_store();
        let settings = AppSettings {
            category_display_enabled: true,
            oauth_browser: None,
            ..Default::default()
        };
        store.save(&settings).unwrap();
        let loaded = store.load().unwrap();
        assert!(loaded.category_display_enabled);
    }

    #[test]
    fn save_is_idempotent() {
        let (_dir, store) = make_store();
        let settings = AppSettings {
            category_display_enabled: true,
            oauth_browser: None,
            ..Default::default()
        };
        store.save(&settings).unwrap();
        store.save(&settings).unwrap();
        let loaded = store.load().unwrap();
        assert!(loaded.category_display_enabled);
    }

    #[test]
    fn import_from_json_works() {
        let (_dir, store) = make_store();
        let settings = AppSettings {
            category_display_enabled: true,
            oauth_browser: None,
            ..Default::default()
        };
        store.import_from_json(&settings).unwrap();
        let loaded = store.load().unwrap();
        assert!(loaded.category_display_enabled);
    }

    #[test]
    fn mechanism_toggles_roundtrip() {
        let (_dir, store) = make_store();
        let toggles = crate::core::auth_mechanism::MechanismToggles {
            cram_md5_enabled: false,
            apop_enabled: true,
            ..Default::default()
        };
        let settings = AppSettings {
            category_display_enabled: false,
            oauth_browser: None,
            mechanism_toggles: toggles,
            ..Default::default()
        };
        store.save(&settings).unwrap();
        let loaded = store.load().unwrap();
        assert!(!loaded.mechanism_toggles.cram_md5_enabled);
        assert!(loaded.mechanism_toggles.apop_enabled);
        assert!(loaded.mechanism_toggles.plain_enabled);
    }

    #[test]
    fn mechanism_toggles_default_on_empty_db() {
        let (_dir, store) = make_store();
        let loaded = store.load().unwrap();
        assert!(loaded.mechanism_toggles.plain_enabled);
        assert!(loaded.mechanism_toggles.login_enabled);
        assert!(loaded.mechanism_toggles.ntlm_enabled);
        assert!(loaded.mechanism_toggles.cram_md5_enabled);
        assert!(!loaded.mechanism_toggles.apop_enabled);
    }
}
