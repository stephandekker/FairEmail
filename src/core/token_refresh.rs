use std::time::SystemTime;

use crate::core::oauth_flow::{percent_encode, OAuthFlowError};

/// Margin in seconds before expiry at which a proactive refresh is triggered (FR-16).
/// Refreshing 5 minutes early ensures the token is always valid when used.
const REFRESH_MARGIN_SECS: u64 = 5 * 60;

/// Minimum interval in seconds between consecutive refresh attempts for the
/// same account, mirroring the Android app's 15-minute guard (Design Note N-3).
pub(crate) const MIN_REFRESH_INTERVAL_SECS: u64 = 15 * 60;

/// Determine whether an access token should be refreshed proactively.
///
/// Returns `true` when the token has expired **or** will expire within
/// [`REFRESH_MARGIN_SECS`]. If no expiry is stored, the token is assumed
/// to have a [`DEFAULT_LIFETIME_SECS`] lifetime from when it was issued;
/// since we cannot know when it was issued without an expiry timestamp we
/// conservatively return `true` to trigger a refresh.
pub(crate) fn should_refresh_token(expiry_epoch: Option<u64>) -> bool {
    let now = now_epoch();
    match expiry_epoch {
        Some(expiry) => now + REFRESH_MARGIN_SECS >= expiry,
        // No expiry recorded — conservatively refresh.
        None => true,
    }
}

/// Whether enough time has elapsed since the last refresh to allow another one.
pub(crate) fn may_refresh_now(last_refresh_epoch: Option<u64>) -> bool {
    match last_refresh_epoch {
        Some(last) => now_epoch() >= last + MIN_REFRESH_INTERVAL_SECS,
        None => true,
    }
}

/// Parameters for an OAuth2 token refresh HTTP POST request.
pub(crate) struct RefreshTokenParams {
    pub token_url: String,
    pub client_id: String,
    pub refresh_token: String,
}

impl RefreshTokenParams {
    /// Build the form-encoded body for the refresh request.
    pub(crate) fn form_body(&self) -> String {
        [
            ("grant_type", "refresh_token"),
            ("refresh_token", &self.refresh_token),
            ("client_id", &self.client_id),
        ]
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
    }
}

/// A successfully parsed token refresh response.
///
/// The refresh token field is optional because many providers do not rotate
/// refresh tokens on every refresh. When `None`, the existing refresh token
/// in the credential store must be kept.
#[derive(Debug, Clone)]
pub(crate) struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
}

/// Parse a JSON token-refresh response body.
pub(crate) fn parse_refresh_response_json(body: &str) -> Result<RefreshResponse, OAuthFlowError> {
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

    Ok(RefreshResponse {
        access_token,
        refresh_token,
        expires_in,
    })
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- should_refresh_token ---

    #[test]
    fn refresh_needed_when_expired() {
        let past = now_epoch() - 60;
        assert!(should_refresh_token(Some(past)));
    }

    #[test]
    fn refresh_needed_within_margin() {
        // Token expires in 2 minutes — within the 5-minute margin
        let soon = now_epoch() + 120;
        assert!(should_refresh_token(Some(soon)));
    }

    #[test]
    fn no_refresh_when_far_from_expiry() {
        let far = now_epoch() + 3600;
        assert!(!should_refresh_token(Some(far)));
    }

    #[test]
    fn refresh_needed_when_no_expiry() {
        assert!(should_refresh_token(None));
    }

    #[test]
    fn refresh_at_exact_margin_boundary() {
        // Expires exactly at now + margin → should refresh (>= comparison)
        let boundary = now_epoch() + REFRESH_MARGIN_SECS;
        assert!(should_refresh_token(Some(boundary)));
    }

    #[test]
    fn no_refresh_one_second_past_margin() {
        let just_safe = now_epoch() + REFRESH_MARGIN_SECS + 1;
        assert!(!should_refresh_token(Some(just_safe)));
    }

    // --- may_refresh_now ---

    #[test]
    fn may_refresh_when_no_prior_refresh() {
        assert!(may_refresh_now(None));
    }

    #[test]
    fn may_not_refresh_too_soon() {
        let recent = now_epoch() - 60; // 1 minute ago
        assert!(!may_refresh_now(Some(recent)));
    }

    #[test]
    fn may_refresh_after_interval() {
        let old = now_epoch() - MIN_REFRESH_INTERVAL_SECS - 1;
        assert!(may_refresh_now(Some(old)));
    }

    #[test]
    fn may_refresh_at_exact_interval() {
        let exact = now_epoch() - MIN_REFRESH_INTERVAL_SECS;
        assert!(may_refresh_now(Some(exact)));
    }

    // --- RefreshTokenParams ---

    #[test]
    fn form_body_contains_grant_type_refresh() {
        let params = RefreshTokenParams {
            token_url: "https://example.com/token".to_string(),
            client_id: "my-client".to_string(),
            refresh_token: "rt-123".to_string(),
        };
        let body = params.form_body();
        assert!(body.contains("grant_type=refresh_token"));
        assert!(body.contains("refresh_token=rt-123"));
        assert!(body.contains("client_id=my-client"));
    }

    #[test]
    fn form_body_encodes_special_chars() {
        let params = RefreshTokenParams {
            token_url: "https://example.com/token".to_string(),
            client_id: "id with spaces".to_string(),
            refresh_token: "1//token+value".to_string(),
        };
        let body = params.form_body();
        assert!(!body.contains(' '));
        assert!(body.contains("id%20with%20spaces"));
        assert!(body.contains("1%2F%2Ftoken%2Bvalue"));
    }

    // --- parse_refresh_response_json ---

    #[test]
    fn parse_full_refresh_response() {
        let json =
            r#"{"access_token":"new-access","refresh_token":"new-refresh","expires_in":7200}"#;
        let resp = parse_refresh_response_json(json).unwrap();
        assert_eq!(resp.access_token, "new-access");
        assert_eq!(resp.refresh_token, Some("new-refresh".to_string()));
        assert_eq!(resp.expires_in, Some(7200));
    }

    #[test]
    fn parse_refresh_response_without_new_refresh_token() {
        let json = r#"{"access_token":"new-access","expires_in":3600}"#;
        let resp = parse_refresh_response_json(json).unwrap();
        assert_eq!(resp.access_token, "new-access");
        assert!(resp.refresh_token.is_none());
        assert_eq!(resp.expires_in, Some(3600));
    }

    #[test]
    fn parse_refresh_response_without_expires_in() {
        let json = r#"{"access_token":"new-access"}"#;
        let resp = parse_refresh_response_json(json).unwrap();
        assert_eq!(resp.access_token, "new-access");
        assert!(resp.expires_in.is_none());
    }

    #[test]
    fn parse_refresh_response_rejects_missing_access_token() {
        let json = r#"{"refresh_token":"rt","expires_in":3600}"#;
        assert!(parse_refresh_response_json(json).is_err());
    }

    #[test]
    fn parse_refresh_response_rejects_invalid_json() {
        assert!(parse_refresh_response_json("not json").is_err());
    }

    // --- Constants ---

    #[test]
    fn refresh_margin_is_five_minutes() {
        assert_eq!(REFRESH_MARGIN_SECS, 300);
    }

    #[test]
    fn min_refresh_interval_is_fifteen_minutes() {
        assert_eq!(MIN_REFRESH_INTERVAL_SECS, 900);
    }
}
