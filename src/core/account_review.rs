use crate::core::account::{FolderRole, SystemFolders};

/// A system folder entry for the review screen, indicating whether it was found on the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewFolderEntry {
    /// The system folder role (e.g. Drafts, Sent, Trash).
    pub role: FolderRole,
    /// The folder name on the server, if detected.
    pub server_name: Option<String>,
}

impl ReviewFolderEntry {
    /// Whether this folder was found on the server.
    pub fn is_detected(&self) -> bool {
        self.server_name.is_some()
    }

    /// Human-readable label for the role.
    pub fn role_label(&self) -> &'static str {
        match self.role {
            FolderRole::Drafts => "Drafts",
            FolderRole::Sent => "Sent",
            FolderRole::Archive => "Archive",
            FolderRole::Trash => "Trash",
            FolderRole::Junk => "Spam",
        }
    }
}

/// Data model for the account review screen (FR-26).
///
/// Presented after successful IMAP and SMTP connectivity checks so the user
/// can confirm detected configuration before saving.
#[derive(Debug, Clone)]
pub struct AccountReviewData {
    /// The provider display name (FR-26a).
    pub provider_name: String,
    /// The account name, initially derived from email (FR-26a, FR-26c).
    pub account_name: String,
    /// Whether the Inbox folder was found on the server.
    pub has_inbox: bool,
    /// System folder detection results (FR-26b).
    pub folder_entries: Vec<ReviewFolderEntry>,
}

/// Build review data from successful connectivity check results.
///
/// `provider_name` is the display name of the detected provider.
/// `account_name` is the initial account name (typically the email address).
/// `has_inbox` indicates whether INBOX was found.
/// `system_folders` contains the detected system folder assignments from the IMAP check.
pub fn build_review_data(
    provider_name: &str,
    account_name: &str,
    has_inbox: bool,
    system_folders: &SystemFolders,
) -> AccountReviewData {
    let folder_entries = FolderRole::all()
        .iter()
        .map(|&role| ReviewFolderEntry {
            role,
            server_name: system_folders.get(role).map(|s| s.to_string()),
        })
        .collect();

    AccountReviewData {
        provider_name: provider_name.to_string(),
        account_name: account_name.to_string(),
        has_inbox,
        folder_entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_review_data_all_folders_detected() {
        let system_folders = SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: Some("Archive".to_string()),
            trash: Some("Trash".to_string()),
            junk: Some("Spam".to_string()),
        };

        let data = build_review_data("Gmail", "user@gmail.com", true, &system_folders);

        assert_eq!(data.provider_name, "Gmail");
        assert_eq!(data.account_name, "user@gmail.com");
        assert!(data.has_inbox);
        assert_eq!(data.folder_entries.len(), 5);
        assert!(data.folder_entries.iter().all(|e| e.is_detected()));
    }

    #[test]
    fn build_review_data_partial_folders() {
        let system_folders = SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: None,
            trash: Some("Trash".to_string()),
            junk: None,
        };

        let data = build_review_data("Fastmail", "user@fastmail.com", true, &system_folders);

        assert_eq!(data.folder_entries.len(), 5);
        let detected_count = data
            .folder_entries
            .iter()
            .filter(|e| e.is_detected())
            .count();
        assert_eq!(detected_count, 3);

        let archive_entry = data
            .folder_entries
            .iter()
            .find(|e| e.role == FolderRole::Archive)
            .unwrap();
        assert!(!archive_entry.is_detected());
    }

    #[test]
    fn build_review_data_no_folders() {
        let system_folders = SystemFolders::default();
        let data = build_review_data("Unknown", "user@example.com", false, &system_folders);

        assert!(!data.has_inbox);
        assert!(data.folder_entries.iter().all(|e| !e.is_detected()));
    }

    #[test]
    fn role_label_values() {
        let entry = ReviewFolderEntry {
            role: FolderRole::Junk,
            server_name: None,
        };
        assert_eq!(entry.role_label(), "Spam");

        let entry = ReviewFolderEntry {
            role: FolderRole::Drafts,
            server_name: Some("Drafts".to_string()),
        };
        assert_eq!(entry.role_label(), "Drafts");
    }
}
