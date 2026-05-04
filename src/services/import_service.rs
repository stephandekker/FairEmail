use std::path::Path;

use crate::core::import_accounts::{
    import_accounts, parse_import_data, ImportError, ImportOptions, ImportResult,
};
use crate::core::Account;

/// Errors from the import persistence layer.
#[derive(Debug, thiserror::Error)]
pub enum ImportServiceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("import error: {0}")]
    Import(#[from] ImportError),
}

/// Read an import file and return the parsed envelope for previewing accounts
/// before the user commits to importing.
pub fn read_import_file(
    path: &Path,
    password: Option<&str>,
) -> Result<crate::core::export_accounts::ExportEnvelope, ImportServiceError> {
    let data = std::fs::read(path)?;
    let envelope = parse_import_data(&data, password)?;
    Ok(envelope)
}

/// Check whether an import file is encrypted.
pub fn is_file_encrypted(path: &Path) -> Result<bool, ImportServiceError> {
    let data = std::fs::read(path)?;
    let encrypted =
        serde_json::from_slice::<crate::core::export_accounts::EncryptedEnvelope>(&data)
            .map(|e| e.encrypted)
            .unwrap_or(false);
    Ok(encrypted)
}

/// Import accounts from a file on disk (FR-49).
///
/// Reads the file, decrypts if needed, applies duplicate detection and category filtering,
/// and mutates `existing_accounts` in place.
pub fn import_from_file(
    existing_accounts: &mut Vec<Account>,
    path: &Path,
    options: &ImportOptions,
) -> Result<ImportResult, ImportServiceError> {
    let data = std::fs::read(path)?;
    let envelope = parse_import_data(&data, options.password.as_deref())?;
    let result = import_accounts(existing_accounts, &envelope, options);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::export_accounts::ExportOptions;
    use crate::core::import_accounts::DuplicateStrategy;
    use crate::core::{AuthMethod, EncryptionMode, NewAccountParams, Protocol};
    use crate::services::export_service::export_to_file;

    fn make_account(name: &str) -> Account {
        Account::new(NewAccountParams {
            display_name: name.into(),
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
        .unwrap()
    }

    #[test]
    fn import_from_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        let acct = make_account("Roundtrip");
        let id = acct.id();

        let export_options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };
        export_to_file(&[acct], &export_options, &path).unwrap();

        let mut existing: Vec<Account> = vec![];
        let import_options = ImportOptions {
            account_ids: vec![],
            categories: vec![],
            duplicate_strategy: DuplicateStrategy::Skip,
            password: None,
        };

        let result = import_from_file(&mut existing, &path, &import_options).unwrap();
        assert_eq!(result.created_count(), 1);
        assert_eq!(existing.len(), 1);
        assert_eq!(existing[0].id(), id);
        assert_eq!(existing[0].display_name(), "Roundtrip");
    }

    #[test]
    fn import_from_file_encrypted_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        let acct = make_account("Encrypted");
        let id = acct.id();
        let password = "test-pw-456";

        let export_options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: Some(password.into()),
        };
        export_to_file(&[acct], &export_options, &path).unwrap();

        assert!(is_file_encrypted(&path).unwrap());

        let mut existing: Vec<Account> = vec![];
        let import_options = ImportOptions {
            account_ids: vec![],
            categories: vec![],
            duplicate_strategy: DuplicateStrategy::Skip,
            password: Some(password.into()),
        };

        let result = import_from_file(&mut existing, &path, &import_options).unwrap();
        assert_eq!(result.created_count(), 1);
        assert_eq!(existing[0].id(), id);
    }

    #[test]
    fn import_from_file_not_found() {
        let mut existing: Vec<Account> = vec![];
        let options = ImportOptions {
            account_ids: vec![],
            categories: vec![],
            duplicate_strategy: DuplicateStrategy::Skip,
            password: None,
        };

        let result = import_from_file(&mut existing, Path::new("/nonexistent/file.json"), &options);
        assert!(matches!(result, Err(ImportServiceError::Io(_))));
    }

    #[test]
    fn is_file_encrypted_plain() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("plain.json");
        let acct = make_account("Plain");

        let export_options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };
        export_to_file(&[acct], &export_options, &path).unwrap();

        assert!(!is_file_encrypted(&path).unwrap());
    }

    #[test]
    fn read_import_file_preview() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        let acct1 = make_account("First");
        let acct2 = make_account("Second");

        let export_options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };
        export_to_file(&[acct1, acct2], &export_options, &path).unwrap();

        let envelope = read_import_file(&path, None).unwrap();
        assert_eq!(envelope.accounts.len(), 2);
        assert_eq!(envelope.accounts[0].display_name, "First");
        assert_eq!(envelope.accounts[1].display_name, "Second");
    }
}
