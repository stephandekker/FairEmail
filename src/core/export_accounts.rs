use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::Account;

/// Categories of account data that can be selectively included in an export (FR-50, US-45).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportCategory {
    /// Core connection settings: host, port, encryption, auth, username, credential, SMTP.
    ConnectionSettings,
    /// Sync preferences: sync_enabled, on_demand, polling, unmetered_only, vpn_only, schedule_exempt.
    SyncSettings,
    /// Folder mappings and swipe defaults (system_folders, swipe_defaults).
    FolderMappings,
    /// Advanced security settings (DNSSEC, DANE, certificate pinning, etc.).
    SecuritySettings,
    /// Advanced fetch and keep-alive settings.
    FetchSettings,
}

impl ExportCategory {
    /// Returns all available export categories.
    pub fn all() -> &'static [ExportCategory] {
        &[
            ExportCategory::ConnectionSettings,
            ExportCategory::SyncSettings,
            ExportCategory::FolderMappings,
            ExportCategory::SecuritySettings,
            ExportCategory::FetchSettings,
        ]
    }
}

impl std::fmt::Display for ExportCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionSettings => write!(f, "Connection settings"),
            Self::SyncSettings => write!(f, "Sync settings"),
            Self::FolderMappings => write!(f, "Folder mappings"),
            Self::SecuritySettings => write!(f, "Security settings"),
            Self::FetchSettings => write!(f, "Fetch & keep-alive settings"),
        }
    }
}

/// Options controlling what and how to export (FR-47, FR-48, FR-50).
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Which account IDs to include. If empty, all accounts are exported.
    pub account_ids: Vec<Uuid>,
    /// Which data categories to include. If empty, all categories are included.
    pub categories: Vec<ExportCategory>,
    /// Optional password for encrypting the export file (FR-48).
    pub password: Option<String>,
}

/// A single account entry in the export file.
/// Fields are optional so that selective category export can omit data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedAccount {
    /// The account's unique identifier, preserved for duplicate detection on import (N-8).
    pub id: Uuid,
    /// Display name is always included.
    pub display_name: String,
    /// Protocol is always included.
    pub protocol: crate::core::Protocol,
    /// Category label, always included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Color, always included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<crate::core::AccountColor>,
    /// Avatar path, always included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_path: Option<String>,

    // -- ConnectionSettings category --
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption: Option<crate::core::EncryptionMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_method: Option<crate::core::AuthMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smtp: Option<crate::core::SmtpConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pop3_settings: Option<crate::core::Pop3Settings>,

    // -- SyncSettings category --
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_demand: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polling_interval_minutes: Option<Option<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unmetered_only: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vpn_only: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule_exempt: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notifications_enabled: Option<bool>,

    // -- FolderMappings category --
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_folders: Option<crate::core::SystemFolders>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swipe_defaults: Option<crate::core::SwipeDefaults>,

    // -- SecuritySettings category --
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_settings: Option<crate::core::SecuritySettings>,

    // -- FetchSettings category --
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fetch_settings: Option<crate::core::FetchSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keep_alive_settings: Option<crate::core::KeepAliveSettings>,
}

/// The top-level export envelope (FR-47).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEnvelope {
    /// Format version for forward compatibility.
    pub format_version: u32,
    /// ISO 8601 timestamp of when the export was created.
    pub exported_at: String,
    /// The exported account configurations.
    pub accounts: Vec<ExportedAccount>,
}

/// Encrypted export envelope (FR-48).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    /// Format version for forward compatibility.
    pub format_version: u32,
    /// Indicates this is an encrypted export.
    pub encrypted: bool,
    /// Base64-encoded Argon2 salt.
    pub salt: String,
    /// Base64-encoded AES-GCM nonce.
    pub nonce: String,
    /// Base64-encoded ciphertext.
    pub ciphertext: String,
}

/// Errors that can occur during export.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("encryption error: {0}")]
    Encryption(String),
    #[error("no accounts to export")]
    NoAccounts,
}

const FORMAT_VERSION: u32 = 1;

