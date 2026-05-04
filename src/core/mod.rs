pub mod account;
pub mod account_order;
pub mod auth_error;
pub mod certificate;
pub mod connection_state;
pub mod connection_test;
pub mod delete_account;
pub(crate) mod detection_progress;
pub mod duplicate_account;
pub mod export_accounts;
pub mod imap_check;
pub mod import_accounts;
pub mod navigation;
pub mod primary;
pub(crate) mod privacy;
pub mod proprietary_provider;
pub mod provider;
pub(crate) mod provider_data;
pub mod smtp_check;
pub mod sync_conditions;
pub mod wizard_validation;

pub use account::{
    collect_categories, detect_system_folders, notification_channel_id, resolve_color, Account,
    AccountColor, AccountValidationError, AuthMethod, DateHeaderPreference, EncryptionMode,
    FetchSettings, Folder, FolderRole, KeepAliveSettings, NewAccountParams, Pop3Settings, Protocol,
    QuotaInfo, SecuritySettings, SmtpConfig, SwipeAction, SwipeDefaults, SystemFolders,
    UpdateAccountParams, QUOTA_HIGH_THRESHOLD_PERCENT,
};
pub use account_order::{apply_custom_order, default_order, move_account, order_from_accounts};
pub use auth_error::{
    app_password_hint_text, build_provider_hint, is_outlook_domain, outlook_documentation_url,
    present_connectivity_error, present_imap_error, present_smtp_error, AuthErrorKind,
    AuthErrorPresentation, ProviderHint, GENERAL_SUPPORT_URL,
};
pub use certificate::{CertificateDecision, CertificateInfo};
pub use connection_state::{
    format_log_timestamp, ConnectionLogEntry, ConnectionState, ConnectionStateManager,
};
pub use connection_test::{
    ConnectionTestError, ConnectionTestRequest, ConnectionTestResult, ServerTestOutcome,
};
pub use delete_account::{clear_primary_if_deleted, remove_from_order};
pub use duplicate_account::duplicate as duplicate_account;
pub use export_accounts::{
    export_accounts, EncryptedEnvelope, ExportCategory, ExportEnvelope, ExportError, ExportOptions,
    ExportedAccount,
};
pub use imap_check::{
    build_imap_success, detect_folder_role, resolve_username_candidates, ImapCheckError,
    ImapCheckResult, ImapCheckSuccess, ImapConnectionParams, ImapFolder, UsernameCandidate,
};
pub use import_accounts::{
    import_accounts, parse_import_data, AccountImportOutcome, DuplicateStrategy, ImportError,
    ImportOptions, ImportResult,
};
pub use navigation::{group_by_category, sort_accounts_flat, CategoryGroup};
pub use primary::{
    auto_designate_on_add, revoke_if_sync_disabled, set_primary, PrimaryDesignationError,
};
pub use provider::{
    LocalizedDoc, MatchScore, MaxTlsVersion, OAuthConfig, Provider, ProviderCandidate,
    ProviderDatabase, ProviderEncryption, ServerConfig, UsernameType,
};
pub use smtp_check::{
    combine_connectivity_results, ConnectivityCheckError, ConnectivityCheckResult, SmtpCheckError,
    SmtpCheckResult, SmtpCheckSuccess, SmtpConnectionParams,
};
pub use sync_conditions::{
    evaluate as evaluate_sync_conditions, EnvironmentStatus, SyncEligibility, SyncPauseReason,
};
