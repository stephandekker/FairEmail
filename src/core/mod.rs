pub mod account;
pub mod account_order;
pub mod connection_test;
pub mod navigation;
pub mod primary;
pub mod sync_conditions;

pub use account::{
    collect_categories, detect_system_folders, resolve_color, Account, AccountColor,
    AccountValidationError, AuthMethod, EncryptionMode, Folder, FolderRole, NewAccountParams,
    Pop3Settings, Protocol, SmtpConfig, SwipeAction, SwipeDefaults, SystemFolders,
    UpdateAccountParams,
};
pub use account_order::{apply_custom_order, default_order, move_account, order_from_accounts};
pub use connection_test::{
    ConnectionTestError, ConnectionTestRequest, ConnectionTestResult, ServerTestOutcome,
};
pub use navigation::{group_by_category, sort_accounts_flat, CategoryGroup};
pub use primary::{
    auto_designate_on_add, revoke_if_sync_disabled, set_primary, PrimaryDesignationError,
};
pub use sync_conditions::{
    evaluate as evaluate_sync_conditions, EnvironmentStatus, SyncEligibility, SyncPauseReason,
};
