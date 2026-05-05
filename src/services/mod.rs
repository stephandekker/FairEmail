pub mod account_store;
pub mod connection_log_store;
pub mod connection_tester;
pub(crate) mod database;
pub(crate) mod dns_resolver;
pub mod export_service;
pub mod folder_store;
pub mod folder_sync;
pub mod fs_content_store;
pub mod identity_store;
pub(crate) mod idle_service;
pub mod imap_checker;
pub(crate) mod imap_client;
pub mod import_service;
pub mod inbound_tester;
pub mod libsecret_credential_store;
pub mod memory_content_store;
pub mod memory_credential_store;
pub mod message_fetch;
pub mod message_store;
pub mod network;
pub mod notification_channel;
#[allow(dead_code)]
pub(crate) mod notification_dispatcher;
pub mod order_store;
pub mod pending_ops_store;
pub mod real_connection_tester;
pub mod real_imap_checker;
pub mod real_inbound_tester;
pub mod real_smtp_checker;
pub mod rebuild_index;
pub mod settings_store;
pub mod smtp_checker;
pub(crate) mod smtp_client;
pub mod sqlite_account_store;
pub mod sqlite_order_store;
pub mod sqlite_settings_store;
#[allow(dead_code)]
pub mod sync_engine;
pub mod sync_state_store;
pub mod user_provider_service;

pub use account_store::StoreError;
pub use connection_log_store::{append_connection_logs, load_connection_logs};
pub use connection_tester::{ConnectionTester, MockConnectionTester};
pub use export_service::{export_to_file, ExportResult, ExportServiceError};
pub use folder_store::{
    is_folder_notifications_enabled, load_folders, replace_folders,
    set_folder_notifications_enabled,
};
pub use folder_sync::{
    perform_folder_setup, FolderSyncService, MockFolderSyncService, RealFolderSyncService,
};
pub use fs_content_store::FsContentStore;
pub use identity_store::{
    insert_identity, load_identities_for_account, load_identity_by_id, update_max_message_size,
    IdentityRow,
};
pub use imap_checker::{ImapChecker, MockImapChecker, MOCK_CERT_FINGERPRINT};
pub use import_service::{
    import_from_file, is_file_encrypted, read_import_file, ImportServiceError,
};
pub use inbound_tester::{InboundTester, MockInboundTester};
pub use libsecret_credential_store::LibsecretCredentialStore;
pub use memory_content_store::MemoryContentStore;
pub use memory_credential_store::MemoryCredentialStore;
#[allow(unused_imports)]
pub(crate) use message_fetch::{
    fetch_and_store_folder, incremental_sync_folder, IncrementalSyncResult,
};
pub use message_store::{
    count_messages, delete_message, delete_messages_for_folder, find_folder_id,
    find_message_by_uid_in_folder, insert_message, load_folder_sync_state, load_message,
    load_uids_for_folder, update_folder_sync_state, update_message_flags,
};
pub use notification_channel::{
    FreedesktopNotificationChannelManager, MockNotificationChannelManager,
    NotificationChannelManager,
};
pub use order_store::OrderStore;
pub use pending_ops_store::{
    complete_op, count_pending_ops, insert_pending_op, load_pending_ops, mark_failed,
    mark_in_flight, requeue_op,
};
pub use real_connection_tester::RealConnectionTester;
pub use real_imap_checker::RealImapChecker;
pub use real_inbound_tester::RealInboundTester;
pub use real_smtp_checker::RealSmtpChecker;
pub use settings_store::{AppSettings, SettingsStore};
pub use smtp_checker::{MockSmtpChecker, SmtpChecker, MOCK_SMTP_CERT_FINGERPRINT};
pub use sqlite_account_store::SqliteAccountStore as AccountStore;
pub use sqlite_order_store::SqliteOrderStore;
pub use sqlite_settings_store::SqliteSettingsStore;
pub use sync_state_store::{load_sync_state, upsert_sync_state};
pub use user_provider_service::{
    load_user_provider_file, load_user_provider_file_from, user_provider_file_path,
};
