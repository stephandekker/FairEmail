//! Graph API mail sending support (FR-23, AC-8).
//!
//! When a provider has an enabled Graph API profile, the application can use
//! the Microsoft Graph `sendMail` endpoint for outbound mail instead of SMTP.
//! This module provides the request construction and response parsing logic,
//! keeping it UI-free and unit-testable.

use serde::{Deserialize, Serialize};

use crate::core::provider::OAuthConfig;

/// The Microsoft Graph v1.0 sendMail endpoint.
pub const GRAPH_SEND_MAIL_URL: &str = "https://graph.microsoft.com/v1.0/me/sendMail";

/// Parameters needed to send a mail via Graph API.
#[derive(Debug, Clone)]
pub struct GraphSendParams {
    /// The access token (Bearer) for Graph API.
    pub access_token: String,
    /// The Graph API sendMail endpoint URL.
    pub endpoint_url: String,
}

/// A recipient address for the Graph API request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRecipient {
    #[serde(rename = "emailAddress")]
    pub email_address: GraphEmailAddress,
}

/// An email address object in the Graph API schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEmailAddress {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// The request body for the Graph API `sendMail` endpoint when sending
/// a raw MIME message (base64-encoded).
#[derive(Debug, Clone, Serialize)]
pub struct GraphSendRawRequest {
    /// Base64-encoded RFC 5322 message.
    #[serde(rename = "contentBytes")]
    pub content_bytes: String,
}

/// Error type for Graph API send operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphSendError {
    /// The access token is missing or empty.
    MissingToken,
    /// The Graph API returned an error response.
    ApiError { status: u16, message: String },
    /// Network or transport error.
    Transport(String),
    /// The message data is empty.
    EmptyMessage,
}

impl std::fmt::Display for GraphSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingToken => write!(f, "Graph API: access token is missing"),
            Self::ApiError { status, message } => {
                write!(f, "Graph API error (HTTP {status}): {message}")
            }
            Self::Transport(msg) => write!(f, "Graph API transport error: {msg}"),
            Self::EmptyMessage => write!(f, "Graph API: message data is empty"),
        }
    }
}

/// Determine whether a provider's Graph profile is active and should be used
/// for mail sending (FR-23, FR-24).
///
/// Returns `true` when the provider has a Graph profile whose status is active
/// in the current build mode.
pub fn should_use_graph_send(graph: Option<&OAuthConfig>, debug_mode: bool) -> bool {
    graph.is_some_and(|g| g.status.is_active(debug_mode))
}

/// Build the HTTP request components for sending a raw MIME message via Graph API.
///
/// The Graph API supports sending raw MIME via:
/// `POST /me/sendMail` with `Content-Type: text/plain` and the base64-encoded
/// RFC 5322 message in the body.
///
/// Alternatively, for the `/me/messages/{id}/send` flow, raw MIME can be PUT
/// then sent. For simplicity and alignment with the offline-first model, we use
/// the direct raw-MIME send approach:
/// `POST /me/sendMail` with `Content-Type: application/json` and a MIME payload.
///
/// Actually, the correct approach for raw MIME is:
/// `POST /v1.0/me/sendMail` — not supported for raw MIME in v1.0.
/// Instead: `POST /v1.0/me/messages` to create draft, then `/send`.
///
/// The simplest supported path is the `/me/sendMail` endpoint with a JSON body
/// that includes the message as base64 in the `message` field structure.
/// However, for raw RFC 5322 messages, the correct endpoint is:
/// `POST /v1.0/me/sendMail` with Content-Type `text/plain` and raw base64 MIME.
///
/// Returns the authorization header value and the base64-encoded body.
pub fn build_graph_send_request(
    params: &GraphSendParams,
    rfc822_data: &[u8],
) -> Result<GraphSendRequest, GraphSendError> {
    if params.access_token.is_empty() {
        return Err(GraphSendError::MissingToken);
    }
    if rfc822_data.is_empty() {
        return Err(GraphSendError::EmptyMessage);
    }

    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(rfc822_data);

    Ok(GraphSendRequest {
        url: params.endpoint_url.clone(),
        authorization: format!("Bearer {}", params.access_token),
        content_type: "text/plain".to_string(),
        body: encoded,
    })
}

