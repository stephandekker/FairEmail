use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::account::{
    Account, AuthMethod, EncryptionMode, FetchSettings, KeepAliveSettings, NewAccountParams,
    Protocol, SecuritySettings, SmtpConfig,
};
use crate::core::imap_check::ImapCheckSuccess;
use crate::core::provider::{MaxTlsVersion, Provider, ProviderEncryption};
use crate::core::smtp_check::SmtpCheckSuccess;

/// A sending identity associated with an account (FR-27b).
///
/// Stores the outgoing SMTP settings, the user's display name, email address,
/// credentials, and the server's maximum message size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendingIdentity {
    /// Unique identifier for this identity.
    id: Uuid,
    /// The account this identity belongs to.
    account_id: Uuid,
    /// User's display name (shown in From: header).
    display_name: String,
    /// User's email address.
    email: String,
    /// SMTP server hostname.
    smtp_host: String,
    /// SMTP server port.
    smtp_port: u16,
    /// SMTP connection encryption.
    smtp_encryption: EncryptionMode,
    /// SMTP authentication method.
    smtp_auth_method: AuthMethod,
    /// SMTP username.
    smtp_username: String,
    /// SMTP credential (password or token).
    smtp_credential: String,
    /// Maximum message size advertised by the server, in bytes (FR-27b).
    max_message_size: Option<u64>,
}

impl SendingIdentity {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn account_id(&self) -> Uuid {
        self.account_id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn smtp_host(&self) -> &str {
        &self.smtp_host
    }

    pub fn smtp_port(&self) -> u16 {
        self.smtp_port
    }

    pub fn smtp_encryption(&self) -> EncryptionMode {
        self.smtp_encryption
    }

    pub fn smtp_auth_method(&self) -> AuthMethod {
        self.smtp_auth_method
    }

    pub fn smtp_username(&self) -> &str {
        &self.smtp_username
    }

    pub fn smtp_credential(&self) -> &str {
        &self.smtp_credential
    }

