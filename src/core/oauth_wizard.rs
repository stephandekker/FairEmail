use crate::core::account::AuthMethod;
use crate::core::account_creation::{
    create_account_and_identity, AccountCreationParams, AccountCreationResult,
};
use crate::core::imap_check::{resolve_username_candidates, ImapCheckSuccess};
use crate::core::provider::Provider;
use crate::core::smtp_check::SmtpCheckSuccess;

/// Provider-specific error when OAuth connection test fails despite valid tokens (FR-24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthConnectionError {
    /// Non-technical message for the user.
    pub user_message: String,
    /// Suggested corrective action (e.g. "Enable IMAP access in your Gmail settings").
    pub corrective_action: String,
    /// Provider documentation URL, if available.
    pub documentation_url: Option<String>,
}

/// Build a provider-specific error message when a connection test fails despite
/// a valid OAuth token (FR-24, AC-5).
///
/// The corrective action is tailored to the provider so the user knows exactly
/// what setting to change (e.g. enabling IMAP in Gmail).
pub fn build_oauth_connection_error(
    provider: &Provider,
    is_imap_failure: bool,
) -> OAuthConnectionError {
    let service = if is_imap_failure { "IMAP" } else { "SMTP" };

    let corrective_action = match provider.id.as_str() {
        "gmail" => "Enable IMAP access in your Gmail settings: \
             Settings → Forwarding and POP/IMAP → Enable IMAP."
            .to_string(),
        "outlook" | "office365" => "Ensure IMAP is enabled in your Outlook settings: \
             Settings → Mail → Sync email → Enable IMAP."
            .to_string(),
        "yahoo" | "aol" => format!(
            "Ensure IMAP access is enabled in your {} account security settings.",
            provider.display_name
        ),
        _ => {
            // Use localized documentation if available, otherwise generic message.
            if let Some(doc) = provider.localized_docs.iter().find(|d| d.locale == "en") {
                doc.text.clone()
            } else {
                format!(
                    "Check that {} access is enabled in your {} account settings.",
                    service, provider.display_name
                )
            }
        }
    };

    OAuthConnectionError {
        user_message: format!(
            "Authorization succeeded, but the {} connection to {} could not be established.",
            service, provider.display_name
        ),
        corrective_action,
        documentation_url: provider.documentation_url.clone(),
    }
}

/// Derive the username to use for OAuth IMAP/SMTP authentication from the
/// email address and provider settings.
///
/// Uses the provider's `username_type` to determine whether the full email
/// or just the local part should be the primary candidate.
pub fn derive_oauth_username(email: &str, provider: &Provider) -> String {
    let candidates = resolve_username_candidates(email, provider);
    candidates
        .first()
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| email.to_string())
}

