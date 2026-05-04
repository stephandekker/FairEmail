pub mod account;
pub mod connection_test;

pub use account::{
    Account, AccountValidationError, AuthMethod, EncryptionMode, Folder, NewAccountParams,
    Protocol, SmtpConfig, UpdateAccountParams,
};
pub use connection_test::{
    ConnectionTestError, ConnectionTestRequest, ConnectionTestResult, ServerTestOutcome,
};