    pub fn max_message_size(&self) -> Option<u64> {
        self.max_message_size
    }
}

/// Parameters for the combined account + identity creation (FR-27, US-21).
pub struct AccountCreationParams {
    /// The detected provider (for tuning parameters).
    pub provider: Provider,
    /// The user's email address.
    pub email: String,
    /// The user's display name.
    pub display_name: String,
    /// The credential (password or OAuth token).
    pub credential: String,
    /// The authentication method used.
    pub auth_method: AuthMethod,
    /// Result of the successful IMAP connectivity check.
    pub imap_result: ImapCheckSuccess,
    /// Result of the successful SMTP connectivity check.
    pub smtp_result: SmtpCheckSuccess,
    /// Certificate fingerprint accepted by the user, if any (FR-27a).
    pub accepted_certificate_fingerprint: Option<String>,
    /// OAuth tenant identifier for multi-tenant providers (FR-10, US-4).
    pub oauth_tenant: Option<String>,
    /// Shared mailbox email address for delegated access.
    pub shared_mailbox: Option<String>,
}

/// The result of a successful account + identity creation (US-21).
pub struct AccountCreationResult {
    /// The newly created incoming-mail account.
    pub account: Account,
    /// The newly created sending identity.
    pub identity: SendingIdentity,
}

/// Convert provider encryption to account encryption mode.
fn encryption_from_provider(enc: ProviderEncryption) -> EncryptionMode {
    match enc {
        ProviderEncryption::None => EncryptionMode::None,
        ProviderEncryption::SslTls => EncryptionMode::SslTls,
        ProviderEncryption::StartTls => EncryptionMode::StartTls,
    }
}

/// Create both an incoming-mail account and a sending identity in a single step (FR-27, US-21).
///
/// Provider-specific tuning parameters (keep-alive interval, NOOP flag, partial-fetch,
/// TLS ceiling — FR-15g through FR-15j) are applied silently (Design Note N-7).
///
/// If `accepted_certificate_fingerprint` is provided, it is stored in the account's
/// security settings.
///
/// The caller is responsible for primary designation (FR-28) by calling
/// `auto_designate_on_add` after adding the account to the store.
pub fn create_account_and_identity(
    params: AccountCreationParams,
) -> Result<AccountCreationResult, crate::core::account::AccountValidationError> {
    // Build provider-specific tuning (FR-15g through FR-15j, Design Note N-7).
    let keep_alive_settings = Some(KeepAliveSettings {
        use_noop_instead_of_idle: params.provider.noop_keep_alive,
    });

    let fetch_settings = Some(FetchSettings {
        partial_fetch: params.provider.partial_fetch,
        raw_fetch: false,
        ignore_size_limits: false,
        date_header_preference: Default::default(),
        utf8_support: false,
    });

    // Store certificate fingerprint if accepted.
    let security_settings = if params.accepted_certificate_fingerprint.is_some()
        || params.provider.max_tls_version == MaxTlsVersion::Tls1_2
    {
        Some(SecuritySettings {
            certificate_fingerprint: params.accepted_certificate_fingerprint,
            ..Default::default()
        })
    } else {
        None
    };

    let smtp_encryption = encryption_from_provider(params.provider.outgoing.encryption);
    let imap_encryption = encryption_from_provider(params.provider.incoming.encryption);

    let smtp_config = SmtpConfig {
        host: params.provider.outgoing.hostname.clone(),
        port: params.provider.outgoing.port,
        encryption: smtp_encryption,
        auth_method: params.auth_method,
        username: params.smtp_result.authenticated_username.clone(),
        credential: params.credential.clone(),
    };

    let account = Account::new(NewAccountParams {
        display_name: params.display_name.clone(),
        protocol: Protocol::Imap,
        host: params.provider.incoming.hostname.clone(),
        port: params.provider.incoming.port,
        encryption: imap_encryption,
        auth_method: params.auth_method,
        username: params.imap_result.authenticated_username.clone(),
        credential: params.credential.clone(),
        smtp: Some(smtp_config),
        pop3_settings: None,
        color: None,
        avatar_path: None,
        category: None,
        sync_enabled: true,
        on_demand: false,
        polling_interval_minutes: Some(params.provider.keep_alive_interval),
        unmetered_only: false,
        vpn_only: false,
        schedule_exempt: false,
        system_folders: Some(params.imap_result.system_folders),
        swipe_defaults: None,
        notifications_enabled: true,
        security_settings,
        fetch_settings,
        keep_alive_settings,
        oauth_tenant: params.oauth_tenant,
        shared_mailbox: params.shared_mailbox,
    })?;

    let identity = SendingIdentity {
        id: Uuid::new_v4(),
        account_id: account.id(),
        display_name: params.display_name,
        email: params.email,
        smtp_host: params.provider.outgoing.hostname,
        smtp_port: params.provider.outgoing.port,
        smtp_encryption,
        smtp_auth_method: params.auth_method,
        smtp_username: params.smtp_result.authenticated_username,
        smtp_credential: params.credential,
        max_message_size: params.smtp_result.max_message_size,
    };

    Ok(AccountCreationResult { account, identity })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::build_imap_success;
    use crate::core::primary::auto_designate_on_add;
    use crate::core::provider::{ServerConfig, UsernameType};

    fn make_provider() -> Provider {
        Provider {
            id: "gmail".to_string(),
            display_name: "Gmail".to_string(),
            domain_patterns: vec!["gmail.com".to_string()],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: "imap.gmail.com".to_string(),
                port: 993,
                encryption: ProviderEncryption::SslTls,
            },
            outgoing: ServerConfig {
                hostname: "smtp.gmail.com".to_string(),
                port: 465,
                encryption: ProviderEncryption::SslTls,
            },
            username_type: UsernameType::EmailAddress,
            keep_alive_interval: 15,
            noop_keep_alive: false,
            partial_fetch: true,
            max_tls_version: MaxTlsVersion::Tls1_3,
            app_password_required: false,
            documentation_url: None,
            localized_docs: vec![],
            oauth: None,
            display_order: 1,
            enabled: true,
            supports_shared_mailbox: false,
            subtitle: None,
            registration_url: None,
            graph: None,
        }
    }

