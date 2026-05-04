use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::export_accounts::{ExportError, ExportOptions};
use crate::core::Account;

/// Errors from the export persistence layer.
#[derive(Debug, thiserror::Error)]
pub enum ExportServiceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("export error: {0}")]
    Export(#[from] ExportError),
}

/// Result type indicating export success with the output path.
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// The path where the export file was written.
    pub path: PathBuf,
    /// Number of accounts exported.
    pub account_count: usize,
    /// Whether the export is encrypted.
    pub encrypted: bool,
}

/// Export account configurations to a file on disk (FR-47).
///
/// The file is written atomically: data goes to a temporary file first,
/// then is renamed to the target path.
pub fn export_to_file(
    accounts: &[Account],
    options: &ExportOptions,
    output_path: &Path,
) -> Result<ExportResult, ExportServiceError> {
    let data = crate::core::export_accounts::export_accounts(accounts, options)?;

    let account_count = if options.account_ids.is_empty() {
        accounts.len()
    } else {
        accounts
            .iter()
            .filter(|a| options.account_ids.contains(&a.id()))
            .count()
    };

    // Write atomically via temp file + rename.
    let dir = output_path.parent().unwrap_or_else(|| Path::new("."));
    let tmp_path = dir.join(".fairmail-export.tmp");
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(&data)?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, output_path)?;

    Ok(ExportResult {
        path: output_path.to_path_buf(),
        account_count,
        encrypted: options.password.as_ref().is_some_and(|p| !p.is_empty()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AuthMethod, EncryptionMode, NewAccountParams, Protocol};

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
    fn export_to_file_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        let acct = make_account("Test");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };

        let result = export_to_file(&[acct], &options, &path).unwrap();
        assert_eq!(result.path, path);
        assert_eq!(result.account_count, 1);
        assert!(!result.encrypted);
        assert!(path.exists());
    }

    #[test]
    fn export_to_file_content_is_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        let acct = make_account("JSON Test");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };

        export_to_file(&[acct], &options, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let envelope: crate::core::export_accounts::ExportEnvelope =
            serde_json::from_str(&content).unwrap();
        assert_eq!(envelope.accounts.len(), 1);
        assert_eq!(envelope.accounts[0].display_name, "JSON Test");
    }

    #[test]
    fn export_to_file_encrypted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        let acct = make_account("Encrypted");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: Some("password123".into()),
        };

        let result = export_to_file(&[acct], &options, &path).unwrap();
        assert!(result.encrypted);

        let content = std::fs::read_to_string(&path).unwrap();
        let encrypted: crate::core::export_accounts::EncryptedEnvelope =
            serde_json::from_str(&content).unwrap();
        assert!(encrypted.encrypted);
    }

    #[test]
    fn export_to_file_atomic_no_tmp_left() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        let acct = make_account("Atomic");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };

        export_to_file(&[acct], &options, &path).unwrap();
        assert!(!dir.path().join(".fairmail-export.tmp").exists());
    }

    #[test]
    fn export_to_file_multiple_accounts() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        let accounts: Vec<Account> = (0..5).map(|i| make_account(&format!("Acct {i}"))).collect();

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };

        let result = export_to_file(&accounts, &options, &path).unwrap();
        assert_eq!(result.account_count, 5);
    }

    #[test]
    fn export_to_file_empty_accounts_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");

        let options = ExportOptions {
            account_ids: vec![],
            categories: vec![],
            password: None,
        };

        let result = export_to_file(&[], &options, &path);
        assert!(result.is_err());
    }
}
