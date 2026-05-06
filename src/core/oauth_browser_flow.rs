//! End-to-end OAuth browser authorization flow orchestration.
//!
//! Ties together the individual OAuth components (session, PKCE, token exchange,
//! token storage) into a single flow that:
//! 1. Binds a local redirect listener
//! 2. Builds the authorization URL (with PKCE, state, provider-specific params)
//! 3. Opens the provider's consent page in the default browser
//! 4. Waits for the authorization code via redirect callback
//! 5. Exchanges the code for access + refresh tokens
//! 6. Validates the token response (requires refresh token)
//! 7. Returns the validated tokens ready for XOAUTH2 authentication
//!
//! The flow is provider-agnostic: all provider-specific behaviour (scopes,
//! extra params, PKCE requirement, privacy policy URL) comes from the
//! `OAuthConfig` in the provider registry.

use crate::core::oauth_flow::{OAuthFlowError, OAuthSession, ValidatedTokenResponse};
use crate::core::provider::OAuthConfig;

/// Result of a successful browser authorization flow.
#[derive(Debug, Clone)]
pub(crate) struct BrowserAuthResult {
    /// Validated tokens (access, refresh, expiry).
    pub tokens: ValidatedTokenResponse,
    /// The OAuth session, retained for potential re-use of the config.
    pub redirect_port: u16,
    /// The OpenID Connect ID token, if the provider returned one.
    pub id_token: Option<String>,
}

/// Parameters for initiating a browser authorization flow.
#[derive(Debug, Clone)]
pub(crate) struct BrowserAuthParams {
    /// The provider's OAuth configuration (from the registry).
    pub oauth_config: OAuthConfig,
    /// User's preferred browser, if configured.
    pub browser_preference: Option<String>,
}

/// Execute the complete browser authorization flow.
///
/// This is a blocking function that:
/// 1. Binds a local TCP listener for the redirect callback
/// 2. Creates an `OAuthSession` with PKCE and state
/// 3. Opens the authorization URL in the system browser
/// 4. Blocks until the callback arrives (or timeout)
/// 5. Validates the callback state (CSRF protection)
/// 6. Exchanges the authorization code for tokens via HTTPS
/// 7. Validates the token response (refresh token required)
///
/// # Errors
///
/// Returns `OAuthFlowError` if any step fails (browser open, callback,
/// token exchange, missing refresh token, etc.).
pub(crate) fn run_browser_authorization_flow(
    params: &BrowserAuthParams,
) -> Result<BrowserAuthResult, OAuthFlowError> {
    // Step 1: Bind local redirect listener on an OS-assigned port.
    let (listener, port) = crate::services::oauth_service::bind_redirect_listener()?;

    // Step 2: Create OAuth session with PKCE, state, and provider config.
    let session = OAuthSession::new(params.oauth_config.clone(), port);

    // Step 3: Build authorization URL and open in the system browser.
    let auth_url = session.authorization_url();
    crate::services::oauth_service::open_browser_with_selection(
        &auth_url,
        params.browser_preference.as_deref(),
    )?;

    // Step 4: Wait for the redirect callback (blocks until connection arrives).
    let callback = crate::services::oauth_service::wait_for_callback(listener)?;

    // Step 5: Validate the state parameter (CSRF + timeout protection).
    session.validate_state(Some(&callback.state))?;

    // Step 6: Exchange the authorization code for tokens.
    let exchange_params = session.token_exchange_params(&callback.code);
    let token_response = crate::services::oauth_service::exchange_code_for_tokens(exchange_params)?;

    // Capture the ID token before validation consumes the response.
    let id_token = token_response.id_token.clone();

    // Step 7: Validate that a refresh token was returned.
    let validated = crate::core::oauth_flow::validate_token_response(token_response)?;

    Ok(BrowserAuthResult {
        tokens: validated,
        redirect_port: port,
        id_token,
    })
}

/// Return the provider's privacy policy URL, if published (US-8).
///
/// This is a convenience accessor so the UI can display a link
/// during the authorization flow without reaching into OAuthConfig directly.
pub(crate) fn privacy_policy_url(config: &OAuthConfig) -> Option<&str> {
    config.privacy_policy_url.as_deref()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::OAuthConfig;

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["https://mail.google.com/".to_string()],
            client_id: Some("test-client-id".to_string()),
            pkce_required: true,
            extra_params: vec![
                ("prompt".to_string(), "consent".to_string()),
                ("access_type".to_string(), "offline".to_string()),
            ],
            userinfo_url: None,
            privacy_policy_url: Some("https://policies.google.com/privacy".to_string()),
        }
    }

    #[test]
    fn privacy_policy_url_returns_some_when_set() {
        let config = test_config();
        assert_eq!(
            privacy_policy_url(&config),
            Some("https://policies.google.com/privacy")
        );
    }

    #[test]
    fn privacy_policy_url_returns_none_when_absent() {
        let mut config = test_config();
        config.privacy_policy_url = None;
        assert_eq!(privacy_policy_url(&config), None);
    }

    #[test]
    fn browser_auth_params_carries_config() {
        let config = test_config();
        let params = BrowserAuthParams {
            oauth_config: config.clone(),
            browser_preference: Some("firefox".to_string()),
        };
        assert_eq!(params.oauth_config.auth_url, config.auth_url);
        assert_eq!(params.browser_preference, Some("firefox".to_string()));
    }

    #[test]
    fn browser_auth_params_no_browser_preference() {
        let params = BrowserAuthParams {
            oauth_config: test_config(),
            browser_preference: None,
        };
        assert!(params.browser_preference.is_none());
    }
}
