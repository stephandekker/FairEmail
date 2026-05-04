pub mod account;
pub mod connection_test;

pub use account::{
    resolve_color, Account, AccountColor, AccountValidationError, AuthMethod, EncryptionMode,
    Folder, NewAccountParams, Pop3Settings, Protocol, SmtpConfig, UpdateAccountParams,
};
pub use connection_test::{
    ConnectionTestError, ConnectionTestRequest, ConnectionTestResult, ServerTestOutcome,
};