/// The fully-resolved HTTP request ready to be executed by the transport layer.
#[derive(Debug, Clone)]
pub struct GraphSendRequest {
    /// The endpoint URL.
    pub url: String,
    /// The `Authorization` header value (e.g. `Bearer <token>`).
    pub authorization: String,
    /// The `Content-Type` header value.
    pub content_type: String,
    /// The request body (base64-encoded MIME for raw send).
    pub body: String,
}

/// Parse a Graph API error response body (JSON) into a human-readable message.
pub fn parse_graph_error_response(status: u16, body: &str) -> GraphSendError {
    // Graph API errors follow the OData error format:
    // { "error": { "code": "...", "message": "..." } }
    #[derive(Deserialize)]
    struct GraphErrorWrapper {
        error: Option<GraphErrorBody>,
    }
    #[derive(Deserialize)]
    struct GraphErrorBody {
        code: Option<String>,
        message: Option<String>,
    }

    let message = serde_json::from_str::<GraphErrorWrapper>(body)
        .ok()
        .and_then(|w| w.error)
        .map(|e| {
            let code = e.code.unwrap_or_default();
            let msg = e.message.unwrap_or_default();
            if code.is_empty() {
                msg
            } else {
                format!("{code}: {msg}")
            }
        })
        .unwrap_or_else(|| body.to_string());

    GraphSendError::ApiError { status, message }
}

