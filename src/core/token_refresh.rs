use std::time::{Duration, SystemTime};

use crate::core::oauth_flow::{percent_encode, OAuthFlowError};

/// Margin in seconds before expiry at which a proactive refresh is triggered (FR-16).
/// Refreshing 5 minutes early ensures the token is always valid when used.
const REFRESH_MARGIN_SECS: u64 = 5 * 60;

/// Minimum interval in seconds between consecutive refresh attempts for the
/// same account, mirroring the Android app's 15-minute guard (Design Note N-3).
pub(crate) const MIN_REFRESH_INTERVAL_SECS: u64 = 15 * 60;

/// Maximum total time to retry transient failures before declaring permanent (NFR-2).
pub(crate) const TRANSIENT_FAILURE_TIMEOUT_SECS: u64 = 90;

/// Backoff schedule for retry attempts (seconds). Total: 5+10+20+30+25 = 90s (NFR-2).
pub(crate) const RETRY_BACKOFF_SECS: &[u64] = &[5, 10, 20, 30, 25];

/// Classification of a token refresh error (FR-17, FR-18).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RefreshErrorKind {
    /// Network timeout, DNS failure, server 5xx — safe to retry.
    Transient,
    /// Refresh token revoked, invalid_grant, HTTP 400/401 — requires re-authorization.
    Permanent,
    /// Provider rate-limited or temporarily blocked (HTTP 429) — wait and retry.
    RateLimited,
}

/// The outcome of a refresh attempt with retry (FR-17, FR-18).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RefreshOutcome {
    /// Token was refreshed successfully.
    Refreshed,
    /// No refresh was needed or another refresh is in progress.
    Skipped,
    /// Account must be re-authorized (permanent failure or transient timeout).
    NeedsReauthorization {
        /// User-facing message explaining why re-auth is needed.
        reason: String,
    },
    /// Provider rate-limited the request; user should wait.
    RateLimited {
        /// User-facing message about the rate limit.
        reason: String,
    },
}

/// Classify an HTTP status code and response body into a `RefreshErrorKind`.
///
/// HTTP 400 with `invalid_grant` → Permanent (revoked refresh token).
/// HTTP 401 → Permanent (unauthorized / consent withdrawn).
/// HTTP 429 → RateLimited.
/// HTTP 5xx → Transient (server error).
/// Network/connection errors → Transient.
pub(crate) fn classify_refresh_error(status_code: u16, body: &str) -> RefreshErrorKind {
    match status_code {
        400 => {
            let lower = body.to_lowercase();
            if lower.contains("invalid_grant")
                || lower.contains("consent")
                || lower.contains("revoked")
                || lower.contains("expired")
            {
                RefreshErrorKind::Permanent
            } else {
                // Other 400 errors (e.g. malformed request) are also permanent
                // since retrying won't change the outcome.
                RefreshErrorKind::Permanent
            }
        }
        401 | 403 => RefreshErrorKind::Permanent,
        429 => RefreshErrorKind::RateLimited,
        500..=599 => RefreshErrorKind::Transient,
        // 0 means we never got an HTTP response (connection/network error).
        0 => RefreshErrorKind::Transient,
        _ => RefreshErrorKind::Transient,
    }
}

/// Return the backoff duration for a given retry attempt (0-indexed).
pub(crate) fn retry_backoff(attempt: usize) -> Duration {
    let idx = attempt.min(RETRY_BACKOFF_SECS.len() - 1);
    Duration::from_secs(RETRY_BACKOFF_SECS[idx])
}

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

    // --- Error classification ---

    #[test]
    fn classify_400_invalid_grant_as_permanent() {
        let kind = classify_refresh_error(400, r#"{"error":"invalid_grant"}"#);
        assert_eq!(kind, RefreshErrorKind::Permanent);
    }

    #[test]
    fn classify_400_revoked_as_permanent() {
        let kind = classify_refresh_error(400, r#"{"error":"revoked"}"#);
        assert_eq!(kind, RefreshErrorKind::Permanent);
    }

    #[test]
    fn classify_400_consent_as_permanent() {
        let kind = classify_refresh_error(400, "consent required");
        assert_eq!(kind, RefreshErrorKind::Permanent);
    }

    #[test]
    fn classify_400_expired_as_permanent() {
        let kind = classify_refresh_error(400, "Token has expired");
        assert_eq!(kind, RefreshErrorKind::Permanent);
    }

    #[test]
    fn classify_400_other_as_permanent() {
        let kind = classify_refresh_error(400, "bad request");
        assert_eq!(kind, RefreshErrorKind::Permanent);
    }

    #[test]
    fn classify_401_as_permanent() {
        let kind = classify_refresh_error(401, "unauthorized");
        assert_eq!(kind, RefreshErrorKind::Permanent);
    }

    #[test]
    fn classify_403_as_permanent() {
        let kind = classify_refresh_error(403, "forbidden");
        assert_eq!(kind, RefreshErrorKind::Permanent);
    }

    #[test]
    fn classify_429_as_rate_limited() {
        let kind = classify_refresh_error(429, "too many requests");
        assert_eq!(kind, RefreshErrorKind::RateLimited);
    }

    #[test]
    fn classify_500_as_transient() {
        let kind = classify_refresh_error(500, "internal server error");
        assert_eq!(kind, RefreshErrorKind::Transient);
    }

    #[test]
    fn classify_502_as_transient() {
        let kind = classify_refresh_error(502, "bad gateway");
        assert_eq!(kind, RefreshErrorKind::Transient);
    }

    #[test]
    fn classify_503_as_transient() {
        let kind = classify_refresh_error(503, "service unavailable");
        assert_eq!(kind, RefreshErrorKind::Transient);
    }

    #[test]
    fn classify_0_network_error_as_transient() {
        let kind = classify_refresh_error(0, "");
        assert_eq!(kind, RefreshErrorKind::Transient);
    }

    // --- Retry backoff ---

    #[test]
    fn retry_backoff_schedule() {
        assert_eq!(retry_backoff(0), Duration::from_secs(5));
        assert_eq!(retry_backoff(1), Duration::from_secs(10));
        assert_eq!(retry_backoff(2), Duration::from_secs(20));
        assert_eq!(retry_backoff(3), Duration::from_secs(30));
        assert_eq!(retry_backoff(4), Duration::from_secs(25));
    }

    #[test]
    fn retry_backoff_caps_at_last() {
        assert_eq!(retry_backoff(100), Duration::from_secs(25));
    }

    #[test]
    fn retry_backoff_total_is_90_seconds() {
        let total: u64 = RETRY_BACKOFF_SECS.iter().sum();
        assert_eq!(total, TRANSIENT_FAILURE_TIMEOUT_SECS);
    }

    // --- Constants ---

    #[test]
    fn transient_failure_timeout_is_90_seconds() {
        assert_eq!(TRANSIENT_FAILURE_TIMEOUT_SECS, 90);
    }
}
