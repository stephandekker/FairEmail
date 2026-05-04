use uuid::Uuid;

use crate::core::export_accounts::{
    EncryptedEnvelope, ExportCategory, ExportEnvelope, ExportedAccount,
};
use crate::core::{Account, NewAccountParams, UpdateAccountParams};

/// How to handle a duplicate (an account with the same ID already exists).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateStrategy {
    /// Skip accounts that already exist.
    Skip,
    /// Update existing accounts with imported data.
    Update,
}

/// Options controlling what and how to import (FR-49, FR-50).
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Which account IDs to import. If empty, all accounts in the file are imported.
    pub account_ids: Vec<Uuid>,
    /// Which data categories to import. If empty, all available categories are imported.
    pub categories: Vec<ExportCategory>,
    /// How to handle duplicates (accounts with matching IDs).
    pub duplicate_strategy: DuplicateStrategy,
    /// Password for decrypting the import file (FR-48).
    pub password: Option<String>,
}

/// Result of importing a single account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountImportOutcome {
    /// Account was newly created.
    Created(Uuid),
    /// Account already existed and was updated.
    Updated(Uuid),
    /// Account already existed and was skipped.
    Skipped(Uuid),
    /// Account import failed with an error message.
    Failed(Uuid, String),
}

/// Overall result of an import operation.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Per-account outcomes.
    pub outcomes: Vec<AccountImportOutcome>,
}

impl ImportResult {
    pub fn created_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| matches!(o, AccountImportOutcome::Created(_)))
            .count()
    }

    pub fn updated_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| matches!(o, AccountImportOutcome::Updated(_)))
            .count()
    }

    pub fn skipped_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| matches!(o, AccountImportOutcome::Skipped(_)))
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| matches!(o, AccountImportOutcome::Failed(_, _)))
            .count()
    }
}

/// Errors that can occur during import.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("deserialization error: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("decryption error: {0}")]
    Decryption(String),
    #[error("password required for encrypted file")]
    PasswordRequired,
    #[error("no accounts in import file")]
    NoAccounts,
    #[error("unsupported format version: {0}")]
    UnsupportedVersion(u32),
}

/// Decrypt an encrypted envelope using the provided password (FR-48).
fn decrypt_payload(envelope: &EncryptedEnvelope, password: &str) -> Result<Vec<u8>, ImportError> {
    use aes_gcm::aead::Aead;
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use argon2::Argon2;
    use base64::Engine;

    let b64 = base64::engine::general_purpose::STANDARD;

    let salt = b64
        .decode(&envelope.salt)
        .map_err(|e| ImportError::Decryption(format!("invalid salt: {e}")))?;
    let nonce_bytes = b64
        .decode(&envelope.nonce)
        .map_err(|e| ImportError::Decryption(format!("invalid nonce: {e}")))?;
    let ciphertext = b64
        .decode(&envelope.ciphertext)
        .map_err(|e| ImportError::Decryption(format!("invalid ciphertext: {e}")))?;

    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|e| ImportError::Decryption(e.to_string()))?;

    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| ImportError::Decryption(e.to_string()))?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| ImportError::Decryption("decryption failed (wrong password?)".into()))?;

    Ok(plaintext)
}

/// Determine if the raw bytes represent an encrypted envelope.
fn is_encrypted(data: &[u8]) -> bool {
    // Try to parse as EncryptedEnvelope and check the `encrypted` flag.
    serde_json::from_slice::<EncryptedEnvelope>(data)
        .map(|e| e.encrypted)
        .unwrap_or(false)
}

/// Parse raw import file bytes into an ExportEnvelope, handling decryption if needed.
pub fn parse_import_data(
    data: &[u8],
    password: Option<&str>,
) -> Result<ExportEnvelope, ImportError> {
    if is_encrypted(data) {
        let encrypted: EncryptedEnvelope = serde_json::from_slice(data)?;
        let pw = password.ok_or(ImportError::PasswordRequired)?;
        let plaintext = decrypt_payload(&encrypted, pw)?;
        let envelope: ExportEnvelope = serde_json::from_slice(&plaintext)?;
        validate_version(envelope.format_version)?;
        Ok(envelope)
    } else {
        let envelope: ExportEnvelope = serde_json::from_slice(data)?;
        validate_version(envelope.format_version)?;
        Ok(envelope)
    }
}

