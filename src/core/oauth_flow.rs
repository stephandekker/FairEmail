use crate::core::provider::OAuthConfig;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::time::{Duration, Instant};

/// Maximum time an OAuth session remains valid (FR-7, NFR-5).
pub const SESSION_TIMEOUT: Duration = Duration::from_secs(20 * 60);

/// Errors that can occur during the OAuth authorization flow.
#[derive(Debug, thiserror::Error)]
pub enum OAuthFlowError {
    #[error("State parameter missing or mismatched")]
    StateMismatch,

    #[error("OAuth session expired (exceeded 20 minute timeout)")]
    SessionExpired,

    #[error("Provider did not return a refresh token")]
    NoRefreshToken,

    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),

    #[error("Failed to open system browser: {0}")]
    BrowserOpenFailed(String),

    #[error("Failed to start redirect listener: {0}")]
    ServerBindFailed(String),

    #[error("Authorization callback error: {0}")]
    CallbackError(String),

    #[error("Credential storage error: {0}")]
    CredentialStoreError(String),
}

/// PKCE code verifier and S256 challenge pair (FR-4, N-2).
///
/// Always uses S256 as defense-in-depth, regardless of provider requirements.
#[derive(Debug, Clone)]
pub struct PkceChallenge {
    verifier: String,
    challenge: String,
}

