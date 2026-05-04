use uuid::Uuid;

use crate::core::account::SystemFolders;
use crate::core::folder_setup::{
    build_default_sync_configs, complete_system_folders, find_missing_system_folders,
    FolderSetupError, FolderSetupResult,
};

/// Trait for performing folder creation and sync operations on the server (FR-29, FR-31).
///
/// Implementations handle the actual IMAP commands (CREATE mailbox, initial sync).
/// The mock implementation simulates success for testing.
pub trait FolderSyncService {
    /// Create a folder on the IMAP server (FR-29).
    /// Returns `Ok(())` on success or an error message on failure.
    fn create_folder(&self, account_id: Uuid, folder_name: &str) -> Result<(), String>;

    /// Trigger an immediate synchronization cycle for the account (FR-31).
    /// Returns `Ok(())` when the sync has been initiated.
    fn trigger_sync(&self, account_id: Uuid) -> Result<(), String>;

    /// Signal navigation to the inbox view (FR-31).
    /// Returns `Ok(())` when navigation has been initiated.
    fn navigate_to_inbox(&self, account_id: Uuid) -> Result<(), String>;
}

/// Perform the complete post-account-creation folder setup (FR-29, FR-30, FR-31).
///
/// This function orchestrates:
/// 1. Detecting missing system folders
/// 2. Creating them on the server
/// 3. Building default sync configurations
/// 4. Triggering immediate sync
/// 5. Navigating to inbox
///
/// The `namespace_prefix` is the IMAP namespace prefix for the account (e.g. "INBOX." or "").
/// The `all_folder_names` should include all folders on the server (including any that will
/// be created).
pub fn perform_folder_setup(
    service: &dyn FolderSyncService,
    account_id: Uuid,
    existing_system_folders: &SystemFolders,
    existing_folder_names: &[String],
    namespace_prefix: &str,
) -> Result<FolderSetupResult, FolderSetupError> {
    // Step 1: Find missing system folders (FR-29)
    let missing = find_missing_system_folders(existing_system_folders, namespace_prefix);

    // Step 2: Create missing folders on the server
    let mut created_folders = Vec::new();
    for (role, folder_name) in &missing {
        service
            .create_folder(account_id, folder_name)
            .map_err(|e| FolderSetupError::FolderCreationFailed(folder_name.clone(), e))?;
        created_folders.push((*role, folder_name.clone()));
    }

    // Step 3: Build complete system folder map
    let system_folders = complete_system_folders(existing_system_folders, &created_folders);

    // Step 4: Build complete folder list (existing + created)
    let mut all_folders: Vec<String> = existing_folder_names.to_vec();
    for (_, name) in &created_folders {
        if !all_folders.iter().any(|f| f == name) {
            all_folders.push(name.clone());
        }
    }

    // Step 5: Build default sync configurations (FR-30)
    let sync_configs = build_default_sync_configs(&system_folders, &all_folders);

    // Step 6: Trigger immediate sync (FR-31)
    service
        .trigger_sync(account_id)
        .map_err(FolderSetupError::SyncTriggerFailed)?;

    // Step 7: Navigate to inbox view (FR-31)
    // Navigation failure is not fatal — we still report success for folder setup
    let _ = service.navigate_to_inbox(account_id);

    Ok(FolderSetupResult {
        account_id,
        created_folders,
        system_folders,
        sync_configs,
        sync_triggered: true,
    })
}

/// Mock implementation of `FolderSyncService` for testing.
///
/// Behavior:
/// - Folder names containing "fail" will fail creation
/// - Account IDs where the first byte is 0xFF will fail sync trigger
/// - Otherwise all operations succeed
pub struct MockFolderSyncService;

impl FolderSyncService for MockFolderSyncService {
    fn create_folder(&self, _account_id: Uuid, folder_name: &str) -> Result<(), String> {
        if folder_name.to_lowercase().contains("fail") {
            return Err("server refused to create folder".to_string());
        }
        Ok(())
    }

    fn trigger_sync(&self, _account_id: Uuid) -> Result<(), String> {
        Ok(())
    }

    fn navigate_to_inbox(&self, _account_id: Uuid) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::FolderRole;