fn validate_version(version: u32) -> Result<(), ImportError> {
    if version > 1 {
        return Err(ImportError::UnsupportedVersion(version));
    }
    Ok(())
}

/// Convert an ExportedAccount into NewAccountParams, applying category filtering.
/// Connection settings are always included for new accounts since they are required
/// for validation (host, username, credential must be non-empty).
fn exported_to_new_params(
    exported: &ExportedAccount,
    categories: &[ExportCategory],
) -> NewAccountParams {
    let include_all = categories.is_empty();
    // Connection settings are always needed for new accounts to pass validation.
    let include_conn = true;
    let include_sync = include_all || categories.contains(&ExportCategory::SyncSettings);
    let include_folders = include_all || categories.contains(&ExportCategory::FolderMappings);
    let include_security = include_all || categories.contains(&ExportCategory::SecuritySettings);
    let include_fetch = include_all || categories.contains(&ExportCategory::FetchSettings);

    NewAccountParams {
        display_name: exported.display_name.clone(),
        protocol: exported.protocol,
        host: if include_conn {
            exported.host.clone().unwrap_or_default()
        } else {
            String::new()
        },
        port: if include_conn {
            exported.port.unwrap_or(993)
        } else {
            993
        },
        encryption: if include_conn {
            exported
                .encryption
                .unwrap_or(crate::core::EncryptionMode::SslTls)
        } else {
            crate::core::EncryptionMode::SslTls
        },
        auth_method: if include_conn {
            exported
                .auth_method
                .unwrap_or(crate::core::AuthMethod::Plain)
        } else {
            crate::core::AuthMethod::Plain
        },
        username: if include_conn {
            exported.username.clone().unwrap_or_default()
        } else {
            String::new()
        },
        credential: if include_conn {
            exported.credential.clone().unwrap_or_default()
        } else {
            String::new()
        },
        smtp: if include_conn {
            exported.smtp.clone()
        } else {
            None
        },
        pop3_settings: if include_conn {
            exported.pop3_settings.clone()
        } else {
            None
        },
        color: exported.color,
        avatar_path: exported.avatar_path.clone(),
        category: exported.category.clone(),
        sync_enabled: if include_sync {
            exported.sync_enabled.unwrap_or(true)
        } else {
            true
        },
        on_demand: if include_sync {
            exported.on_demand.unwrap_or(false)
        } else {
            false
        },
        polling_interval_minutes: if include_sync {
            exported.polling_interval_minutes.unwrap_or(None)
        } else {
            None
        },
        unmetered_only: if include_sync {
            exported.unmetered_only.unwrap_or(false)
        } else {
            false
        },
        vpn_only: if include_sync {
            exported.vpn_only.unwrap_or(false)
        } else {
            false
        },
        schedule_exempt: if include_sync {
            exported.schedule_exempt.unwrap_or(false)
        } else {
            false
        },
        system_folders: if include_folders {
            exported.system_folders.clone()
        } else {
            None
        },
        swipe_defaults: if include_folders {
            exported.swipe_defaults.clone()
        } else {
            None
        },
        notifications_enabled: if include_sync {
            exported.notifications_enabled.unwrap_or(true)
        } else {
            true
        },
        security_settings: if include_security {
            exported.security_settings.clone()
        } else {
            None
        },
        fetch_settings: if include_fetch {
            exported.fetch_settings.clone()
        } else {
            None
        },
        keep_alive_settings: if include_fetch {
            exported.keep_alive_settings.clone()
        } else {
            None
        },
    }
}