impl PkceChallenge {
    /// Generate a new random PKCE verifier and its S256 challenge.
    pub fn generate() -> Self {
        let random_bytes: [u8; 32] = rand::random();
        let verifier = URL_SAFE_NO_PAD.encode(random_bytes);
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        Self {
            verifier,
            challenge,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_test(verifier: &str) -> Self {
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        Self {
            verifier: verifier.to_string(),
            challenge,
        }
    }

    /// The code verifier sent during token exchange.
    pub fn verifier(&self) -> &str {
        &self.verifier
    }

    /// The S256 challenge sent in the authorization request.
    pub fn challenge(&self) -> &str {
        &self.challenge
    }
}

/// Generate a cryptographic state parameter for CSRF protection (FR-6, NFR-4).
pub fn generate_state() -> String {
    let random_bytes: [u8; 32] = rand::random();
    URL_SAFE_NO_PAD.encode(random_bytes)
}

/// An in-progress OAuth authorization session tracking PKCE, state, and timeout.
pub struct OAuthSession {
    state: String,
    pkce: PkceChallenge,
    created_at: Instant,
    redirect_port: u16,
    oauth_config: OAuthConfig,
}

impl OAuthSession {
    /// Create a new OAuth session with fresh PKCE and state values.
    pub fn new(oauth_config: OAuthConfig, redirect_port: u16) -> Self {
        Self {
            state: generate_state(),
            pkce: PkceChallenge::generate(),
            created_at: Instant::now(),
            redirect_port,
            oauth_config,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_values(
        oauth_config: OAuthConfig,
        redirect_port: u16,
        state: String,
        pkce: PkceChallenge,
    ) -> Self {
        Self {
            state,
            pkce,
            created_at: Instant::now(),
            redirect_port,
            oauth_config,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_created_at(mut self, created_at: Instant) -> Self {
        self.created_at = created_at;
        self
    }

    /// Whether the session has exceeded the 20-minute timeout.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > SESSION_TIMEOUT
    }

    /// Validate the state parameter from a redirect callback (FR-6, FR-7).
    pub fn validate_state(&self, received_state: Option<&str>) -> Result<(), OAuthFlowError> {
        if self.is_expired() {
            return Err(OAuthFlowError::SessionExpired);
        }
        match received_state {
            Some(s) if s == self.state => Ok(()),
            _ => Err(OAuthFlowError::StateMismatch),
        }
    }

    /// Build the authorization URL for the system browser (FR-5).
    pub fn authorization_url(&self) -> String {
        let redirect = format!("http://127.0.0.1:{}/callback", self.redirect_port);
        let mut params = Vec::new();

        params.push(("response_type".to_string(), "code".to_string()));
        if let Some(ref client_id) = self.oauth_config.client_id {
            params.push(("client_id".to_string(), client_id.clone()));
        }
        params.push(("redirect_uri".to_string(), redirect));
        if !self.oauth_config.scopes.is_empty() {
            params.push(("scope".to_string(), self.oauth_config.scopes.join(" ")));
        }
        params.push(("state".to_string(), self.state.clone()));
        params.push((
            "code_challenge".to_string(),
            self.pkce.challenge().to_string(),
        ));
        params.push(("code_challenge_method".to_string(), "S256".to_string()));

        for (key, value) in &self.oauth_config.extra_params {
            params.push((key.clone(), value.clone()));
        }

        let query: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", self.oauth_config.auth_url, query)
    }

    /// Build the parameters needed for the token exchange request.
    pub fn token_exchange_params(&self, authorization_code: &str) -> TokenExchangeParams {
        TokenExchangeParams {
            token_url: self.oauth_config.token_url.clone(),
            code: authorization_code.to_string(),
            redirect_uri: format!("http://127.0.0.1:{}/callback", self.redirect_port),
            client_id: self.oauth_config.client_id.clone().unwrap_or_default(),
            code_verifier: self.pkce.verifier().to_string(),
        }
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn redirect_port(&self) -> u16 {
        self.redirect_port
    }

    pub fn oauth_config(&self) -> &OAuthConfig {
        &self.oauth_config
    }
}

/// Parameters for the token exchange HTTP POST request (FR-8).
pub struct TokenExchangeParams {
    pub token_url: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub code_verifier: String,
}

impl TokenExchangeParams {
    /// Build the form-encoded body string for the POST request.
    pub fn form_body(&self) -> String {
        [
            ("grant_type", "authorization_code"),
            ("code", &self.code),
            ("redirect_uri", &self.redirect_uri),
            ("client_id", &self.client_id),
            ("code_verifier", &self.code_verifier),
        ]
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
    }
}

/// Raw token response fields from the provider.
#[derive(Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    /// OpenID Connect ID token (JWT), if returned by the provider.
    /// Contains identity claims (email, name) that can be decoded without
    /// verification for extracting user info (FR-34).
    pub id_token: Option<String>,
}

/// A validated token response with a guaranteed refresh token (FR-9, N-4).
#[derive(Debug, Clone)]
pub struct ValidatedTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: Option<u64>,
}

/// Validate that a token response contains a non-empty refresh token (FR-9, N-4).
///
/// If the provider did not return a refresh token, the authorization is treated
/// as failed. This is deliberate (Design Note N-4).
pub fn validate_token_response(
    response: TokenResponse,
) -> Result<ValidatedTokenResponse, OAuthFlowError> {
    match response.refresh_token {
        Some(ref rt) if !rt.is_empty() => Ok(ValidatedTokenResponse {
            access_token: response.access_token,
            refresh_token: rt.clone(),
            expires_in: response.expires_in,
        }),
        _ => Err(OAuthFlowError::NoRefreshToken),
    }
}

/// Parsed parameters from an OAuth redirect callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

/// Parse the query string from an OAuth redirect callback URL.
///
/// Extracts `code` and `state`, or returns an error if the provider
/// returned an error response or required parameters are missing.
pub fn parse_callback_query(query: &str) -> Result<CallbackParams, OAuthFlowError> {
    let mut code = None;
    let mut state = None;
    let mut error = None;
    let mut error_description = None;

    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let value = kv.next().unwrap_or("");
        let decoded = percent_decode(value);
        match key {
            "code" => code = Some(decoded),
            "state" => state = Some(decoded),
            "error" => error = Some(decoded),
            "error_description" => error_description = Some(decoded),
            _ => {}
        }
    }

    if let Some(err) = error {
        let desc = error_description.unwrap_or_default();
        return Err(OAuthFlowError::CallbackError(if desc.is_empty() {
            err
        } else {
            format!("{err}: {desc}")
        }));
    }

    match (code, state) {
        (Some(c), Some(s)) if !c.is_empty() && !s.is_empty() => {
            Ok(CallbackParams { code: c, state: s })
        }
        _ => Err(OAuthFlowError::CallbackError(
            "Missing code or state parameter in callback".to_string(),
        )),
    }
}

/// Parse a JSON token response body into a `TokenResponse`.
pub fn parse_token_response_json(body: &str) -> Result<TokenResponse, OAuthFlowError> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("Invalid JSON: {e}")))?;

    let access_token = value["access_token"]
        .as_str()
        .ok_or_else(|| {
            OAuthFlowError::TokenExchangeFailed("Missing access_token in response".to_string())
        })?
        .to_string();

    let refresh_token = value["refresh_token"].as_str().map(String::from);
    let expires_in = value["expires_in"].as_u64();
    let id_token = value["id_token"].as_str().map(String::from);