/// Build an [`ExportedAccount`] from an [`Account`], including only the selected categories.
fn build_exported_account(account: &Account, categories: &[ExportCategory]) -> ExportedAccount {
    let include_all = categories.is_empty();

    let include_conn = include_all || categories.contains(&ExportCategory::ConnectionSettings);
    let include_sync = include_all || categories.contains(&ExportCategory::SyncSettings);
    let include_folders = include_all || categories.contains(&ExportCategory::FolderMappings);
    let include_security = include_all || categories.contains(&ExportCategory::SecuritySettings);
    let include_fetch = include_all || categories.contains(&ExportCategory::FetchSettings);

    ExportedAccount {
        // Always included
        id: account.id(),
        display_name: account.display_name().to_string(),
        protocol: account.protocol(),
        category: account.category().map(String::from),
        color: account.color(),
        avatar_path: account.avatar_path().map(String::from),

        // ConnectionSettings
        host: include_conn.then(|| account.host().to_string()),
        port: include_conn.then(|| account.port()),
        encryption: include_conn.then(|| account.encryption()),
        auth_method: include_conn.then(|| account.auth_method()),
        username: include_conn.then(|| account.username().to_string()),
        credential: include_conn.then(|| account.credential().to_string()),
        smtp: if include_conn {
            account.smtp().cloned()
        } else {
            None
        },
        pop3_settings: if include_conn {
            account.pop3_settings().cloned()
        } else {
            None
        },

        // SyncSettings
        sync_enabled: include_sync.then(|| account.sync_enabled()),
        on_demand: include_sync.then(|| account.on_demand()),
        polling_interval_minutes: include_sync.then(|| account.polling_interval_minutes()),
        unmetered_only: include_sync.then(|| account.unmetered_only()),
        vpn_only: include_sync.then(|| account.vpn_only()),
        schedule_exempt: include_sync.then(|| account.schedule_exempt()),
        notifications_enabled: include_sync.then(|| account.notifications_enabled()),

        // FolderMappings
        system_folders: if include_folders {
            account.system_folders().cloned()
        } else {
            None
        },
        swipe_defaults: if include_folders {
            account.swipe_defaults().cloned()
        } else {
            None
        },

        // SecuritySettings
        security_settings: if include_security {
            account.security_settings().cloned()
        } else {
            None
        },

        // FetchSettings
        fetch_settings: if include_fetch {
            account.fetch_settings().cloned()
        } else {
            None
        },
        keep_alive_settings: if include_fetch {
            account.keep_alive_settings().cloned()
        } else {
            None
        },
    }
}

/// Generate an ISO 8601 timestamp string for the current time.
fn now_iso8601() -> String {
    // Use std::time for a simple UTC timestamp without extra dependencies.
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple UTC date-time formatting.
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Calculate year/month/day from days since epoch (1970-01-01).
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Encrypt plaintext JSON bytes using AES-256-GCM with an Argon2-derived key (FR-48).
fn encrypt_payload(plaintext: &[u8], password: &str) -> Result<EncryptedEnvelope, ExportError> {
    use aes_gcm::aead::Aead;
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use argon2::Argon2;
    use rand::RngCore;

    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);

    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|e| ExportError::Encryption(e.to_string()))?;

    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| ExportError::Encryption(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| ExportError::Encryption(e.to_string()))?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD;

    Ok(EncryptedEnvelope {
        format_version: FORMAT_VERSION,
        encrypted: true,
        salt: b64.encode(salt),
        nonce: b64.encode(nonce_bytes),
        ciphertext: b64.encode(ciphertext),
    })
}

