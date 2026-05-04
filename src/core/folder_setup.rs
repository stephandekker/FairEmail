use crate::core::account::{FolderRole, SystemFolders};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Synchronization mode for a folder (FR-30).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMode {
    /// Synchronize folder list and download message bodies.
    SyncAndDownload,
    /// Synchronize folder list only (no body download).
    SyncOnly,
    /// Do not synchronize this folder.
    NoSync,
}

/// Push/notification mode for a folder (FR-30a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PushMode {
    /// Use IMAP IDLE or push notifications if supported.
    IdleIfSupported,
    /// Poll periodically only.
    Poll,
    /// No push or polling.
    None,
}

/// Default sync settings for a folder (FR-30).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderSyncConfig {
    /// The folder name on the server.
    pub folder_name: String,
    /// The synchronization mode.
    pub sync_mode: SyncMode,
    /// The push/notification mode.
    pub push_mode: PushMode,
}

/// The standard system folder names used when creating missing folders (FR-29).
/// These are the default English names; future iterations may allow configuration (OQ-8).
const DEFAULT_DRAFTS_NAME: &str = "Drafts";
const DEFAULT_SENT_NAME: &str = "Sent";
const DEFAULT_ARCHIVE_NAME: &str = "Archive";
const DEFAULT_TRASH_NAME: &str = "Trash";
const DEFAULT_SPAM_NAME: &str = "Spam";

/// Returns the default folder name for a given role when it needs to be created.
pub fn default_folder_name(role: FolderRole) -> &'static str {
    match role {
        FolderRole::Drafts => DEFAULT_DRAFTS_NAME,
        FolderRole::Sent => DEFAULT_SENT_NAME,
        FolderRole::Archive => DEFAULT_ARCHIVE_NAME,
        FolderRole::Trash => DEFAULT_TRASH_NAME,
        FolderRole::Junk => DEFAULT_SPAM_NAME,
    }
}

/// Determine which system folders are missing and need to be created (FR-29).
///
/// Returns a list of `(FolderRole, folder_name)` pairs for folders that don't exist
/// in the current `SystemFolders`. The folder name uses the namespace separator
/// if provided (Design Note N-6).
pub fn find_missing_system_folders(
    system_folders: &SystemFolders,
    namespace_prefix: &str,
) -> Vec<(FolderRole, String)> {
    let mut missing = Vec::new();

    for role in FolderRole::all() {
        if system_folders.get(*role).is_none() {
            let name = format!("{}{}", namespace_prefix, default_folder_name(*role));
            missing.push((*role, name));
        }
    }

    missing
}

/// After creating missing folders, produce the updated SystemFolders with all roles assigned.
pub fn complete_system_folders(
    existing: &SystemFolders,
    created: &[(FolderRole, String)],
) -> SystemFolders {
    let mut result = existing.clone();
    for (role, name) in created {
        if result.get(*role).is_none() {
            result.set(*role, Some(name.clone()));
        }
    }
    result
}

/// Build the default sync configuration for all folders of a newly created account (FR-30).
///
/// - Inbox: sync + download + push/idle (FR-30a)
/// - Drafts, Sent, Archive: sync + download, polled (FR-30b)
/// - Trash, Spam/Junk: sync only (no body download), polled (FR-30c)
/// - Other (user-created) folders: no sync (FR-30d)
pub fn build_default_sync_configs(
    system_folders: &SystemFolders,
    all_folder_names: &[String],
) -> Vec<FolderSyncConfig> {
    let mut configs = Vec::new();

    // Inbox always gets sync + download + idle
    configs.push(FolderSyncConfig {
        folder_name: "INBOX".to_string(),
        sync_mode: SyncMode::SyncAndDownload,
        push_mode: PushMode::IdleIfSupported,
    });

    // Build a set of known system folder names for quick lookup
    let system_folder_names: Vec<&str> = FolderRole::all()
        .iter()
        .filter_map(|role| system_folders.get(*role))
        .collect();

    for folder_name in all_folder_names {
        // Skip inbox (already handled)
        if folder_name.eq_ignore_ascii_case("INBOX") {
            continue;
        }

        // Determine the role of this folder
        let role = FolderRole::all()
            .iter()
            .find(|r| system_folders.get(**r) == Some(folder_name.as_str()));

        match role {
            Some(FolderRole::Drafts) | Some(FolderRole::Sent) | Some(FolderRole::Archive) => {
                // FR-30b: sync + download, polled
                configs.push(FolderSyncConfig {
                    folder_name: folder_name.clone(),
                    sync_mode: SyncMode::SyncAndDownload,
                    push_mode: PushMode::Poll,
                });
            }
            Some(FolderRole::Trash) | Some(FolderRole::Junk) => {
                // FR-30c: sync only (no body download), polled
                configs.push(FolderSyncConfig {
                    folder_name: folder_name.clone(),
                    sync_mode: SyncMode::SyncOnly,
                    push_mode: PushMode::Poll,
                });
            }
            None => {
                // FR-30d: user-created folders do not synchronize by default
                // Only if it's not a known system folder (safety check)
                if !system_folder_names.contains(&folder_name.as_str()) {
                    configs.push(FolderSyncConfig {
                        folder_name: folder_name.clone(),
                        sync_mode: SyncMode::NoSync,
                        push_mode: PushMode::None,
                    });
                }
            }
        }
    }

    configs
}