/// Convert an ExportedAccount into UpdateAccountParams, applying category filtering.
/// Uses the existing account's values for categories not being imported.
fn exported_to_update_params(
    exported: &ExportedAccount,
    existing: &Account,
    categories: &[ExportCategory],
) -> UpdateAccountParams {
    let include_all = categories.is_empty();
    let include_conn = include_all || categories.contains(&ExportCategory::ConnectionSettings);
    let include_sync = include_all || categories.contains(&ExportCategory::SyncSettings);
    let include_folders = include_all || categories.contains(&ExportCategory::FolderMappings);
    let include_security = include_all || categories.contains(&ExportCategory::SecuritySettings);
    let include_fetch = include_all || categories.contains(&ExportCategory::FetchSettings);

    UpdateAccountParams {
        display_name: exported.display_name.clone(),
        protocol: exported.protocol,
        host: if include_conn {
            exported
                .host
                .clone()
                .unwrap_or_else(|| existing.host().to_string())
        } else {
            existing.host().to_string()
        },
        port: if include_conn {
            exported.port.unwrap_or_else(|| existing.port())
        } else {
            existing.port()
        },
        encryption: if include_conn {
            exported.encryption.unwrap_or_else(|| existing.encryption())
        } else {
            existing.encryption()
        },
        auth_method: if include_conn {
            exported
                .auth_method
                .unwrap_or_else(|| existing.auth_method())
        } else {
            existing.auth_method()
        },
        username: if include_conn {
            exported
                .username
                .clone()
                .unwrap_or_else(|| existing.username().to_string())
        } else {
            existing.username().to_string()
        },
        credential: if include_conn {
            exported
                .credential
                .clone()
                .unwrap_or_else(|| existing.credential().to_string())
        } else {
            existing.credential().to_string()
        },
        smtp: if include_conn {
            exported.smtp.clone()
        } else {
            existing.smtp().cloned()
        },
        pop3_settings: if include_conn {
            exported.pop3_settings.clone()
        } else {
            existing.pop3_settings().cloned()
        },
        color: exported.color.or(existing.color()),
        avatar_path: exported
            .avatar_path
            .clone()
            .or_else(|| existing.avatar_path().map(String::from)),
        category: exported
            .category
            .clone()
            .or_else(|| existing.category().map(String::from)),
        sync_enabled: if include_sync {
            exported
                .sync_enabled
                .unwrap_or_else(|| existing.sync_enabled())
        } else {
            existing.sync_enabled()
        },
        on_demand: if include_sync {
            exported.on_demand.unwrap_or_else(|| existing.on_demand())
        } else {
            existing.on_demand()
        },
        polling_interval_minutes: if include_sync {
            exported
                .polling_interval_minutes
                .unwrap_or_else(|| existing.polling_interval_minutes())
        } else {
            existing.polling_interval_minutes()
        },
        unmetered_only: if include_sync {
            exported
                .unmetered_only
                .unwrap_or_else(|| existing.unmetered_only())
        } else {
            existing.unmetered_only()
        },
        vpn_only: if include_sync {
            exported.vpn_only.unwrap_or_else(|| existing.vpn_only())
        } else {
            existing.vpn_only()
        },
        schedule_exempt: if include_sync {
            exported
                .schedule_exempt
                .unwrap_or_else(|| existing.schedule_exempt())
        } else {
            existing.schedule_exempt()
        },
        system_folders: if include_folders {
            exported.system_folders.clone()
        } else {
            existing.system_folders().cloned()
        },
        swipe_defaults: if include_folders {
            exported.swipe_defaults.clone()
        } else {
            existing.swipe_defaults().cloned()
        },
        notifications_enabled: if include_sync {
            exported
                .notifications_enabled
                .unwrap_or_else(|| existing.notifications_enabled())
        } else {
            existing.notifications_enabled()
        },
        security_settings: if include_security {
            exported.security_settings.clone()
        } else {
            existing.security_settings().cloned()
        },
        fetch_settings: if include_fetch {
            exported.fetch_settings.clone()
        } else {
            existing.fetch_settings().cloned()
        },
        keep_alive_settings: if include_fetch {
            exported.keep_alive_settings.clone()
        } else {
            existing.keep_alive_settings().cloned()
        },
    }
}