/// Resolve Graph send parameters from a provider's Graph profile and an access token.
///
/// The Graph profile's `token_url` field is not used for sending — the endpoint is
/// always the standard Graph `sendMail` URL. The profile mainly gates whether Graph
/// sending is enabled and what scopes are required.
pub fn resolve_graph_params(access_token: &str, tenant: Option<&str>) -> GraphSendParams {
    let _ = tenant; // Tenant is already baked into the token during OAuth flow
    GraphSendParams {
        access_token: access_token.to_string(),
        endpoint_url: GRAPH_SEND_MAIL_URL.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::OAuthProfileStatus;

    fn make_graph_config(status: OAuthProfileStatus) -> OAuthConfig {
        OAuthConfig {
            auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
            token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["https://graph.microsoft.com/Mail.Send".to_string()],
            client_id: Some("test-client".to_string()),
            client_secret: None,
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: None,
            privacy_policy_url: None,
            status,
        }
    }

    #[test]
    fn should_use_graph_send_enabled() {
        let config = make_graph_config(OAuthProfileStatus::Enabled);
        assert!(should_use_graph_send(Some(&config), false));
    }

    #[test]
    fn should_use_graph_send_disabled() {
        let config = make_graph_config(OAuthProfileStatus::Disabled);
        assert!(!should_use_graph_send(Some(&config), false));
    }

    #[test]
    fn should_use_graph_send_debug_only_in_release() {
        let config = make_graph_config(OAuthProfileStatus::DebugOnly);
        assert!(!should_use_graph_send(Some(&config), false));
    }

    #[test]
    fn should_use_graph_send_debug_only_in_debug() {
        let config = make_graph_config(OAuthProfileStatus::DebugOnly);
        assert!(should_use_graph_send(Some(&config), true));
    }

    #[test]
    fn should_use_graph_send_none() {
        assert!(!should_use_graph_send(None, false));
        assert!(!should_use_graph_send(None, true));
    }

    #[test]
    fn build_request_success() {
        let params = GraphSendParams {
            access_token: "ya29.test-token".to_string(),
            endpoint_url: GRAPH_SEND_MAIL_URL.to_string(),
        };
        let data = b"From: a@b.com\r\nTo: c@d.com\r\nSubject: Test\r\n\r\nHello";
        let request = build_graph_send_request(&params, data).unwrap();

        assert_eq!(request.url, GRAPH_SEND_MAIL_URL);
        assert_eq!(request.authorization, "Bearer ya29.test-token");
        assert_eq!(request.content_type, "text/plain");
        // Body is base64 of the message
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&request.body)
            .unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn build_request_missing_token() {
        let params = GraphSendParams {
            access_token: String::new(),
            endpoint_url: GRAPH_SEND_MAIL_URL.to_string(),
        };
        let err = build_graph_send_request(&params, b"data").unwrap_err();
        assert_eq!(err, GraphSendError::MissingToken);
    }

    #[test]
    fn build_request_empty_message() {
        let params = GraphSendParams {
            access_token: "token".to_string(),
            endpoint_url: GRAPH_SEND_MAIL_URL.to_string(),
        };
        let err = build_graph_send_request(&params, b"").unwrap_err();
        assert_eq!(err, GraphSendError::EmptyMessage);
    }

    #[test]
    fn parse_graph_error_json() {
        let body = r#"{"error":{"code":"MailboxNotFound","message":"The mailbox was not found."}}"#;
        let err = parse_graph_error_response(404, body);
        match err {
            GraphSendError::ApiError { status, message } => {
                assert_eq!(status, 404);
                assert!(message.contains("MailboxNotFound"));
                assert!(message.contains("mailbox was not found"));
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[test]
    fn parse_graph_error_plain_text() {
        let err = parse_graph_error_response(500, "Internal Server Error");
        match err {
            GraphSendError::ApiError { status, message } => {
                assert_eq!(status, 500);
                assert_eq!(message, "Internal Server Error");
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[test]
    fn resolve_graph_params_builds_correctly() {
        let params = resolve_graph_params("my-access-token", Some("contoso.com"));
        assert_eq!(params.access_token, "my-access-token");
        assert_eq!(params.endpoint_url, GRAPH_SEND_MAIL_URL);
    }

    #[test]
    fn resolve_graph_params_no_tenant() {
        let params = resolve_graph_params("token123", None);
        assert_eq!(params.access_token, "token123");
        assert_eq!(params.endpoint_url, GRAPH_SEND_MAIL_URL);
    }

    #[test]
    fn graph_send_error_display() {
        assert_eq!(
            GraphSendError::MissingToken.to_string(),
            "Graph API: access token is missing"
        );
        assert_eq!(
            GraphSendError::EmptyMessage.to_string(),
            "Graph API: message data is empty"
        );
        assert_eq!(
            GraphSendError::Transport("timeout".to_string()).to_string(),
            "Graph API transport error: timeout"
        );
        assert_eq!(
            GraphSendError::ApiError {
                status: 401,
                message: "Unauthorized".to_string()
            }
            .to_string(),
            "Graph API error (HTTP 401): Unauthorized"
        );
    }

    #[test]
    fn bundled_outlook_has_graph_mail_send_scope() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("outlook.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(
            oauth
                .scopes
                .contains(&"https://graph.microsoft.com/Mail.Send".to_string()),
            "Outlook OAuth must include Graph Mail.Send scope"
        );
    }

    #[test]
    fn bundled_outlook_graph_profile_enabled() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("outlook.com").unwrap();
        let graph = candidate.provider.graph.as_ref();
        assert!(
            graph.is_some(),
            "Outlook should have a Graph profile for REST-based mail sending"
        );
        assert!(should_use_graph_send(graph, false));
    }

    #[test]
    fn bundled_office365_graph_profile_enabled() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        // Office365 is a variant that might not have its own domain pattern,
        // check via provider list.
        let providers = db.providers();
        let office365 = providers.iter().find(|p| p.id == "office365");
        assert!(office365.is_some(), "Office365 provider should exist");
        let graph = office365.unwrap().graph.as_ref();
        assert!(
            graph.is_some(),
            "Office365 should have a Graph profile for REST-based mail sending"
        );
        assert!(should_use_graph_send(graph, false));
    }

    #[test]
    fn bundled_gmail_no_graph_profile() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("gmail.com").unwrap();
        assert!(
            candidate.provider.graph.is_none(),
            "Gmail should not have a Graph profile"
        );
    }
}