    Ok(TokenResponse {
        access_token,
        refresh_token,
        expires_in,
        id_token,
    })
}

/// Compute the token expiry as a Unix epoch timestamp from an `expires_in` seconds value.
pub fn compute_expiry_epoch(expires_in: Option<u64>) -> Option<u64> {
    expires_in.map(|secs| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + secs
    })
}

/// Percent-encode a string per RFC 3986 unreserved character set.
pub(crate) fn percent_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                let _ = write!(result, "%{:02X}", byte);
            }
        }
    }
    result
}

/// Percent-decode a URL-encoded string (also handles `+` as space).
pub(crate) fn percent_decode(input: &str) -> String {
    let mut result = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                result.push(byte);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            result.push(b' ');
        } else {
            result.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&result).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::OAuthConfig;

    fn test_oauth_config() -> OAuthConfig {
        OAuthConfig {
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["https://mail.google.com/".to_string(), "openid".to_string()],
            client_id: Some("test-client-id.apps.googleusercontent.com".to_string()),
            extra_params: vec![
                ("prompt".to_string(), "consent".to_string()),
                ("access_type".to_string(), "offline".to_string()),
            ],
            userinfo_url: None,
        }
    }

    // --- PKCE tests ---

    #[test]
    fn pkce_verifier_is_base64url_encoded() {
        let pkce = PkceChallenge::generate();
        // Base64url uses A-Z, a-z, 0-9, -, _ (no padding with URL_SAFE_NO_PAD)
        assert!(pkce
            .verifier()
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn pkce_verifier_length_within_spec() {
        let pkce = PkceChallenge::generate();
        let len = pkce.verifier().len();
        // RFC 7636: verifier must be 43-128 characters
        assert!(
            (43..=128).contains(&len),
            "verifier length {len} out of range"
        );
    }

    #[test]
    fn pkce_challenge_is_s256_of_verifier() {
        let pkce = PkceChallenge::generate();
        let expected = URL_SAFE_NO_PAD.encode(Sha256::digest(pkce.verifier().as_bytes()));
        assert_eq!(pkce.challenge(), expected);
    }

    #[test]
    fn pkce_generate_is_random() {
        let a = PkceChallenge::generate();
        let b = PkceChallenge::generate();
        assert_ne!(a.verifier(), b.verifier());
        assert_ne!(a.challenge(), b.challenge());
    }

    #[test]
    fn pkce_new_test_computes_correct_challenge() {
        let pkce = PkceChallenge::new_test("test-verifier-value");
        let expected = URL_SAFE_NO_PAD.encode(Sha256::digest(b"test-verifier-value"));
        assert_eq!(pkce.challenge(), expected);
        assert_eq!(pkce.verifier(), "test-verifier-value");
    }

    // --- State tests ---

    #[test]
    fn state_is_base64url_encoded() {
        let state = generate_state();
        assert!(state
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn state_has_sufficient_entropy() {
        let state = generate_state();
        // 32 bytes of randomness → 43 base64url characters
        assert!(state.len() >= 43);
    }

    #[test]
    fn state_is_random() {
        let a = generate_state();
        let b = generate_state();
        assert_ne!(a, b);
    }

    // --- Session tests ---

    #[test]
    fn session_not_expired_when_fresh() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        assert!(!session.is_expired());
    }

    #[test]
    fn session_expired_after_timeout() {
        let session = OAuthSession::new(test_oauth_config(), 8080)
            .with_created_at(Instant::now() - Duration::from_secs(21 * 60));
        assert!(session.is_expired());
    }

    #[test]
    fn session_not_expired_just_before_timeout() {
        let session = OAuthSession::new(test_oauth_config(), 8080)
            .with_created_at(Instant::now() - Duration::from_secs(19 * 60));
        assert!(!session.is_expired());
    }

    #[test]
    fn validate_state_accepts_matching() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        let state = session.state().to_string();
        assert!(session.validate_state(Some(&state)).is_ok());
    }

    #[test]
    fn validate_state_rejects_mismatch() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        let result = session.validate_state(Some("wrong-state"));
        assert!(matches!(result, Err(OAuthFlowError::StateMismatch)));
    }

    #[test]
    fn validate_state_rejects_none() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        let result = session.validate_state(None);
        assert!(matches!(result, Err(OAuthFlowError::StateMismatch)));
    }

    #[test]
    fn validate_state_rejects_expired_even_if_matching() {
        let session = OAuthSession::new(test_oauth_config(), 8080)
            .with_created_at(Instant::now() - Duration::from_secs(21 * 60));
        let state = session.state().to_string();
        let result = session.validate_state(Some(&state));
        assert!(matches!(result, Err(OAuthFlowError::SessionExpired)));
    }

    // --- Authorization URL tests ---

    #[test]
    fn authorization_url_contains_response_type_code() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        let url = session.authorization_url();
        assert!(url.contains("response_type=code"));
    }

    #[test]
    fn authorization_url_contains_client_id() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        let url = session.authorization_url();
        assert!(url.contains("client_id=test-client-id.apps.googleusercontent.com"));
    }

    #[test]
    fn authorization_url_contains_redirect_uri_with_port() {
        let session = OAuthSession::new(test_oauth_config(), 12345);
        let url = session.authorization_url();
        assert!(url.contains(&percent_encode("http://127.0.0.1:12345/callback")));
    }

    #[test]
    fn authorization_url_contains_scopes() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        let url = session.authorization_url();
        // Scopes joined with space, which gets percent-encoded
        assert!(url.contains("scope="));
        assert!(url.contains("mail.google.com"));
    }

    #[test]
    fn authorization_url_contains_state() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        let state = session.state().to_string();
        let url = session.authorization_url();
        assert!(url.contains(&format!("state={state}")));
    }

    #[test]
    fn authorization_url_contains_pkce_challenge() {
        let pkce = PkceChallenge::new_test("test-verifier");
        let session =
            OAuthSession::new_with_values(test_oauth_config(), 8080, "s".to_string(), pkce.clone());
        let url = session.authorization_url();
        assert!(url.contains(&format!("code_challenge={}", pkce.challenge())));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[test]
    fn authorization_url_contains_extra_params() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        let url = session.authorization_url();
        assert!(url.contains("prompt=consent"));
        assert!(url.contains("access_type=offline"));
    }

    #[test]
    fn authorization_url_starts_with_auth_endpoint() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        let url = session.authorization_url();
        assert!(url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
    }

    #[test]
    fn authorization_url_omits_client_id_when_none() {
        let mut config = test_oauth_config();
        config.client_id = None;
        let session = OAuthSession::new(config, 8080);
        let url = session.authorization_url();
        assert!(!url.contains("client_id="));
    }

    // --- Token exchange params tests ---

    #[test]
    fn token_exchange_params_has_correct_fields() {
        let session = OAuthSession::new(test_oauth_config(), 9999);
        let params = session.token_exchange_params("auth-code-123");
        assert_eq!(params.token_url, "https://oauth2.googleapis.com/token");
        assert_eq!(params.code, "auth-code-123");
        assert_eq!(params.redirect_uri, "http://127.0.0.1:9999/callback");
        assert_eq!(
            params.client_id,
            "test-client-id.apps.googleusercontent.com"
        );
        assert!(!params.code_verifier.is_empty());
    }

    #[test]
    fn token_exchange_form_body_contains_grant_type() {
        let session = OAuthSession::new(test_oauth_config(), 8080);
        let params = session.token_exchange_params("code");
        let body = params.form_body();
        assert!(body.contains("grant_type=authorization_code"));
    }

    #[test]
    fn token_exchange_form_body_contains_code_verifier() {
        let pkce = PkceChallenge::new_test("my-verifier");
        let session =
            OAuthSession::new_with_values(test_oauth_config(), 8080, "s".to_string(), pkce);
        let params = session.token_exchange_params("code");
        let body = params.form_body();
        assert!(body.contains("code_verifier=my-verifier"));
    }

    // --- Token response validation tests ---

    #[test]
    fn validate_token_response_accepts_with_refresh_token() {
        let response = TokenResponse {
            access_token: "access-123".to_string(),
            refresh_token: Some("refresh-456".to_string()),
            expires_in: Some(3600),
            id_token: None,
        };
        let validated = validate_token_response(response).unwrap();
        assert_eq!(validated.access_token, "access-123");
        assert_eq!(validated.refresh_token, "refresh-456");
        assert_eq!(validated.expires_in, Some(3600));
    }

    #[test]
    fn validate_token_response_rejects_missing_refresh_token() {
        let response = TokenResponse {
            access_token: "access-123".to_string(),
            refresh_token: None,
            expires_in: Some(3600),
            id_token: None,
        };
        let result = validate_token_response(response);
        assert!(matches!(result, Err(OAuthFlowError::NoRefreshToken)));
    }

    #[test]
    fn validate_token_response_rejects_empty_refresh_token() {
        let response = TokenResponse {
            access_token: "access-123".to_string(),
            refresh_token: Some(String::new()),
            expires_in: Some(3600),
            id_token: None,
        };
        let result = validate_token_response(response);
        assert!(matches!(result, Err(OAuthFlowError::NoRefreshToken)));
    }

    #[test]
    fn validate_token_response_accepts_without_expires_in() {
        let response = TokenResponse {
            access_token: "access".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_in: None,
            id_token: None,
        };
        let validated = validate_token_response(response).unwrap();
        assert!(validated.expires_in.is_none());
    }

    // --- Callback parsing tests ---

    #[test]
    fn parse_callback_extracts_code_and_state() {
        let params = parse_callback_query("code=abc123&state=xyz789").unwrap();
        assert_eq!(params.code, "abc123");
        assert_eq!(params.state, "xyz789");
    }

    #[test]
    fn parse_callback_handles_url_encoded_values() {
        let params = parse_callback_query("code=abc%20123&state=xyz%3D789").unwrap();
        assert_eq!(params.code, "abc 123");
        assert_eq!(params.state, "xyz=789");
    }

    #[test]
    fn parse_callback_handles_plus_as_space() {
        let params = parse_callback_query("code=abc+123&state=xyz").unwrap();
        assert_eq!(params.code, "abc 123");
    }

    #[test]
    fn parse_callback_ignores_extra_params() {
        let params =
            parse_callback_query("code=abc&state=xyz&session_state=ignored&extra=also").unwrap();
        assert_eq!(params.code, "abc");
        assert_eq!(params.state, "xyz");
    }

    #[test]
    fn parse_callback_returns_error_on_provider_error() {
        let result = parse_callback_query("error=access_denied");
        assert!(matches!(result, Err(OAuthFlowError::CallbackError(_))));
        if let Err(OAuthFlowError::CallbackError(msg)) = result {
            assert!(msg.contains("access_denied"));
        }
    }

    #[test]
    fn parse_callback_returns_error_with_description() {
        let result =
            parse_callback_query("error=access_denied&error_description=User+cancelled+login");
        if let Err(OAuthFlowError::CallbackError(msg)) = result {
            assert!(msg.contains("access_denied"));
            assert!(msg.contains("User cancelled login"));
        } else {
            panic!("Expected CallbackError");
        }
    }

    #[test]
    fn parse_callback_rejects_missing_code() {
        let result = parse_callback_query("state=xyz");
        assert!(matches!(result, Err(OAuthFlowError::CallbackError(_))));
    }

    #[test]
    fn parse_callback_rejects_missing_state() {
        let result = parse_callback_query("code=abc");
        assert!(matches!(result, Err(OAuthFlowError::CallbackError(_))));
    }

    #[test]
    fn parse_callback_rejects_empty_code() {
        let result = parse_callback_query("code=&state=xyz");
        assert!(matches!(result, Err(OAuthFlowError::CallbackError(_))));
    }

    #[test]
    fn parse_callback_rejects_empty_state() {
        let result = parse_callback_query("code=abc&state=");
        assert!(matches!(result, Err(OAuthFlowError::CallbackError(_))));
    }

    // --- JSON token response parsing ---

    #[test]
    fn parse_json_full_response() {
        let json = r#"{"access_token":"ya29.abc","refresh_token":"1//xyz","expires_in":3600,"token_type":"Bearer"}"#;
        let response = parse_token_response_json(json).unwrap();
        assert_eq!(response.access_token, "ya29.abc");
        assert_eq!(response.refresh_token, Some("1//xyz".to_string()));
        assert_eq!(response.expires_in, Some(3600));
    }

    #[test]
    fn parse_json_without_refresh_token() {
        let json = r#"{"access_token":"ya29.abc","expires_in":3600}"#;
        let response = parse_token_response_json(json).unwrap();
        assert_eq!(response.access_token, "ya29.abc");
        assert!(response.refresh_token.is_none());
    }

    #[test]
    fn parse_json_without_expires_in() {
        let json = r#"{"access_token":"ya29.abc","refresh_token":"1//xyz"}"#;
        let response = parse_token_response_json(json).unwrap();
        assert!(response.expires_in.is_none());
    }

    #[test]
    fn parse_json_rejects_missing_access_token() {
        let json = r#"{"refresh_token":"1//xyz","expires_in":3600}"#;
        let result = parse_token_response_json(json);
        assert!(matches!(
            result,
            Err(OAuthFlowError::TokenExchangeFailed(_))
        ));
    }

    #[test]
    fn parse_json_rejects_invalid_json() {
        let result = parse_token_response_json("not json");
        assert!(matches!(
            result,
            Err(OAuthFlowError::TokenExchangeFailed(_))
        ));
    }

    // --- Expiry computation ---

    #[test]
    fn compute_expiry_epoch_returns_future_timestamp() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiry = compute_expiry_epoch(Some(3600)).unwrap();
        assert!(expiry >= now + 3599);
        assert!(expiry <= now + 3601);
    }

    #[test]
    fn compute_expiry_epoch_returns_none_for_none() {
        assert!(compute_expiry_epoch(None).is_none());
    }

    // --- Percent encoding/decoding ---

    #[test]
    fn percent_encode_unreserved_chars_unchanged() {
        assert_eq!(percent_encode("abcXYZ0189-_.~"), "abcXYZ0189-_.~");
    }

    #[test]
    fn percent_encode_spaces() {
        assert_eq!(percent_encode("hello world"), "hello%20world");
    }

    #[test]
    fn percent_encode_special_chars() {
        assert_eq!(percent_encode("a=b&c"), "a%3Db%26c");
    }

    #[test]
    fn percent_encode_url() {
        let encoded = percent_encode("http://127.0.0.1:8080/callback");
        assert_eq!(encoded, "http%3A%2F%2F127.0.0.1%3A8080%2Fcallback");
    }

    #[test]
    fn percent_decode_basic() {
        assert_eq!(percent_decode("hello%20world"), "hello world");
    }

    #[test]
    fn percent_decode_plus_as_space() {
        assert_eq!(percent_decode("hello+world"), "hello world");
    }

    #[test]
    fn percent_decode_special_chars() {
        assert_eq!(percent_decode("a%3Db%26c"), "a=b&c");
    }

    #[test]
    fn percent_decode_passthrough_unreserved() {
        assert_eq!(percent_decode("abcXYZ0189-_.~"), "abcXYZ0189-_.~");
    }

    #[test]
    fn percent_encode_decode_roundtrip() {
        let original = "https://mail.google.com/ openid email";
        let encoded = percent_encode(original);
        let decoded = percent_decode(&encoded);
        assert_eq!(decoded, original);
    }

    // --- Per-account independence (FR-14) ---

    #[test]
    fn different_sessions_have_different_state() {
        let s1 = OAuthSession::new(test_oauth_config(), 8080);
        let s2 = OAuthSession::new(test_oauth_config(), 8081);
        assert_ne!(s1.state(), s2.state());
    }

    // --- Session timeout constant ---

    #[test]
    fn session_timeout_is_20_minutes() {
        assert_eq!(SESSION_TIMEOUT, Duration::from_secs(20 * 60));
    }

    // --- End-to-end flow simulation ---

    #[test]
    fn full_flow_simulation() {
        let config = test_oauth_config();
        let pkce = PkceChallenge::new_test("test-verifier-12345");
        let state = "test-state-abc".to_string();
        let session = OAuthSession::new_with_values(config, 54321, state.clone(), pkce);

        // Step 1: Build auth URL
        let url = session.authorization_url();
        assert!(url.starts_with("https://accounts.google.com/"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state=test-state-abc"));

        // Step 2: Simulate callback
        let query = format!("code=auth-code-xyz&state={state}");
        let params = parse_callback_query(&query).unwrap();
        assert_eq!(params.code, "auth-code-xyz");

        // Step 3: Validate state
        session.validate_state(Some(&params.state)).unwrap();

        // Step 4: Get exchange params
        let exchange = session.token_exchange_params(&params.code);
        assert_eq!(exchange.code, "auth-code-xyz");
        assert_eq!(exchange.code_verifier, "test-verifier-12345");
        assert_eq!(exchange.redirect_uri, "http://127.0.0.1:54321/callback");

        // Step 5: Simulate token response
        let json = r#"{"access_token":"ya29.xxx","refresh_token":"1//yyy","expires_in":3600}"#;
        let response = parse_token_response_json(json).unwrap();
        let validated = validate_token_response(response).unwrap();
        assert_eq!(validated.access_token, "ya29.xxx");
        assert_eq!(validated.refresh_token, "1//yyy");
    }
}