/// Import accounts from a parsed envelope into the existing account list (FR-49, FR-50, AC-15).
///
/// This function mutates `existing_accounts` in place: creating new accounts or updating
/// existing ones based on duplicate detection by UUID.
pub fn import_accounts(
    existing_accounts: &mut Vec<Account>,
    envelope: &ExportEnvelope,
    options: &ImportOptions,
) -> ImportResult {
    let accounts_to_import: Vec<&ExportedAccount> = if options.account_ids.is_empty() {
        envelope.accounts.iter().collect()
    } else {
        envelope
            .accounts
            .iter()
            .filter(|a| options.account_ids.contains(&a.id))
            .collect()
    };

    if accounts_to_import.is_empty() {
        return ImportResult { outcomes: vec![] };
    }

    let mut outcomes = Vec::new();

    for exported in &accounts_to_import {
        let existing_idx = existing_accounts.iter().position(|a| a.id() == exported.id);

        match existing_idx {
            Some(idx) => {
                // Duplicate detected
                match options.duplicate_strategy {
                    DuplicateStrategy::Skip => {
                        outcomes.push(AccountImportOutcome::Skipped(exported.id));
                    }
                    DuplicateStrategy::Update => {
                        let update_params = exported_to_update_params(
                            exported,
                            &existing_accounts[idx],
                            &options.categories,
                        );
                        match existing_accounts[idx].update(update_params) {
                            Ok(()) => {
                                outcomes.push(AccountImportOutcome::Updated(exported.id));
                            }
                            Err(e) => {
                                outcomes
                                    .push(AccountImportOutcome::Failed(exported.id, e.to_string()));
                            }
                        }
                    }
                }
            }
            None => {
                // New account
                let params = exported_to_new_params(exported, &options.categories);
                match Account::new_with_id(exported.id, params) {
                    Ok(account) => {
                        existing_accounts.push(account);
                        outcomes.push(AccountImportOutcome::Created(exported.id));
                    }
                    Err(e) => {
                        outcomes.push(AccountImportOutcome::Failed(exported.id, e.to_string()));
                    }
                }
            }
        }
    }

    ImportResult { outcomes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::export_accounts::{export_accounts, ExportOptions};
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

    fn export_and_parse(accounts: &[Account], password: Option<&str>) -> ExportEnvelope {
        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: password.map(String::from),
        };
        let data = export_accounts(accounts, &options).unwrap();
        parse_import_data(&data, password).unwrap()
    }

    #[test]
    fn import_new_accounts() {
        let acct1 = make_test_account("Account 1");
        let acct2 = make_test_account("Account 2");
        let id1 = acct1.id();
        let id2 = acct2.id();

        let envelope = export_and_parse(&[acct1, acct2], None);

        let mut existing: Vec<Account> = vec![];
        let options = ImportOptions {
            account_ids: vec![],
            categories: vec![],
            duplicate_strategy: DuplicateStrategy::Skip,
            password: None,
        };

        let result = import_accounts(&mut existing, &envelope, &options);
        assert_eq!(result.created_count(), 2);
        assert_eq!(existing.len(), 2);
        assert_eq!(existing[0].id(), id1);
        assert_eq!(existing[1].id(), id2);
    }

    #[test]
    fn import_preserves_all_settings() {
        let acct = make_test_account("Full");
        let original_id = acct.id();

        let envelope = export_and_parse(&[acct], None);

        let mut existing: Vec<Account> = vec![];
        let options = ImportOptions {
            account_ids: vec![],
            categories: vec![],
            duplicate_strategy: DuplicateStrategy::Skip,
            password: None,
        };

        import_accounts(&mut existing, &envelope, &options);
        let imported = &existing[0];

        assert_eq!(imported.id(), original_id);
        assert_eq!(imported.display_name(), "Full");
        assert_eq!(imported.protocol(), Protocol::Imap);
        assert_eq!(imported.host(), "imap.example.com");
        assert_eq!(imported.port(), 993);
        assert_eq!(imported.encryption(), EncryptionMode::SslTls);
        assert_eq!(imported.auth_method(), AuthMethod::Plain);
        assert_eq!(imported.username(), "user@example.com");
        assert_eq!(imported.credential(), "secret");
        assert!(imported.smtp().is_some());
        assert!(imported.color().is_some());
        assert_eq!(imported.category(), Some("Work"));
        assert!(imported.sync_enabled());
        assert!(!imported.on_demand());
        assert_eq!(imported.polling_interval_minutes(), Some(15));
        assert!(imported.unmetered_only());
        assert!(!imported.vpn_only());
        assert!(!imported.schedule_exempt());
        assert!(imported.system_folders().is_some());
        assert!(imported.swipe_defaults().is_some());
        assert!(imported.notifications_enabled());
        assert!(imported.security_settings().is_some());
        assert!(imported.fetch_settings().is_some());
        assert!(imported.keep_alive_settings().is_some());
    }

    #[test]
    fn import_duplicate_skip() {
        let acct = make_test_account("Existing");
        let id = acct.id();

        let envelope = export_and_parse(std::slice::from_ref(&acct), None);

        let mut existing = vec![acct];
        let options = ImportOptions {
            account_ids: vec![],
            categories: vec![],
            duplicate_strategy: DuplicateStrategy::Skip,
            password: None,
        };

        let result = import_accounts(&mut existing, &envelope, &options);
        assert_eq!(result.skipped_count(), 1);
        assert_eq!(result.outcomes[0], AccountImportOutcome::Skipped(id));
        assert_eq!(existing.len(), 1);
    }

    #[test]
    fn import_duplicate_update() {
        let mut acct = make_test_account("Original Name");
        let id = acct.id();
        // Change name in the exported version
        acct.update(UpdateAccountParams {
            display_name: "Updated Name".into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            smtp: None,
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
        })
        .unwrap();

        let envelope = export_and_parse(&[acct], None);

        // Original account with old name
        let original = make_test_account("Original Name");
        let original = Account::new_with_id(id, original.to_new_account_params()).unwrap();
        let mut existing = vec![original];

        let options = ImportOptions {
            account_ids: vec![],
            categories: vec![],
            duplicate_strategy: DuplicateStrategy::Update,
            password: None,
        };

        let result = import_accounts(&mut existing, &envelope, &options);
        assert_eq!(result.updated_count(), 1);
        assert_eq!(existing[0].display_name(), "Updated Name");
    }

    #[test]
    fn import_selective_accounts() {
        let acct1 = make_test_account("Account 1");
        let acct2 = make_test_account("Account 2");
        let acct3 = make_test_account("Account 3");
        let id2 = acct2.id();

        let envelope = export_and_parse(&[acct1, acct2, acct3], None);

        let mut existing: Vec<Account> = vec![];
        let options = ImportOptions {
            account_ids: vec![id2],
            categories: vec![],
            duplicate_strategy: DuplicateStrategy::Skip,
            password: None,
        };

        let result = import_accounts(&mut existing, &envelope, &options);
        assert_eq!(result.created_count(), 1);
        assert_eq!(existing.len(), 1);
        assert_eq!(existing[0].id(), id2);
    }

    #[test]
    fn import_selective_categories() {
        let acct = make_test_account("Test");
        let id = acct.id();

        // Export with all categories
        let envelope = export_and_parse(&[acct], None);

        let mut existing: Vec<Account> = vec![];
        // Only import sync settings
        let options = ImportOptions {
            account_ids: vec![],
            categories: vec![ExportCategory::SyncSettings],
            duplicate_strategy: DuplicateStrategy::Skip,
            password: None,
        };

        let result = import_accounts(&mut existing, &envelope, &options);
        assert_eq!(result.created_count(), 1);
        let imported = &existing[0];
        assert_eq!(imported.id(), id);
        // Sync settings should be imported
        assert!(imported.sync_enabled());
        assert!(imported.unmetered_only());
        // Connection settings are always included for new accounts (required for validation)
        assert_eq!(imported.host(), "imap.example.com");
    }

    #[test]
    fn import_encrypted_file() {
        let acct = make_test_account("Encrypted");
        let id = acct.id();
        let password = "test-password-123";

        let export_options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: Some(password.into()),
        };
        let data = export_accounts(&[acct], &export_options).unwrap();

        let envelope = parse_import_data(&data, Some(password)).unwrap();
        assert_eq!(envelope.accounts.len(), 1);
        assert_eq!(envelope.accounts[0].id, id);
    }

    #[test]
    fn import_encrypted_wrong_password() {
        let acct = make_test_account("Encrypted");

        let export_options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: Some("correct-password".into()),
        };
        let data = export_accounts(&[acct], &export_options).unwrap();

        let result = parse_import_data(&data, Some("wrong-password"));
        assert!(matches!(result, Err(ImportError::Decryption(_))));
    }

    #[test]
    fn import_encrypted_no_password() {
        let acct = make_test_account("Encrypted");

        let export_options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: Some("password".into()),
        };
        let data = export_accounts(&[acct], &export_options).unwrap();

        let result = parse_import_data(&data, None);
        assert!(matches!(result, Err(ImportError::PasswordRequired)));
    }

    #[test]
    fn import_empty_file() {
        let envelope = ExportEnvelope {
            format_version: 1,
            exported_at: "2026-01-01T00:00:00Z".into(),
            accounts: vec![],
        };

        let mut existing: Vec<Account> = vec![];
        let options = ImportOptions {
            account_ids: vec![],
            categories: vec![],
            duplicate_strategy: DuplicateStrategy::Skip,
            password: None,
        };

        let result = import_accounts(&mut existing, &envelope, &options);
        assert_eq!(result.outcomes.len(), 0);
    }

    #[test]
    fn import_mixed_new_and_duplicate() {
        let acct1 = make_test_account("Existing");
        let acct2 = make_test_account("New");
        let id1 = acct1.id();
        let id2 = acct2.id();

        let envelope = export_and_parse(&[acct1.clone(), acct2], None);

        let mut existing = vec![acct1];
        let options = ImportOptions {
            account_ids: vec![],
            categories: vec![],
            duplicate_strategy: DuplicateStrategy::Skip,
            password: None,
        };

        let result = import_accounts(&mut existing, &envelope, &options);
        assert_eq!(result.skipped_count(), 1);
        assert_eq!(result.created_count(), 1);
        assert_eq!(existing.len(), 2);
        assert_eq!(existing[0].id(), id1);
        assert_eq!(existing[1].id(), id2);
    }

    #[test]
    fn parse_import_data_plain() {
        let acct = make_test_account("Plain");

        let export_options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };
        let data = export_accounts(std::slice::from_ref(&acct), &export_options).unwrap();

        let envelope = parse_import_data(&data, None).unwrap();
        assert_eq!(envelope.format_version, 1);
        assert_eq!(envelope.accounts.len(), 1);
        assert_eq!(envelope.accounts[0].id, acct.id());
    }

    #[test]
    fn import_result_counts() {
        let result = ImportResult {
            outcomes: vec![
                AccountImportOutcome::Created(Uuid::new_v4()),
                AccountImportOutcome::Created(Uuid::new_v4()),
                AccountImportOutcome::Skipped(Uuid::new_v4()),
                AccountImportOutcome::Updated(Uuid::new_v4()),
                AccountImportOutcome::Failed(Uuid::new_v4(), "error".into()),
            ],
        };
        assert_eq!(result.created_count(), 2);
        assert_eq!(result.skipped_count(), 1);
        assert_eq!(result.updated_count(), 1);
        assert_eq!(result.failed_count(), 1);
    }

    #[test]
    fn import_update_selective_categories_preserves_other_fields() {
        let acct = make_test_account("Original");
        let id = acct.id();

        // Export only sync settings
        let export_options = ExportOptions {
            account_ids: vec![],
            categories: vec![ExportCategory::SyncSettings],
            password: None,
        };
        let data = export_accounts(std::slice::from_ref(&acct), &export_options).unwrap();
        let envelope: ExportEnvelope = serde_json::from_slice(&data).unwrap();

        let mut existing = vec![acct];
        let options = ImportOptions {
            account_ids: vec![],
            categories: vec![ExportCategory::SyncSettings],
            duplicate_strategy: DuplicateStrategy::Update,
            password: None,
        };

        let result = import_accounts(&mut existing, &envelope, &options);
        assert_eq!(result.updated_count(), 1);
        // Connection settings should be preserved from existing
        assert_eq!(existing[0].id(), id);
        assert_eq!(existing[0].host(), "imap.example.com");
        assert_eq!(existing[0].port(), 993);
    }
}
