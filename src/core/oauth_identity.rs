use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

use crate::core::oauth_flow::OAuthFlowError;
use crate::core::provider::OAuthConfig;

/// Identity information extracted from an OAuth token response or user-info
/// endpoint (FR-34, FR-35, FR-36).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityInfo {
    /// The user's email address.
    pub email: String,
    /// The user's display name, if available.
    pub display_name: Option<String>,
}

/// The result of attempting to extract identity from the OAuth flow.
/// Tells the caller whether identity was found or manual entry is needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityExtractionResult {
    /// Identity was successfully extracted (from ID token or user-info endpoint).
    Extracted(IdentityInfo),
    /// Neither token nor user-info yielded a usable email; prompt the user (FR-36).
    ManualEntryRequired,
}

/// Check whether the provider requires a user-info endpoint fetch to obtain
/// identity information (FR-35, N-9). Returns `true` when the provider's
/// `OAuthConfig` has a `userinfo_url` set (analogous to `askAccount` in the
/// Android codebase).
pub fn needs_userinfo_fetch(config: &OAuthConfig) -> bool {
    config.userinfo_url.is_some()
}

/// Extract identity claims (email, name) from an OpenID Connect ID token JWT
/// (FR-34).
///
/// The ID token is decoded but **not** cryptographically verified — we only
/// use it to pre-fill the user's email and display name, not for
/// authentication decisions. The token has already been received over a
/// validated TLS connection from the provider's token endpoint.
pub fn extract_identity_from_id_token(id_token: &str) -> Option<IdentityInfo> {
    // JWT structure: header.payload.signature
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let payload_bytes = decode_jwt_segment(parts[1])?;
    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).ok()?;

    let email = payload["email"].as_str().filter(|s| !s.is_empty())?;

    let display_name = payload["name"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(String::from);

    Some(IdentityInfo {
        email: email.to_string(),
        display_name,
    })
}

/// Parse a JSON response from a provider's user-info endpoint (FR-35).
///
/// Supports the common claim names used by major providers:
/// - `email` (standard OpenID Connect)
/// - `name` / `display_name` / `real_name` for the display name
pub fn parse_userinfo_json(body: &str) -> Option<IdentityInfo> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;

    let email = value["email"].as_str().filter(|s| !s.is_empty())?;

    let display_name = value["name"]
        .as_str()
        .or_else(|| value["display_name"].as_str())
        .or_else(|| value["real_name"].as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    Some(IdentityInfo {
        email: email.to_string(),
        display_name,
    })
}

/// Attempt to extract identity from the token response, falling back to
/// a user-info fetch result if provided (FR-34, FR-35, FR-36).
///
/// The `id_token` comes from the OAuth token response. The `userinfo_json`
/// is the body from fetching the provider's user-info endpoint (only
/// attempted when `needs_userinfo_fetch()` returns `true`).
///
/// Returns `ManualEntryRequired` if neither source yields a usable email.
pub fn resolve_identity(
    id_token: Option<&str>,
    userinfo_json: Option<&str>,
) -> IdentityExtractionResult {
    // Try ID token first
    if let Some(token) = id_token {
        if let Some(info) = extract_identity_from_id_token(token) {
            return IdentityExtractionResult::Extracted(info);
        }
    }

    // Fall back to user-info endpoint response
    if let Some(json) = userinfo_json {
        if let Some(info) = parse_userinfo_json(json) {
            return IdentityExtractionResult::Extracted(info);
        }
    }

    IdentityExtractionResult::ManualEntryRequired
}

/// Build the userinfo URL from the OAuthConfig, if configured.
pub fn userinfo_url(config: &OAuthConfig) -> Option<&str> {
    config.userinfo_url.as_deref()
}

/// Validate that a userinfo fetch response has a successful HTTP status
/// and extract the JSON body. Returns an error if the fetch failed.
pub fn validate_userinfo_response(
    status_ok: bool,
    body: &str,
) -> Result<Option<IdentityInfo>, OAuthFlowError> {
    if !status_ok {
        return Err(OAuthFlowError::TokenExchangeFailed(format!(
            "User-info endpoint returned error: {body}"
        )));
    }
    Ok(parse_userinfo_json(body))
}