/// Create an account and sending identity using OAuth credentials (FR-23, AC-4).
///
/// This is the final step of the OAuth wizard flow: after connections have been
/// tested successfully, the account and a default sending identity are created
/// in a single call.
#[allow(clippy::too_many_arguments)]
pub fn create_oauth_account(
    provider: Provider,
    email: String,
    display_name: String,
    access_token: String,
    imap_result: ImapCheckSuccess,
    smtp_result: SmtpCheckSuccess,
    oauth_tenant: Option<String>,
    shared_mailbox: Option<String>,
) -> Result<AccountCreationResult, crate::core::account::AccountValidationError> {
    create_account_and_identity(AccountCreationParams {
        provider,
        email,
        display_name,
        credential: access_token,
        auth_method: AuthMethod::OAuth2,
        imap_result,
        smtp_result,
        accepted_certificate_fingerprint: None,
        oauth_tenant,
        shared_mailbox,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::build_imap_success;
    use crate::core::provider::{
        LocalizedDoc, MaxTlsVersion, OAuthConfig, ProviderEncryption, ServerConfig, UsernameType,
    };

    fn make_provider(id: &str, display_name: &str) -> Provider {
        Provider {
            id: id.to_string(),
            display_name: display_name.to_string(),
            domain_patterns: vec![format!("{id}.com")],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: format!("imap.{id}.com"),
                port: 993,
                encryption: ProviderEncryption::SslTls,
            },
            outgoing: ServerConfig {
                hostname: format!("smtp.{id}.com"),
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
            oauth: Some(OAuthConfig {
                auth_url: "https://auth.example.com/oauth".to_string(),
                token_url: "https://auth.example.com/token".to_string(),
                redirect_uri: "http://127.0.0.1/callback".to_string(),
                scopes: vec!["mail".to_string()],
                client_id: None,
                pkce_required: true,
                extra_params: vec![],
                userinfo_url: None,
            }),
            display_order: 1,
            enabled: true,
            supports_shared_mailbox: false,
        }
    }

    // --- build_oauth_connection_error tests ---

    #[test]
    fn gmail_imap_error_mentions_imap_settings() {
        let provider = make_provider("gmail", "Gmail (Google)");
        let error = build_oauth_connection_error(&provider, true);
        assert!(error.corrective_action.contains("IMAP"));
        assert!(error.corrective_action.contains("Gmail settings"));
        assert!(error.user_message.contains("IMAP"));
        assert!(error.user_message.contains("Gmail (Google)"));
    }

    #[test]
    fn gmail_smtp_error_mentions_smtp() {
        let provider = make_provider("gmail", "Gmail (Google)");
        let error = build_oauth_connection_error(&provider, false);
        assert!(error.user_message.contains("SMTP"));
    }

    #[test]
    fn outlook_error_mentions_outlook_settings() {
        let provider = make_provider("outlook", "Outlook.com (Microsoft)");
        let error = build_oauth_connection_error(&provider, true);
        assert!(error.corrective_action.contains("Outlook"));
        assert!(error.corrective_action.contains("IMAP"));
    }

    #[test]
    fn office365_error_mentions_outlook_settings() {
        let provider = make_provider("office365", "Microsoft 365");
        let error = build_oauth_connection_error(&provider, true);
        assert!(error.corrective_action.contains("Outlook"));
    }

    #[test]
    fn yahoo_error_mentions_provider_name() {
        let provider = make_provider("yahoo", "Yahoo Mail");
        let error = build_oauth_connection_error(&provider, true);
        assert!(error.corrective_action.contains("Yahoo Mail"));
    }

    #[test]
    fn aol_error_mentions_provider_name() {
        let provider = make_provider("aol", "AOL Mail");
        let error = build_oauth_connection_error(&provider, true);
        assert!(error.corrective_action.contains("AOL Mail"));
    }

    #[test]
    fn unknown_provider_uses_localized_docs() {
        let mut provider = make_provider("custom", "Custom Provider");
        provider.localized_docs.push(LocalizedDoc {
            locale: "en".to_string(),
            text: "Go to account settings and enable IMAP.".to_string(),
        });
        let error = build_oauth_connection_error(&provider, true);
        assert_eq!(
            error.corrective_action,
            "Go to account settings and enable IMAP."
        );
    }

    #[test]
    fn unknown_provider_generic_message_without_docs() {
        let provider = make_provider("custom", "Custom Provider");
        let error = build_oauth_connection_error(&provider, true);
        assert!(error.corrective_action.contains("Custom Provider"));
        assert!(error.corrective_action.contains("IMAP"));
    }

    #[test]
    fn error_includes_documentation_url_when_present() {
        let mut provider = make_provider("gmail", "Gmail");
        provider.documentation_url =
            Some("https://support.google.com/mail/answer/7126229".to_string());
        let error = build_oauth_connection_error(&provider, true);
        assert_eq!(
            error.documentation_url,
            Some("https://support.google.com/mail/answer/7126229".to_string())
        );
    }

    #[test]
    fn error_has_no_documentation_url_when_absent() {
        let provider = make_provider("custom", "Custom");
        let error = build_oauth_connection_error(&provider, true);
        assert!(error.documentation_url.is_none());
    }

    // --- derive_oauth_username tests ---

    #[test]
    fn username_is_email_for_email_type() {
        let provider = make_provider("gmail", "Gmail");
        let username = derive_oauth_username("user@gmail.com", &provider);
        assert_eq!(username, "user@gmail.com");
    }

    #[test]
    fn username_is_local_part_for_local_part_type() {
        let mut provider = make_provider("custom", "Custom");
        provider.username_type = UsernameType::LocalPart;
        let username = derive_oauth_username("user@custom.com", &provider);
        assert_eq!(username, "user");
    }

    // --- create_oauth_account tests ---

    #[test]
    fn creates_account_with_oauth2_auth_method() {
        let provider = make_provider("gmail", "Gmail");
        let imap_result = build_imap_success(
            "user@gmail.com".to_string(),
            vec![
                ("INBOX".to_string(), "".to_string()),
                ("Sent".to_string(), "\\Sent".to_string()),
                ("Drafts".to_string(), "\\Drafts".to_string()),
                ("Trash".to_string(), "\\Trash".to_string()),
            ],
        );
        let smtp_result = SmtpCheckSuccess {
            authenticated_username: "user@gmail.com".to_string(),
            max_message_size: Some(25_000_000),
        };

        let result = create_oauth_account(
            provider,
            "user@gmail.com".to_string(),
            "Test User".to_string(),
            "ya29.access-token".to_string(),
            imap_result,
            smtp_result,
            None,
            None,
        )
        .unwrap();

        assert_eq!(result.account.auth_method(), AuthMethod::OAuth2);
        assert_eq!(result.account.credential(), "ya29.access-token");
        assert_eq!(result.account.host(), "imap.gmail.com");
        assert_eq!(result.identity.email(), "user@gmail.com");
        assert_eq!(result.identity.display_name(), "Test User");
        assert_eq!(result.identity.smtp_auth_method(), AuthMethod::OAuth2);
    }

    #[test]
    fn oauth_account_has_provider_tuning() {
        let mut provider = make_provider("gmail", "Gmail");
        provider.keep_alive_interval = 20;
        provider.noop_keep_alive = true;

        let imap_result = build_imap_success(
            "user@gmail.com".to_string(),
            vec![("INBOX".to_string(), "".to_string())],
        );
        let smtp_result = SmtpCheckSuccess {
            authenticated_username: "user@gmail.com".to_string(),
            max_message_size: None,
        };

        let result = create_oauth_account(
            provider,
            "user@gmail.com".to_string(),
            "Test".to_string(),
            "token".to_string(),
            imap_result,
            smtp_result,
            None,
            None,
        )
        .unwrap();

        assert_eq!(result.account.polling_interval_minutes(), Some(20));
        assert!(
            result
                .account
                .keep_alive_settings()
                .unwrap()
                .use_noop_instead_of_idle
        );
    }

    #[test]
    fn oauth_account_stores_system_folders() {
        let provider = make_provider("gmail", "Gmail");
        let imap_result = build_imap_success(
            "user@gmail.com".to_string(),
            vec![
                ("INBOX".to_string(), "".to_string()),
                ("Sent".to_string(), "\\Sent".to_string()),
                ("Archive".to_string(), "\\Archive".to_string()),
            ],
        );
        let smtp_result = SmtpCheckSuccess {
            authenticated_username: "user@gmail.com".to_string(),
            max_message_size: None,
        };

        let result = create_oauth_account(
            provider,
            "user@gmail.com".to_string(),
            "Test".to_string(),
            "token".to_string(),
            imap_result,
            smtp_result,
            None,
            None,
        )
        .unwrap();

        let folders = result.account.system_folders().unwrap();
        assert_eq!(folders.sent, Some("Sent".to_string()));
        assert_eq!(folders.archive, Some("Archive".to_string()));
    }

    #[test]
    fn oauth_identity_stores_max_message_size() {
        let provider = make_provider("gmail", "Gmail");
        let imap_result = build_imap_success(
            "user@gmail.com".to_string(),
            vec![("INBOX".to_string(), "".to_string())],
        );
        let smtp_result = SmtpCheckSuccess {
            authenticated_username: "user@gmail.com".to_string(),
            max_message_size: Some(35_000_000),
        };

        let result = create_oauth_account(
            provider,
            "user@gmail.com".to_string(),
            "Test".to_string(),
            "token".to_string(),
            imap_result,
            smtp_result,
            None,
            None,
        )
        .unwrap();

        assert_eq!(result.identity.max_message_size(), Some(35_000_000));
    }
}
