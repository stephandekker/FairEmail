pub mod account;
pub mod connection_test;
pub mod primary;
pub mod sync_conditions;

pub use account::{
    resolve_color, Account, AccountColor, AccountValidationError, AuthMethod, EncryptionMode,
    Folder, NewAccountParams, Pop3Settings, Protocol, SmtpConfig, UpdateAccountParams,
};
pub use connection_test::{
    ConnectionTestError, ConnectionTestRequest, ConnectionTestResult, ServerTestOutcome,
};
pub use primary::{
    auto_designate_on_add, revoke_if_sync_disabled, set_primary, PrimaryDesignationError,
};
pub use sync_conditions::{
    evaluate as evaluate_sync_conditions, EnvironmentStatus, SyncEligibility, SyncPauseReason,
};
