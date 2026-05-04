pub mod account;
pub mod account_order;
pub mod connection_state;
pub mod connection_test;
pub mod delete_account;
pub mod navigation;
pub mod primary;
pub mod sync_conditions;

pub use account::{
    collect_categories, detect_system_folders, notification_channel_id, resolve_color, Account,
    AccountColor, AccountValidationError, AuthMethod, EncryptionMode, Folder, FolderRole,
    NewAccountParams, Pop3Settings, Protocol, QuotaInfo, SmtpConfig, SwipeAction, SwipeDefaults,
    SystemFolders, UpdateAccountParams, QUOTA_HIGH_THRESHOLD_PERCENT,
};
pub use account_order::{apply_custom_order, default_order, move_account, order_from_accounts};
pub use connection_state::{
    format_log_timestamp, ConnectionLogEntry, ConnectionState, ConnectionStateManager,
};
pub use connection_test::{
    ConnectionTestError, ConnectionTestRequest, ConnectionTestResult, ServerTestOutcome,
};
pub use delete_account::{clear_primary_if_deleted, remove_from_order};
pub use navigation::{group_by_category, sort_accounts_flat, CategoryGroup};
pub use primary::{
    auto_designate_on_add, revoke_if_sync_disabled, set_primary, PrimaryDesignationError,
};
pub use sync_conditions::{
    evaluate as evaluate_sync_conditions, EnvironmentStatus, SyncEligibility, SyncPauseReason,
};