/// Decode a base64url-encoded JWT segment, handling missing padding.
fn decode_jwt_segment(segment: &str) -> Option<Vec<u8>> {
    // JWT segments use base64url without padding; URL_SAFE_NO_PAD handles this.
    // Some providers may include padding — strip it to be safe.
    let stripped = segment.trim_end_matches('=');
    URL_SAFE_NO_PAD.decode(stripped).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::OAuthProfileStatus;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    /// Build a minimal JWT with the given payload claims.
    fn make_jwt(claims: &serde_json::Value) -> String {
        let header = URL_SAFE_NO_PAD.encode(b"{\"alg\":\"RS256\",\"typ\":\"JWT\"}");
        let payload = URL_SAFE_NO_PAD.encode(claims.to_string().as_bytes());
        let signature = URL_SAFE_NO_PAD.encode(b"fake-signature");
        format!("{header}.{payload}.{signature}")
    }

    fn test_oauth_config_no_userinfo() -> OAuthConfig {
        OAuthConfig {
            auth_url: "https://auth.example.com".to_string(),
            token_url: "https://token.example.com".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["mail".to_string()],
            client_id: None,
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: None,
            privacy_policy_url: None,
            client_secret: None,
            status: OAuthProfileStatus::Enabled,
        }
    }

    fn test_oauth_config_with_userinfo() -> OAuthConfig {
        OAuthConfig {
            auth_url: "https://auth.example.com".to_string(),
            token_url: "https://token.example.com".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["mail".to_string()],
            client_id: None,
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: Some("https://oauth.mail.ru/userinfo".to_string()),
            privacy_policy_url: None,
            client_secret: None,
            status: OAuthProfileStatus::Enabled,
        }
    }

    // --- needs_userinfo_fetch ---

    #[test]
    fn needs_userinfo_fetch_false_when_no_url() {
        assert!(!needs_userinfo_fetch(&test_oauth_config_no_userinfo()));
    }

    #[test]
    fn needs_userinfo_fetch_true_when_url_set() {
        assert!(needs_userinfo_fetch(&test_oauth_config_with_userinfo()));
    }

    // --- extract_identity_from_id_token ---

    #[test]
    fn extracts_email_and_name_from_id_token() {
        let claims = serde_json::json!({
            "sub": "12345",
            "email": "user@gmail.com",
            "name": "Test User",
            "iss": "https://accounts.google.com"
        });
        let jwt = make_jwt(&claims);
        let info = extract_identity_from_id_token(&jwt).unwrap();
        assert_eq!(info.email, "user@gmail.com");
        assert_eq!(info.display_name, Some("Test User".to_string()));
    }

    #[test]
    fn extracts_email_only_when_no_name() {
        let claims = serde_json::json!({
            "email": "user@outlook.com"
        });
        let jwt = make_jwt(&claims);
        let info = extract_identity_from_id_token(&jwt).unwrap();
        assert_eq!(info.email, "user@outlook.com");
        assert!(info.display_name.is_none());
    }

    #[test]
    fn returns_none_when_no_email_in_token() {
        let claims = serde_json::json!({
            "sub": "12345",
            "name": "Test User"
        });
        let jwt = make_jwt(&claims);
        assert!(extract_identity_from_id_token(&jwt).is_none());
    }

    #[test]
    fn returns_none_for_empty_email() {
        let claims = serde_json::json!({
            "email": "",
            "name": "Test"
        });
        let jwt = make_jwt(&claims);
        assert!(extract_identity_from_id_token(&jwt).is_none());
    }

    #[test]
    fn returns_none_for_invalid_jwt_structure() {
        assert!(extract_identity_from_id_token("not-a-jwt").is_none());
        assert!(extract_identity_from_id_token("only.two").is_none());
        assert!(extract_identity_from_id_token("").is_none());
    }

    #[test]
    fn returns_none_for_invalid_base64_payload() {
        assert!(extract_identity_from_id_token("header.!!!invalid!!!.sig").is_none());
    }

    #[test]
    fn returns_none_for_non_json_payload() {
        let header = URL_SAFE_NO_PAD.encode(b"{}");
        let payload = URL_SAFE_NO_PAD.encode(b"not json");
        let sig = URL_SAFE_NO_PAD.encode(b"sig");
        let jwt = format!("{header}.{payload}.{sig}");
        assert!(extract_identity_from_id_token(&jwt).is_none());
    }

    #[test]
    fn ignores_empty_name_in_token() {
        let claims = serde_json::json!({
            "email": "user@example.com",
            "name": ""
        });
        let jwt = make_jwt(&claims);
        let info = extract_identity_from_id_token(&jwt).unwrap();
        assert!(info.display_name.is_none());
    }

    // --- parse_userinfo_json ---

    #[test]
    fn parses_standard_userinfo_response() {
        let json = r#"{"email": "user@mail.ru", "name": "Иван Петров"}"#;
        let info = parse_userinfo_json(json).unwrap();
        assert_eq!(info.email, "user@mail.ru");
        assert_eq!(info.display_name, Some("Иван Петров".to_string()));
    }

    #[test]
    fn parses_userinfo_with_display_name_field() {
        let json = r#"{"email": "user@example.com", "display_name": "John Doe"}"#;
        let info = parse_userinfo_json(json).unwrap();
        assert_eq!(info.display_name, Some("John Doe".to_string()));
    }

    #[test]
    fn parses_userinfo_with_real_name_field() {
        let json = r#"{"email": "user@example.com", "real_name": "Jane Smith"}"#;
        let info = parse_userinfo_json(json).unwrap();
        assert_eq!(info.display_name, Some("Jane Smith".to_string()));
    }

    #[test]
    fn parses_userinfo_email_only() {
        let json = r#"{"email": "user@mail.ru"}"#;
        let info = parse_userinfo_json(json).unwrap();
        assert_eq!(info.email, "user@mail.ru");
        assert!(info.display_name.is_none());
    }

    #[test]
    fn returns_none_for_missing_email_in_userinfo() {
        let json = r#"{"name": "Test User"}"#;
        assert!(parse_userinfo_json(json).is_none());
    }

    #[test]
    fn returns_none_for_empty_email_in_userinfo() {
        let json = r#"{"email": "", "name": "Test"}"#;
        assert!(parse_userinfo_json(json).is_none());
    }

    #[test]
    fn returns_none_for_invalid_json_userinfo() {
        assert!(parse_userinfo_json("not json").is_none());
    }

    #[test]
    fn name_field_takes_priority_over_alternatives() {
        let json = r#"{"email": "u@x.com", "name": "Primary", "display_name": "Alt", "real_name": "Other"}"#;
        let info = parse_userinfo_json(json).unwrap();
        assert_eq!(info.display_name, Some("Primary".to_string()));
    }

    // --- resolve_identity ---

    #[test]
    fn resolve_uses_id_token_first() {
        let claims = serde_json::json!({"email": "token@example.com", "name": "Token User"});
        let jwt = make_jwt(&claims);
        let userinfo = r#"{"email": "info@example.com", "name": "Info User"}"#;

        let result = resolve_identity(Some(&jwt), Some(userinfo));
        match result {
            IdentityExtractionResult::Extracted(info) => {
                assert_eq!(info.email, "token@example.com");
                assert_eq!(info.display_name, Some("Token User".to_string()));
            }
            _ => panic!("Expected Extracted"),
        }
    }

    #[test]
    fn resolve_falls_back_to_userinfo() {
        let result = resolve_identity(None, Some(r#"{"email": "info@mail.ru", "name": "Info"}"#));
        match result {
            IdentityExtractionResult::Extracted(info) => {
                assert_eq!(info.email, "info@mail.ru");
            }
            _ => panic!("Expected Extracted"),
        }
    }

    #[test]
    fn resolve_falls_back_when_id_token_has_no_email() {
        let claims = serde_json::json!({"sub": "123"});
        let jwt = make_jwt(&claims);
        let result = resolve_identity(Some(&jwt), Some(r#"{"email": "fallback@example.com"}"#));
        match result {
            IdentityExtractionResult::Extracted(info) => {
                assert_eq!(info.email, "fallback@example.com");
            }
            _ => panic!("Expected Extracted"),
        }
    }

    #[test]
    fn resolve_returns_manual_entry_when_neither_works() {
        let result = resolve_identity(None, None);
        assert_eq!(result, IdentityExtractionResult::ManualEntryRequired);
    }

    #[test]
    fn resolve_returns_manual_entry_when_both_lack_email() {
        let claims = serde_json::json!({"sub": "123"});
        let jwt = make_jwt(&claims);
        let result = resolve_identity(Some(&jwt), Some(r#"{"name": "No Email"}"#));
        assert_eq!(result, IdentityExtractionResult::ManualEntryRequired);
    }

    // --- validate_userinfo_response ---

    #[test]
    fn validate_userinfo_response_success() {
        let result =
            validate_userinfo_response(true, r#"{"email": "user@mail.ru", "name": "Test"}"#)
                .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().email, "user@mail.ru");
    }

    #[test]
    fn validate_userinfo_response_success_no_email() {
        let result = validate_userinfo_response(true, r#"{"name": "No Email"}"#).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn validate_userinfo_response_error_status() {
        let result = validate_userinfo_response(false, "Unauthorized");
        assert!(matches!(
            result,
            Err(OAuthFlowError::TokenExchangeFailed(_))
        ));
    }

    // --- userinfo_url helper ---

    #[test]
    fn userinfo_url_returns_none_when_not_set() {
        assert!(userinfo_url(&test_oauth_config_no_userinfo()).is_none());
    }

    #[test]
    fn userinfo_url_returns_url_when_set() {
        assert_eq!(
            userinfo_url(&test_oauth_config_with_userinfo()),
            Some("https://oauth.mail.ru/userinfo")
        );
    }

    // --- Mail.ru provider has userinfo_url in bundled database ---

    #[test]
    fn bundled_mailru_has_userinfo_url() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("mail.ru").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(needs_userinfo_fetch(oauth));
        assert_eq!(
            oauth.userinfo_url,
            Some("https://oauth.mail.ru/userinfo".to_string())
        );
    }

    #[test]
    fn bundled_gmail_does_not_need_userinfo_fetch() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("gmail.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(!needs_userinfo_fetch(oauth));
    }

    // --- JWT padding tolerance ---

    #[test]
    fn handles_jwt_with_padding() {
        // Some providers might include base64 padding in JWT segments
        let claims = serde_json::json!({"email": "pad@example.com"});
        let header = format!("{}=", URL_SAFE_NO_PAD.encode(b"{\"alg\":\"RS256\"}"));
        let payload = format!("{}=", URL_SAFE_NO_PAD.encode(claims.to_string().as_bytes()));
        let sig = URL_SAFE_NO_PAD.encode(b"sig");
        let jwt = format!("{header}.{payload}.{sig}");
        let info = extract_identity_from_id_token(&jwt).unwrap();
        assert_eq!(info.email, "pad@example.com");
    }
}