    fn make_imap_success() -> ImapCheckSuccess {
        build_imap_success(
            "user@gmail.com".to_string(),
            vec![
                ("INBOX".to_string(), "".to_string()),
                ("Sent".to_string(), "\\Sent".to_string()),
                ("Drafts".to_string(), "\\Drafts".to_string()),
                ("Trash".to_string(), "\\Trash".to_string()),
            ],
        )
    }

    fn make_smtp_success() -> SmtpCheckSuccess {
        SmtpCheckSuccess {
            authenticated_username: "user@gmail.com".to_string(),
            max_message_size: Some(25_000_000),
        }
    }

    #[test]
    fn creates_account_with_imap_settings_and_credentials() {
        let result = create_account_and_identity(AccountCreationParams {
            provider: make_provider(),
            email: "user@gmail.com".to_string(),
            display_name: "Test User".to_string(),
            credential: "app-password".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: make_imap_success(),
            smtp_result: make_smtp_success(),
            accepted_certificate_fingerprint: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();

        let acct = &result.account;
        assert_eq!(acct.host(), "imap.gmail.com");
        assert_eq!(acct.port(), 993);
        assert_eq!(acct.encryption(), EncryptionMode::SslTls);
        assert_eq!(acct.username(), "user@gmail.com");
        assert_eq!(acct.credential(), "app-password");
        assert_eq!(acct.protocol(), Protocol::Imap);
    }

    #[test]
    fn applies_provider_tuning_parameters() {
        let mut provider = make_provider();
        provider.keep_alive_interval = 20;
        provider.noop_keep_alive = true;
        provider.partial_fetch = true;

        let result = create_account_and_identity(AccountCreationParams {
            provider,
            email: "user@gmail.com".to_string(),
            display_name: "Test User".to_string(),
            credential: "secret".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: make_imap_success(),
            smtp_result: make_smtp_success(),
            accepted_certificate_fingerprint: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();

        let acct = &result.account;
        assert_eq!(acct.polling_interval_minutes(), Some(20));
        assert!(acct.keep_alive_settings().unwrap().use_noop_instead_of_idle);
        assert!(acct.fetch_settings().unwrap().partial_fetch);
    }

    #[test]
    fn creates_sending_identity_with_smtp_settings() {
        let result = create_account_and_identity(AccountCreationParams {
            provider: make_provider(),
            email: "user@gmail.com".to_string(),
            display_name: "Test User".to_string(),
            credential: "app-password".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: make_imap_success(),
            smtp_result: make_smtp_success(),
            accepted_certificate_fingerprint: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();

        let id = &result.identity;
        assert_eq!(id.display_name(), "Test User");
        assert_eq!(id.email(), "user@gmail.com");
        assert_eq!(id.smtp_host(), "smtp.gmail.com");
        assert_eq!(id.smtp_port(), 465);
        assert_eq!(id.smtp_encryption(), EncryptionMode::SslTls);
        assert_eq!(id.smtp_username(), "user@gmail.com");
        assert_eq!(id.smtp_credential(), "app-password");
        assert_eq!(id.account_id(), result.account.id());
    }

    #[test]
    fn stores_max_message_size_with_identity() {
        let result = create_account_and_identity(AccountCreationParams {
            provider: make_provider(),
            email: "user@gmail.com".to_string(),
            display_name: "Test User".to_string(),
            credential: "secret".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: make_imap_success(),
            smtp_result: make_smtp_success(),
            accepted_certificate_fingerprint: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();

        assert_eq!(result.identity.max_message_size(), Some(25_000_000));
    }

    #[test]
    fn stores_max_message_size_none_when_not_advertised() {
        let smtp = SmtpCheckSuccess {
            authenticated_username: "user@gmail.com".to_string(),
            max_message_size: None,
        };

        let result = create_account_and_identity(AccountCreationParams {
            provider: make_provider(),
            email: "user@gmail.com".to_string(),
            display_name: "Test User".to_string(),
            credential: "secret".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: make_imap_success(),
            smtp_result: smtp,
            accepted_certificate_fingerprint: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();

        assert_eq!(result.identity.max_message_size(), None);
    }

    #[test]
    fn stores_certificate_fingerprint_when_accepted() {
        let fingerprint = "AB:CD:EF:12:34:56".to_string();

        let result = create_account_and_identity(AccountCreationParams {
            provider: make_provider(),
            email: "user@gmail.com".to_string(),
            display_name: "Test User".to_string(),
            credential: "secret".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: make_imap_success(),
            smtp_result: make_smtp_success(),
            accepted_certificate_fingerprint: Some(fingerprint.clone()),
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();

        let sec = result.account.security_settings().unwrap();
        assert_eq!(
            sec.certificate_fingerprint.as_deref(),
            Some("AB:CD:EF:12:34:56")
        );
    }

    #[test]
    fn no_certificate_fingerprint_when_not_accepted() {
        let result = create_account_and_identity(AccountCreationParams {
            provider: make_provider(),
            email: "user@gmail.com".to_string(),
            display_name: "Test User".to_string(),
            credential: "secret".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: make_imap_success(),
            smtp_result: make_smtp_success(),
            accepted_certificate_fingerprint: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();

        // With TLS 1.3 and no fingerprint, security_settings may be None
        assert!(result
            .account
            .security_settings()
            .and_then(|s| s.certificate_fingerprint.as_ref())
            .is_none());
    }

    #[test]
    fn first_account_becomes_primary() {
        let result = create_account_and_identity(AccountCreationParams {
            provider: make_provider(),
            email: "user@gmail.com".to_string(),
            display_name: "Test User".to_string(),
            credential: "secret".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: make_imap_success(),
            smtp_result: make_smtp_success(),
            accepted_certificate_fingerprint: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();

        let mut accounts = vec![result.account];
        let id = accounts[0].id();
        let designated = auto_designate_on_add(&mut accounts, id);
        assert!(designated);
        assert!(accounts[0].is_primary());
    }

    #[test]
    fn second_account_does_not_override_primary() {
        let result1 = create_account_and_identity(AccountCreationParams {
            provider: make_provider(),
            email: "user@gmail.com".to_string(),
            display_name: "First User".to_string(),
            credential: "secret".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: make_imap_success(),
            smtp_result: make_smtp_success(),
            accepted_certificate_fingerprint: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();

        let result2 = create_account_and_identity(AccountCreationParams {
            provider: make_provider(),
            email: "other@gmail.com".to_string(),
            display_name: "Second User".to_string(),
            credential: "secret2".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: build_imap_success(
                "other@gmail.com".to_string(),
                vec![("INBOX".to_string(), "".to_string())],
            ),
            smtp_result: SmtpCheckSuccess {
                authenticated_username: "other@gmail.com".to_string(),
                max_message_size: Some(10_000_000),
            },
            accepted_certificate_fingerprint: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();

        let mut accounts = vec![result1.account, result2.account];
        let id1 = accounts[0].id();
        let id2 = accounts[1].id();

        // First account becomes primary
        auto_designate_on_add(&mut accounts, id1);
        assert!(accounts[0].is_primary());

        // Second account does NOT override
        let designated = auto_designate_on_add(&mut accounts, id2);
        assert!(!designated);
        assert!(accounts[0].is_primary());
        assert!(!accounts[1].is_primary());
    }

    #[test]
    fn account_and_identity_created_in_single_call() {
        // This test verifies US-21: both are created from a single function call.
        let result = create_account_and_identity(AccountCreationParams {
            provider: make_provider(),
            email: "user@gmail.com".to_string(),
            display_name: "Test User".to_string(),
            credential: "secret".to_string(),
            auth_method: AuthMethod::Plain,
            imap_result: make_imap_success(),
            smtp_result: make_smtp_success(),
            accepted_certificate_fingerprint: None,
            oauth_tenant: None,
            shared_mailbox: None,
        });

        assert!(result.is_ok());
        let res = result.unwrap();
        // Both exist from the single call
        assert_eq!(res.account.display_name(), "Test User");
        assert_eq!(res.identity.email(), "user@gmail.com");
        assert_eq!(res.identity.account_id(), res.account.id());
    }
}