/// Export the given accounts according to the provided options (FR-47, FR-48, FR-50).
///
/// Returns the serialized JSON bytes ready to be written to a file.
/// If a password is provided, the payload is encrypted with AES-256-GCM.
pub fn export_accounts(
    all_accounts: &[Account],
    options: &ExportOptions,
) -> Result<Vec<u8>, ExportError> {
    // Filter accounts by selected IDs (FR-50).
    let selected: Vec<&Account> = if options.account_ids.is_empty() {
        all_accounts.iter().collect()
    } else {
        all_accounts
            .iter()
            .filter(|a| options.account_ids.contains(&a.id()))
            .collect()
    };

    if selected.is_empty() {
        return Err(ExportError::NoAccounts);
    }

    let exported: Vec<ExportedAccount> = selected
        .iter()
        .map(|a| build_exported_account(a, &options.categories))
        .collect();

    let envelope = ExportEnvelope {
        format_version: FORMAT_VERSION,
        exported_at: now_iso8601(),
        accounts: exported,
    };

    let json = serde_json::to_string_pretty(&envelope)?;

    match &options.password {
        Some(pw) if !pw.is_empty() => {
            let encrypted = encrypt_payload(json.as_bytes(), pw)?;
            let result = serde_json::to_string_pretty(&encrypted)?;
            Ok(result.into_bytes())
        }
        _ => Ok(json.into_bytes()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        AccountColor, AuthMethod, EncryptionMode, FetchSettings, KeepAliveSettings,
        NewAccountParams, Protocol, SecuritySettings, SmtpConfig, SwipeAction, SwipeDefaults,
        SystemFolders,
    };

    fn make_test_account(name: &str) -> Account {
        Account::new(NewAccountParams {
            display_name: name.into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            smtp: Some(SmtpConfig {
                host: "smtp.example.com".into(),
                port: 587,
                encryption: EncryptionMode::StartTls,
                auth_method: AuthMethod::Plain,
                username: "user@example.com".into(),
                credential: "secret".into(),
            }),
            pop3_settings: None,
            color: Some(AccountColor::new(1.0, 0.0, 0.0)),
            avatar_path: Some("/tmp/avatar.png".into()),
            category: Some("Work".into()),
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: Some(15),
            unmetered_only: true,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: Some(SystemFolders {
                drafts: Some("Drafts".into()),
                sent: Some("Sent".into()),
                archive: None,
                trash: Some("Trash".into()),
                junk: None,
            }),
            swipe_defaults: Some(SwipeDefaults {
                swipe_left: SwipeAction::Delete,
                swipe_right: SwipeAction::Archive,
                default_move_to: None,
            }),
            notifications_enabled: true,
            security_settings: Some(SecuritySettings {
                dnssec: true,
                dane: false,
                insecure: false,
                certificate_fingerprint: None,
                client_certificate: None,
                auth_realm: Some("example.com".into()),
            }),
            fetch_settings: Some(FetchSettings {
                partial_fetch: true,
                raw_fetch: false,
                ignore_size_limits: false,
                date_header_preference: Default::default(),
                utf8_support: true,
            }),
            keep_alive_settings: Some(KeepAliveSettings {
                use_noop_instead_of_idle: true,
            }),
        })
        .unwrap()
    }

    #[test]
    fn export_all_accounts_all_categories() {
        let acct1 = make_test_account("Account 1");
        let acct2 = make_test_account("Account 2");
        let accounts = vec![acct1.clone(), acct2.clone()];

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };

        let result = export_accounts(&accounts, &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();

        assert_eq!(envelope.format_version, 1);
        assert_eq!(envelope.accounts.len(), 2);
        assert_eq!(envelope.accounts[0].id, acct1.id());
        assert_eq!(envelope.accounts[1].id, acct2.id());
    }

    #[test]
    fn export_preserves_unique_id() {
        let acct = make_test_account("Test");
        let original_id = acct.id();

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();

        assert_eq!(envelope.accounts[0].id, original_id);
    }

    #[test]
    fn export_selective_accounts() {
        let acct1 = make_test_account("Account 1");
        let acct2 = make_test_account("Account 2");
        let acct3 = make_test_account("Account 3");
        let id2 = acct2.id();
        let accounts = vec![acct1, acct2, acct3];

        let options = ExportOptions {
            account_ids: vec![id2],
            categories: vec![],
            password: None,
        };

        let result = export_accounts(&accounts, &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();

        assert_eq!(envelope.accounts.len(), 1);
        assert_eq!(envelope.accounts[0].id, id2);
        assert_eq!(envelope.accounts[0].display_name, "Account 2");
    }

    #[test]
    fn export_selective_categories_connection_only() {
        let acct = make_test_account("Test");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![ExportCategory::ConnectionSettings],
            password: None,
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();
        let exported = &envelope.accounts[0];

        // Connection settings present
        assert!(exported.host.is_some());
        assert!(exported.port.is_some());
        assert!(exported.encryption.is_some());
        assert!(exported.username.is_some());
        assert!(exported.credential.is_some());
        assert!(exported.smtp.is_some());

        // Other categories absent
        assert!(exported.sync_enabled.is_none());
        assert!(exported.system_folders.is_none());
        assert!(exported.security_settings.is_none());
        assert!(exported.fetch_settings.is_none());
    }

    #[test]
    fn export_selective_categories_sync_only() {
        let acct = make_test_account("Test");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![ExportCategory::SyncSettings],
            password: None,
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();
        let exported = &envelope.accounts[0];

        // Sync settings present
        assert!(exported.sync_enabled.is_some());
        assert!(exported.on_demand.is_some());
        assert!(exported.notifications_enabled.is_some());

        // Connection settings absent
        assert!(exported.host.is_none());
        assert!(exported.credential.is_none());
    }

    #[test]
    fn export_selective_categories_folders_only() {
        let acct = make_test_account("Test");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![ExportCategory::FolderMappings],
            password: None,
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();
        let exported = &envelope.accounts[0];

        assert!(exported.system_folders.is_some());
        assert!(exported.swipe_defaults.is_some());
        assert!(exported.host.is_none());
        assert!(exported.sync_enabled.is_none());
    }

    #[test]
    fn export_selective_categories_security_only() {
        let acct = make_test_account("Test");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![ExportCategory::SecuritySettings],
            password: None,
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();
        let exported = &envelope.accounts[0];

        assert!(exported.security_settings.is_some());
        assert!(exported.host.is_none());
        assert!(exported.fetch_settings.is_none());
    }

    #[test]
    fn export_selective_categories_fetch_only() {
        let acct = make_test_account("Test");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![ExportCategory::FetchSettings],
            password: None,
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();
        let exported = &envelope.accounts[0];

        assert!(exported.fetch_settings.is_some());
        assert!(exported.keep_alive_settings.is_some());
        assert!(exported.host.is_none());
        assert!(exported.security_settings.is_none());
    }

    #[test]
    fn export_multiple_categories() {
        let acct = make_test_account("Test");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![
                ExportCategory::ConnectionSettings,
                ExportCategory::FolderMappings,
            ],
            password: None,
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();
        let exported = &envelope.accounts[0];

        assert!(exported.host.is_some());
        assert!(exported.system_folders.is_some());
        assert!(exported.sync_enabled.is_none());
    }

    #[test]
    fn export_with_password_produces_encrypted_envelope() {
        let acct = make_test_account("Test");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: Some("my-secret-password".into()),
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let encrypted: EncryptedEnvelope = serde_json::from_slice(&result).unwrap();

        assert_eq!(encrypted.format_version, 1);
        assert!(encrypted.encrypted);
        assert!(!encrypted.salt.is_empty());
        assert!(!encrypted.nonce.is_empty());
        assert!(!encrypted.ciphertext.is_empty());
    }

    #[test]
    fn export_with_password_can_be_decrypted() {
        use aes_gcm::aead::Aead;
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        use argon2::Argon2;
        use base64::Engine;

        let acct = make_test_account("Decrypt Test");
        let original_id = acct.id();
        let password = "test-password-123";

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: Some(password.into()),
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let encrypted: EncryptedEnvelope = serde_json::from_slice(&result).unwrap();

        // Decrypt
        let b64 = base64::engine::general_purpose::STANDARD;
        let salt = b64.decode(&encrypted.salt).unwrap();
        let nonce_bytes = b64.decode(&encrypted.nonce).unwrap();
        let ciphertext = b64.decode(&encrypted.ciphertext).unwrap();

        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &salt, &mut key)
            .unwrap();

        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).unwrap();

        let envelope: ExportEnvelope = serde_json::from_slice(&plaintext).unwrap();
        assert_eq!(envelope.accounts.len(), 1);
        assert_eq!(envelope.accounts[0].id, original_id);
        assert_eq!(envelope.accounts[0].display_name, "Decrypt Test");
    }

    #[test]
    fn export_no_accounts_returns_error() {
        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };

        let result = export_accounts(&[], &options);
        assert!(matches!(result, Err(ExportError::NoAccounts)));
    }

    #[test]
    fn export_selected_ids_not_found_returns_error() {
        let acct = make_test_account("Test");

        let options = ExportOptions {
            account_ids: vec![Uuid::new_v4()], // ID that doesn't exist
            categories: vec![],
            password: None,
        };

        let result = export_accounts(&[acct], &options);
        assert!(matches!(result, Err(ExportError::NoAccounts)));
    }

    #[test]
    fn export_always_includes_identity_fields() {
        let acct = make_test_account("Test");

        // Even with only SecuritySettings category, identity fields are present.
        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![ExportCategory::SecuritySettings],
            password: None,
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();
        let exported = &envelope.accounts[0];

        assert_eq!(exported.display_name, "Test");
        assert_eq!(exported.protocol, Protocol::Imap);
        assert!(exported.color.is_some());
    }

    #[test]
    fn export_envelope_has_timestamp() {
        let acct = make_test_account("Test");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();

        assert!(!envelope.exported_at.is_empty());
        assert!(envelope.exported_at.ends_with('Z'));
    }

    #[test]
    fn export_all_categories_includes_everything() {
        let acct = make_test_account("Full");

        let options = ExportOptions {
            account_ids: vec![],
            categories: ExportCategory::all().to_vec(),
            password: None,
        };

        let result = export_accounts(&[acct], &options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&result).unwrap();
        let exported = &envelope.accounts[0];

        assert!(exported.host.is_some());
        assert!(exported.sync_enabled.is_some());
        assert!(exported.system_folders.is_some());
        assert!(exported.security_settings.is_some());
        assert!(exported.fetch_settings.is_some());
    }

    #[test]
    fn days_to_ymd_epoch() {
        let (y, m, d) = super::days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn days_to_ymd_known_date() {
        // 2026-05-04 is day 20577 since epoch
        let (y, m, d) = super::days_to_ymd(20577);
        assert_eq!((y, m, d), (2026, 5, 4));
    }

    #[test]
    fn export_category_display() {
        assert_eq!(
            ExportCategory::ConnectionSettings.to_string(),
            "Connection settings"
        );
        assert_eq!(ExportCategory::SyncSettings.to_string(), "Sync settings");
        assert_eq!(
            ExportCategory::FolderMappings.to_string(),
            "Folder mappings"
        );
        assert_eq!(
            ExportCategory::SecuritySettings.to_string(),
            "Security settings"
        );
        assert_eq!(
            ExportCategory::FetchSettings.to_string(),
            "Fetch & keep-alive settings"
        );
    }
}