/// The result of the post-account-creation folder setup (FR-29, FR-30, FR-31).
#[derive(Debug, Clone)]
pub struct FolderSetupResult {
    /// The account ID this setup was performed for.
    pub account_id: Uuid,
    /// Folders that were created on the server (FR-29).
    pub created_folders: Vec<(FolderRole, String)>,
    /// The complete system folder assignments after creation.
    pub system_folders: SystemFolders,
    /// The sync configuration applied to all folders (FR-30).
    pub sync_configs: Vec<FolderSyncConfig>,
    /// Whether an immediate sync was triggered (FR-31).
    pub sync_triggered: bool,
}

/// Errors that can occur during folder setup.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FolderSetupError {
    #[error("failed to create folder '{0}' on server: {1}")]
    FolderCreationFailed(String, String),
    #[error("failed to trigger sync: {0}")]
    SyncTriggerFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_all_missing_folders_when_none_assigned() {
        let system_folders = SystemFolders::default();
        let missing = find_missing_system_folders(&system_folders, "");

        assert_eq!(missing.len(), 5);
        assert!(missing.contains(&(FolderRole::Drafts, "Drafts".to_string())));
        assert!(missing.contains(&(FolderRole::Sent, "Sent".to_string())));
        assert!(missing.contains(&(FolderRole::Archive, "Archive".to_string())));
        assert!(missing.contains(&(FolderRole::Trash, "Trash".to_string())));
        assert!(missing.contains(&(FolderRole::Junk, "Spam".to_string())));
    }

    #[test]
    fn finds_no_missing_folders_when_all_assigned() {
        let system_folders = SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: Some("Archive".to_string()),
            trash: Some("Trash".to_string()),
            junk: Some("Junk".to_string()),
        };
        let missing = find_missing_system_folders(&system_folders, "");
        assert!(missing.is_empty());
    }

    #[test]
    fn finds_partially_missing_folders() {
        let system_folders = SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: None,
            trash: Some("Trash".to_string()),
            junk: None,
        };
        let missing = find_missing_system_folders(&system_folders, "");
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&(FolderRole::Archive, "Archive".to_string())));
        assert!(missing.contains(&(FolderRole::Junk, "Spam".to_string())));
    }

    #[test]
    fn applies_namespace_prefix_to_created_folders() {
        let system_folders = SystemFolders::default();
        let missing = find_missing_system_folders(&system_folders, "INBOX.");

        assert!(missing.contains(&(FolderRole::Drafts, "INBOX.Drafts".to_string())));
        assert!(missing.contains(&(FolderRole::Sent, "INBOX.Sent".to_string())));
        assert!(missing.contains(&(FolderRole::Archive, "INBOX.Archive".to_string())));
        assert!(missing.contains(&(FolderRole::Trash, "INBOX.Trash".to_string())));
        assert!(missing.contains(&(FolderRole::Junk, "INBOX.Spam".to_string())));
    }

    #[test]
    fn complete_system_folders_fills_gaps() {
        let existing = SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: None,
            archive: None,
            trash: Some("Trash".to_string()),
            junk: None,
        };
        let created = vec![
            (FolderRole::Sent, "Sent".to_string()),
            (FolderRole::Archive, "Archive".to_string()),
            (FolderRole::Junk, "Spam".to_string()),
        ];
        let result = complete_system_folders(&existing, &created);

        assert_eq!(result.drafts.as_deref(), Some("Drafts"));
        assert_eq!(result.sent.as_deref(), Some("Sent"));
        assert_eq!(result.archive.as_deref(), Some("Archive"));
        assert_eq!(result.trash.as_deref(), Some("Trash"));
        assert_eq!(result.junk.as_deref(), Some("Spam"));
    }

    #[test]
    fn complete_system_folders_does_not_overwrite_existing() {
        let existing = SystemFolders {
            drafts: Some("MyDrafts".to_string()),
            sent: Some("MySent".to_string()),
            archive: Some("MyArchive".to_string()),
            trash: Some("MyTrash".to_string()),
            junk: Some("MyJunk".to_string()),
        };
        let created = vec![
            (FolderRole::Drafts, "Drafts".to_string()),
            (FolderRole::Sent, "Sent".to_string()),
        ];
        let result = complete_system_folders(&existing, &created);

        // Existing names are preserved
        assert_eq!(result.drafts.as_deref(), Some("MyDrafts"));
        assert_eq!(result.sent.as_deref(), Some("MySent"));
    }

    #[test]
    fn inbox_gets_sync_download_and_idle() {
        let system_folders = SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: Some("Archive".to_string()),
            trash: Some("Trash".to_string()),
            junk: Some("Spam".to_string()),
        };
        let all_folders = vec![
            "INBOX".to_string(),
            "Drafts".to_string(),
            "Sent".to_string(),
            "Archive".to_string(),
            "Trash".to_string(),
            "Spam".to_string(),
        ];
        let configs = build_default_sync_configs(&system_folders, &all_folders);

        let inbox = configs.iter().find(|c| c.folder_name == "INBOX").unwrap();
        assert_eq!(inbox.sync_mode, SyncMode::SyncAndDownload);
        assert_eq!(inbox.push_mode, PushMode::IdleIfSupported);
    }

    #[test]
    fn drafts_sent_archive_get_sync_download_polled() {
        let system_folders = SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: Some("Archive".to_string()),
            trash: Some("Trash".to_string()),
            junk: Some("Spam".to_string()),
        };
        let all_folders = vec![
            "INBOX".to_string(),
            "Drafts".to_string(),
            "Sent".to_string(),
            "Archive".to_string(),
            "Trash".to_string(),
            "Spam".to_string(),
        ];
        let configs = build_default_sync_configs(&system_folders, &all_folders);

        for name in &["Drafts", "Sent", "Archive"] {
            let cfg = configs.iter().find(|c| c.folder_name == *name).unwrap();
            assert_eq!(cfg.sync_mode, SyncMode::SyncAndDownload);
            assert_eq!(cfg.push_mode, PushMode::Poll);
        }
    }

    #[test]
    fn trash_spam_get_sync_only_polled() {
        let system_folders = SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: Some("Archive".to_string()),
            trash: Some("Trash".to_string()),
            junk: Some("Spam".to_string()),
        };
        let all_folders = vec![
            "INBOX".to_string(),
            "Drafts".to_string(),
            "Sent".to_string(),
            "Archive".to_string(),
            "Trash".to_string(),
            "Spam".to_string(),
        ];
        let configs = build_default_sync_configs(&system_folders, &all_folders);

        for name in &["Trash", "Spam"] {
            let cfg = configs.iter().find(|c| c.folder_name == *name).unwrap();
            assert_eq!(cfg.sync_mode, SyncMode::SyncOnly);
            assert_eq!(cfg.push_mode, PushMode::Poll);
        }
    }

    #[test]
    fn user_folders_get_no_sync() {
        let system_folders = SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: Some("Archive".to_string()),
            trash: Some("Trash".to_string()),
            junk: Some("Spam".to_string()),
        };
        let all_folders = vec![
            "INBOX".to_string(),
            "Drafts".to_string(),
            "Sent".to_string(),
            "Archive".to_string(),
            "Trash".to_string(),
            "Spam".to_string(),
            "Personal".to_string(),
            "Work".to_string(),
        ];
        let configs = build_default_sync_configs(&system_folders, &all_folders);

        for name in &["Personal", "Work"] {
            let cfg = configs.iter().find(|c| c.folder_name == *name).unwrap();
            assert_eq!(cfg.sync_mode, SyncMode::NoSync);
            assert_eq!(cfg.push_mode, PushMode::None);
        }
    }

    #[test]
    fn default_folder_name_returns_correct_names() {
        assert_eq!(default_folder_name(FolderRole::Drafts), "Drafts");
        assert_eq!(default_folder_name(FolderRole::Sent), "Sent");
        assert_eq!(default_folder_name(FolderRole::Archive), "Archive");
        assert_eq!(default_folder_name(FolderRole::Trash), "Trash");
        assert_eq!(default_folder_name(FolderRole::Junk), "Spam");
    }

    #[test]
    fn empty_namespace_prefix_uses_bare_names() {
        let system_folders = SystemFolders::default();
        let missing = find_missing_system_folders(&system_folders, "");

        for (_, name) in &missing {
            assert!(!name.starts_with('.'));
            assert!(!name.starts_with('/'));
        }
    }
}