    fn make_full_system_folders() -> SystemFolders {
        SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: Some("Archive".to_string()),
            trash: Some("Trash".to_string()),
            junk: Some("Spam".to_string()),
        }
    }

    fn make_partial_system_folders() -> SystemFolders {
        SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: None,
            trash: None,
            junk: None,
        }
    }

    #[test]
    fn setup_with_all_folders_present_creates_nothing() {
        let service = MockFolderSyncService;
        let account_id = Uuid::new_v4();
        let system_folders = make_full_system_folders();
        let folder_names = vec![
            "INBOX".to_string(),
            "Drafts".to_string(),
            "Sent".to_string(),
            "Archive".to_string(),
            "Trash".to_string(),
            "Spam".to_string(),
        ];

        let result =
            perform_folder_setup(&service, account_id, &system_folders, &folder_names, "").unwrap();

        assert!(result.created_folders.is_empty());
        assert_eq!(result.account_id, account_id);
        assert!(result.sync_triggered);
    }

    #[test]
    fn setup_creates_missing_folders() {
        let service = MockFolderSyncService;
        let account_id = Uuid::new_v4();
        let system_folders = make_partial_system_folders();
        let folder_names = vec![
            "INBOX".to_string(),
            "Drafts".to_string(),
            "Sent".to_string(),
        ];

        let result =
            perform_folder_setup(&service, account_id, &system_folders, &folder_names, "").unwrap();

        assert_eq!(result.created_folders.len(), 3);
        assert!(result
            .created_folders
            .contains(&(FolderRole::Archive, "Archive".to_string())));
        assert!(result
            .created_folders
            .contains(&(FolderRole::Trash, "Trash".to_string())));
        assert!(result
            .created_folders
            .contains(&(FolderRole::Junk, "Spam".to_string())));
    }

    #[test]
    fn setup_updates_system_folders_after_creation() {
        let service = MockFolderSyncService;
        let account_id = Uuid::new_v4();
        let system_folders = make_partial_system_folders();
        let folder_names = vec![
            "INBOX".to_string(),
            "Drafts".to_string(),
            "Sent".to_string(),
        ];

        let result =
            perform_folder_setup(&service, account_id, &system_folders, &folder_names, "").unwrap();

        assert_eq!(result.system_folders.drafts.as_deref(), Some("Drafts"));
        assert_eq!(result.system_folders.sent.as_deref(), Some("Sent"));
        assert_eq!(result.system_folders.archive.as_deref(), Some("Archive"));
        assert_eq!(result.system_folders.trash.as_deref(), Some("Trash"));
        assert_eq!(result.system_folders.junk.as_deref(), Some("Spam"));
    }

    #[test]
    fn setup_applies_correct_sync_defaults() {
        let service = MockFolderSyncService;
        let account_id = Uuid::new_v4();
        let system_folders = make_full_system_folders();
        let folder_names = vec![
            "INBOX".to_string(),
            "Drafts".to_string(),
            "Sent".to_string(),
            "Archive".to_string(),
            "Trash".to_string(),
            "Spam".to_string(),
            "Personal".to_string(),
        ];

        let result =
            perform_folder_setup(&service, account_id, &system_folders, &folder_names, "").unwrap();

        use crate::core::folder_setup::{PushMode, SyncMode};

        // Inbox: sync + download + idle
        let inbox = result
            .sync_configs
            .iter()
            .find(|c| c.folder_name == "INBOX")
            .unwrap();
        assert_eq!(inbox.sync_mode, SyncMode::SyncAndDownload);
        assert_eq!(inbox.push_mode, PushMode::IdleIfSupported);

        // Drafts: sync + download, polled
        let drafts = result
            .sync_configs
            .iter()
            .find(|c| c.folder_name == "Drafts")
            .unwrap();
        assert_eq!(drafts.sync_mode, SyncMode::SyncAndDownload);
        assert_eq!(drafts.push_mode, PushMode::Poll);

        // Trash: sync only, polled
        let trash = result
            .sync_configs
            .iter()
            .find(|c| c.folder_name == "Trash")
            .unwrap();
        assert_eq!(trash.sync_mode, SyncMode::SyncOnly);
        assert_eq!(trash.push_mode, PushMode::Poll);

        // Personal: no sync
        let personal = result
            .sync_configs
            .iter()
            .find(|c| c.folder_name == "Personal")
            .unwrap();
        assert_eq!(personal.sync_mode, SyncMode::NoSync);
        assert_eq!(personal.push_mode, PushMode::None);
    }

    #[test]
    fn setup_triggers_sync() {
        let service = MockFolderSyncService;
        let account_id = Uuid::new_v4();
        let system_folders = make_full_system_folders();
        let folder_names = vec!["INBOX".to_string()];

        let result =
            perform_folder_setup(&service, account_id, &system_folders, &folder_names, "").unwrap();

        assert!(result.sync_triggered);
    }

    #[test]
    fn setup_uses_namespace_prefix_for_created_folders() {
        let service = MockFolderSyncService;
        let account_id = Uuid::new_v4();
        let system_folders = SystemFolders::default();
        let folder_names = vec!["INBOX".to_string()];

        let result = perform_folder_setup(
            &service,
            account_id,
            &system_folders,
            &folder_names,
            "INBOX.",
        )
        .unwrap();

        assert!(result
            .created_folders
            .contains(&(FolderRole::Drafts, "INBOX.Drafts".to_string())));
        assert!(result
            .created_folders
            .contains(&(FolderRole::Sent, "INBOX.Sent".to_string())));
    }

    #[test]
    fn setup_fails_when_folder_creation_fails() {
        // Create a service that fails for specific folder names
        struct FailingService;
        impl FolderSyncService for FailingService {
            fn create_folder(&self, _account_id: Uuid, folder_name: &str) -> Result<(), String> {
                if folder_name == "Archive" {
                    return Err("permission denied".to_string());
                }
                Ok(())
            }
            fn trigger_sync(&self, _account_id: Uuid) -> Result<(), String> {
                Ok(())
            }
            fn navigate_to_inbox(&self, _account_id: Uuid) -> Result<(), String> {
                Ok(())
            }
        }

        let service = FailingService;
        let account_id = Uuid::new_v4();
        let system_folders = SystemFolders {
            drafts: Some("Drafts".to_string()),
            sent: Some("Sent".to_string()),
            archive: None,
            trash: Some("Trash".to_string()),
            junk: Some("Spam".to_string()),
        };
        let folder_names = vec!["INBOX".to_string(), "Drafts".to_string()];

        let err = perform_folder_setup(&service, account_id, &system_folders, &folder_names, "")
            .unwrap_err();

        match err {
            FolderSetupError::FolderCreationFailed(name, msg) => {
                assert_eq!(name, "Archive");
                assert_eq!(msg, "permission denied");
            }
            _ => panic!("expected FolderCreationFailed"),
        }
    }

    #[test]
    fn setup_fails_when_sync_trigger_fails() {
        struct SyncFailService;
        impl FolderSyncService for SyncFailService {
            fn create_folder(&self, _account_id: Uuid, _folder_name: &str) -> Result<(), String> {
                Ok(())
            }
            fn trigger_sync(&self, _account_id: Uuid) -> Result<(), String> {
                Err("connection lost".to_string())
            }
            fn navigate_to_inbox(&self, _account_id: Uuid) -> Result<(), String> {
                Ok(())
            }
        }

        let service = SyncFailService;
        let account_id = Uuid::new_v4();
        let system_folders = make_full_system_folders();
        let folder_names = vec!["INBOX".to_string()];

        let err = perform_folder_setup(&service, account_id, &system_folders, &folder_names, "")
            .unwrap_err();

        match err {
            FolderSetupError::SyncTriggerFailed(msg) => {
                assert_eq!(msg, "connection lost");
            }
            _ => panic!("expected SyncTriggerFailed"),
        }
    }

    #[test]
    fn all_system_folders_present_in_folder_list_after_setup() {
        let service = MockFolderSyncService;
        let account_id = Uuid::new_v4();
        let system_folders = SystemFolders::default();
        let folder_names = vec!["INBOX".to_string()];

        let result =
            perform_folder_setup(&service, account_id, &system_folders, &folder_names, "").unwrap();

        // All system folder roles should be assigned
        assert!(result.system_folders.drafts.is_some());
        assert!(result.system_folders.sent.is_some());
        assert!(result.system_folders.archive.is_some());
        assert!(result.system_folders.trash.is_some());
        assert!(result.system_folders.junk.is_some());
    }

    #[test]
    fn inbox_sync_config_uses_push_idle_when_supported() {
        let service = MockFolderSyncService;
        let account_id = Uuid::new_v4();
        let system_folders = make_full_system_folders();
        let folder_names = vec!["INBOX".to_string()];

        let result =
            perform_folder_setup(&service, account_id, &system_folders, &folder_names, "").unwrap();

        use crate::core::folder_setup::PushMode;
        let inbox = result
            .sync_configs
            .iter()
            .find(|c| c.folder_name == "INBOX")
            .unwrap();
        assert_eq!(inbox.push_mode, PushMode::IdleIfSupported);
    }
}
