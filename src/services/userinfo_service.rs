use std::io::{Read, Write};

use crate::core::oauth_flow::OAuthFlowError;

/// Fetch user identity information from a provider's user-info endpoint (FR-35).
///
/// Performs an HTTPS GET request with the access token as a Bearer token in the
/// Authorization header. Returns the response body as a string.
///
/// This is used for providers like Mail.ru that do not include identity claims
/// in the token response (N-9, AC-12).
pub fn fetch_userinfo(url: &str, access_token: &str) -> Result<String, OAuthFlowError> {
    let stripped = url.strip_prefix("https://").ok_or_else(|| {
        OAuthFlowError::TokenExchangeFailed("User-info URL must use HTTPS".to_string())
    })?;

    let (host_and_port, path) = match stripped.find('/') {
        Some(i) => (&stripped[..i], &stripped[i..]),
        None => (stripped, "/"),
    };

    let (host, port) = match host_and_port.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().unwrap_or(443)),
        None => (host_and_port, 443u16),
    };

    let tcp = std::net::TcpStream::connect((host, port))
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("Connect failed: {e}")))?;

    tcp.set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .ok();

    let connector = native_tls::TlsConnector::new()
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("TLS init failed: {e}")))?;

    let mut tls = connector
        .connect(host, tcp)
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("TLS handshake failed: {e}")))?;

    let request = format!(
        "GET {path} HTTP/1.1\r\n\
         Host: {host_and_port}\r\n\
         Authorization: Bearer {access_token}\r\n\
         Accept: application/json\r\n\
         Connection: close\r\n\
         \r\n",
    );

    tls.write_all(request.as_bytes())
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("Write failed: {e}")))?;

    let mut response = String::new();
    tls.read_to_string(&mut response)
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("Read failed: {e}")))?;

    let (headers, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| OAuthFlowError::TokenExchangeFailed("Invalid HTTP response".to_string()))?;

    let status_line = headers.lines().next().unwrap_or("");
    let status_code = status_line.split_whitespace().nth(1).unwrap_or("0");
    if !status_code.starts_with('2') {
        return Err(OAuthFlowError::TokenExchangeFailed(format!(
            "User-info endpoint returned HTTP {status_code}: {body}"
        )));
    }

    Ok(body.to_string())
}
