pub mod account;
pub mod connection_test;

pub use account::{
    Account, AccountValidationError, AuthMethod, EncryptionMode, NewAccountParams, Protocol,
    SmtpConfig,
};
pub use connection_test::{
    ConnectionTestError, ConnectionTestRequest, ConnectionTestResult, ServerTestOutcome,
};
