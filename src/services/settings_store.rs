use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::auth_mechanism::MechanismToggles;

/// Controls when EXPUNGE is issued after setting the \Deleted flag.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpungeMode {
    /// Expunge immediately after flagging (default).
    #[default]
    Immediate,
    /// Defer expunge to a manual or scheduled batch operation.
    Deferred,
}

/// Application settings that can be toggled by the user.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppSettings {
    /// Whether accounts are grouped by category in the navigation pane (FR-21).
    #[serde(default)]
    pub category_display_enabled: bool,

    /// User-configured browser for OAuth flows (FR-31).
    ///
    /// Can be a browser name (e.g. "Firefox") or an absolute path to an executable.
    /// When `None`, the application auto-selects a privacy-focused browser or
    /// falls back to the system default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_browser: Option<String>,

    /// Global toggles for password-based authentication mechanisms (FR-25 – FR-29).
    #[serde(default)]
    pub mechanism_toggles: MechanismToggles,

    /// Controls when EXPUNGE is issued after permanent deletion.
    /// `Immediate` (default) expunges right away; `Deferred` waits for a
    /// manual or scheduled batch operation.
    #[serde(default)]
    pub expunge_mode: ExpungeMode,
}

/// Errors from the settings persistence layer.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Persists application settings as a JSON file.
#[derive(Debug, Clone)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Load settings from disk. Returns defaults if the file does not exist.
    pub fn load(&self) -> Result<AppSettings, SettingsError> {
        if !self.path.exists() {
            return Ok(AppSettings::default());
        }
        let data = fs::read_to_string(&self.path)?;
        let settings: AppSettings = serde_json::from_str(&data)?;
        Ok(settings)
    }

    /// Save settings to disk atomically.
    pub fn save(&self, settings: &AppSettings) -> Result<(), SettingsError> {
        let json = serde_json::to_string_pretty(settings)?;
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        let tmp_path = dir.join(".settings.tmp");
        {
            let mut f = fs::File::create(&tmp_path)?;
            f.write_all(json.as_bytes())?;
            f.sync_all()?;
        }
        fs::rename(&tmp_path, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings() {
        let settings = AppSettings::default();
        assert!(!settings.category_display_enabled);
    }

    #[test]
    fn load_returns_defaults_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));
        let settings = store.load().unwrap();
        assert!(!settings.category_display_enabled);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));
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
    fn atomic_write_leaves_no_tmp_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));
        store.save(&AppSettings::default()).unwrap();
        assert!(!dir.path().join(".settings.tmp").exists());
    }

    #[test]
    fn deserialize_without_category_display_defaults_to_false() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, "{}").unwrap();
        let store = SettingsStore::new(path);
        let settings = store.load().unwrap();
        assert!(!settings.category_display_enabled);
    }
}
