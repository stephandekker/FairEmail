//! Real SMTP client implementation using `lettre` with tokio + rustls.
//!
//! Provides the networking layer for SMTP connections with:
//! - Implicit SSL/TLS, STARTTLS, and plaintext modes
//! - Authentication via LOGIN/PLAIN
//! - EHLO max message size extraction
//! - Certificate fingerprint extraction on TLS failure

use std::io::{BufRead, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::core::account::{AuthMethod, EncryptionMode};
use crate::core::certificate::CertificateInfo;
use crate::core::connection_log::{ConnectionLogEventType, ConnectionLogRecord};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Parameters needed to establish an SMTP connection.
#[derive(Debug, Clone)]
pub(crate) struct SmtpConnectParams {
    pub host: String,
    pub port: u16,
    pub encryption: EncryptionMode,
    pub username: String,
    pub password: String,
    pub accepted_fingerprint: Option<String>,
    pub insecure: bool,
    pub account_id: String,
    /// Custom hostname to use in the EHLO command. If `None`, defaults to "localhost".
    pub ehlo_hostname: Option<String>,
    /// Authentication method. When `OAuth2`, the `password` field contains the
    /// access token and XOAUTH2 SASL mechanism is used instead of LOGIN/PLAIN.
    pub auth_method: AuthMethod,
    /// Path to a PKCS#12 client certificate file for mutual TLS.
    pub client_certificate: Option<String>,
    /// Optional authentication realm for SASL/NTLM domain (Design Note N-7).
    pub auth_realm: Option<String>,
    /// Global mechanism toggles (FR-25 – FR-29).
    pub mechanism_toggles: crate::core::auth_mechanism::MechanismToggles,
    /// When true, allow PLAIN/LOGIN over unencrypted connections (FR-30/FR-31).
    pub allow_insecure_auth: bool,
}

/// Result of a successful SMTP session.
#[allow(dead_code)]
pub(crate) struct SmtpSessionResult {
    pub max_message_size: Option<u64>,
    pub logs: Vec<ConnectionLogRecord>,
}

/// Errors from the real SMTP client.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum SmtpClientError {
    DnsResolution(String),
    ConnectionRefused {
        host: String,
        port: u16,
    },
    Timeout,
    TlsHandshake(String),
    UntrustedCertificate(CertificateInfo),
    AuthenticationFailed,
    /// No common mechanism between client and server after negotiation.
    NoMechanismAvailable,
    /// All compatible mechanisms were disabled by user toggles.
    AllMechanismsDisabled,
    /// OAuth token expired or revoked.
    TokenExpired(String),
    /// Server-side error during authentication (5xx-like).
    ServerAuthError(String),
    ProtocolMismatch(String),
    ConnectionFailed(String),
    InsecureAuthRefused(String),
}

/// Build the EHLO command string from connection params.
fn build_ehlo_cmd(params: &SmtpConnectParams) -> String {
    let hostname = params
        .ehlo_hostname
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("localhost");
    format!("EHLO {hostname}")
}

/// Run a full SMTP session: connect, TLS handshake, authenticate, query EHLO extensions.
pub(crate) fn run_smtp_session(
    params: &SmtpConnectParams,
) -> Result<SmtpSessionResult, SmtpClientError> {
    let mut logs = Vec::new();

    logs.push(ConnectionLogRecord::new(
        params.account_id.clone(),
        ConnectionLogEventType::ConnectAttempt,
        format!("SMTP connecting to {}:{}", params.host, params.port),
    ));

    let addr = resolve_addr(&params.host, params.port)?;
    let tcp_stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).map_err(|e| {
        let err = classify_connect_error(&e.to_string(), &params.host, params.port);
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::Error,
            format!("SMTP connection failed: {e}"),
        ));
        err
    })?;

    tcp_stream.set_read_timeout(Some(CONNECT_TIMEOUT)).ok();
    tcp_stream.set_write_timeout(Some(CONNECT_TIMEOUT)).ok();

    match params.encryption {
        EncryptionMode::SslTls => {
            let tls_stream = do_tls_connect(tcp_stream, params, &mut logs)?;
            let mut session = SmtpSession::new_tls(tls_stream);
            run_session(&mut session, params, &mut logs)
        }
        EncryptionMode::StartTls => {
            let mut session = SmtpSession::new_plain(tcp_stream);

            // Read greeting
            let greeting = session.read_response()?;
            check_smtp_greeting(&greeting)?;

            // Send EHLO to get initial capabilities
            session.send_line(&build_ehlo_cmd(params))?;
            let _ehlo_response = session.read_response()?;

            // Send STARTTLS
            session.send_line("STARTTLS")?;
            let starttls_response = session.read_response()?;
            if !starttls_response.starts_with("220") {
                return Err(SmtpClientError::TlsHandshake(
                    "Server rejected STARTTLS".to_string(),
                ));
            }

            // Upgrade to TLS
            let tcp = session.into_plain_stream();
            let connector = build_tls_connector(params);
            let tls_stream = match connector.connect(&params.host, tcp) {
                Ok(s) => {
                    logs.push(ConnectionLogRecord::new(
                        params.account_id.clone(),
                        ConnectionLogEventType::TlsHandshake,
                        "SMTP STARTTLS upgrade successful".to_string(),
                    ));
                    s
                }
                Err(_) => {
                    return Err(handle_tls_error(&params.host, params.port));
                }
            };

            let mut session = SmtpSession::new_tls(tls_stream);
            run_authenticated_session(&mut session, params, &mut logs)
        }
        EncryptionMode::None => {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::TlsHandshake,
                "SMTP: No encryption (plaintext connection)".to_string(),
            ));
            let mut session = SmtpSession::new_plain(tcp_stream);
            run_session(&mut session, params, &mut logs)
        }
    }
}

fn run_session(
    session: &mut SmtpSession,
    params: &SmtpConnectParams,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<SmtpSessionResult, SmtpClientError> {
    // Read greeting
    let greeting = session.read_response()?;
    check_smtp_greeting(&greeting)?;

    logs.push(ConnectionLogRecord::new(
        params.account_id.clone(),
        ConnectionLogEventType::ConnectAttempt,
        format!("SMTP greeting: {}", greeting.lines().next().unwrap_or("")),
    ));

    run_authenticated_session(session, params, logs)
}

fn run_authenticated_session(
    session: &mut SmtpSession,
    params: &SmtpConnectParams,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<SmtpSessionResult, SmtpClientError> {
    // Send EHLO
    session.send_line(&build_ehlo_cmd(params))?;
    let ehlo_response = session.read_response()?;

    if !ehlo_response.starts_with("250") {
        return Err(SmtpClientError::ConnectionFailed(format!(
            "EHLO rejected: {}",
            ehlo_response.lines().next().unwrap_or("")
        )));
    }

    // Extract max message size from EHLO response
    let max_message_size = parse_max_size_from_ehlo(&ehlo_response);

    logs.push(ConnectionLogRecord::new(
        params.account_id.clone(),
        ConnectionLogEventType::CapabilityList,
        format!("SMTP EHLO response: max_size={:?}", max_message_size),
    ));

    // Authenticate
    smtp_authenticate(session, params, logs)?;

    // QUIT
    let _ = session.send_line("QUIT");

    Ok(SmtpSessionResult {
        max_message_size,
        logs: std::mem::take(logs),
    })
}

/// Result of a successful SMTP send.
#[allow(dead_code)]
pub(crate) struct SmtpSendResult {
    pub logs: Vec<ConnectionLogRecord>,
}

/// Send a fully-composed RFC 5322 message via SMTP.
///
/// Authenticates using the provided parameters, then submits the message
/// using the SMTP envelope (MAIL FROM / RCPT TO / DATA).
pub(crate) fn send_message(
    params: &SmtpConnectParams,
    envelope_from: &str,
    envelope_to: &[String],
    rfc822_data: &[u8],
) -> Result<SmtpSendResult, SmtpClientError> {
    let mut logs = Vec::new();

    logs.push(ConnectionLogRecord::new(
        params.account_id.clone(),
        ConnectionLogEventType::ConnectAttempt,
        format!("SMTP send connecting to {}:{}", params.host, params.port),
    ));

    let addr = resolve_addr(&params.host, params.port)?;
    let tcp_stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
        .map_err(|e| classify_connect_error(&e.to_string(), &params.host, params.port))?;
    tcp_stream.set_read_timeout(Some(CONNECT_TIMEOUT)).ok();
    tcp_stream.set_write_timeout(Some(CONNECT_TIMEOUT)).ok();

    match params.encryption {
        EncryptionMode::SslTls => {
            let tls_stream = do_tls_connect(tcp_stream, params, &mut logs)?;
            let mut session = SmtpSession::new_tls(tls_stream);
            send_after_connect(
                &mut session,
                params,
                envelope_from,
                envelope_to,
                rfc822_data,
                &mut logs,
            )
        }
        EncryptionMode::StartTls => {
            let mut session = SmtpSession::new_plain(tcp_stream);
            let greeting = session.read_response()?;
            check_smtp_greeting(&greeting)?;
            session.send_line(&build_ehlo_cmd(params))?;
            let _ehlo = session.read_response()?;
            session.send_line("STARTTLS")?;
            let starttls_resp = session.read_response()?;
            if !starttls_resp.starts_with("220") {
                return Err(SmtpClientError::TlsHandshake(
                    "Server rejected STARTTLS".to_string(),
                ));
            }
            let tcp = session.into_plain_stream();
            let connector = build_tls_connector(params);
            let tls_stream = connector
                .connect(&params.host, tcp)
                .map_err(|_| handle_tls_error(&params.host, params.port))?;
            let mut session = SmtpSession::new_tls(tls_stream);
            send_after_connect_no_greeting(
                &mut session,
                params,
                envelope_from,
                envelope_to,
                rfc822_data,
                &mut logs,
            )
        }
        EncryptionMode::None => {
            let mut session = SmtpSession::new_plain(tcp_stream);
            send_after_connect(
                &mut session,
                params,
                envelope_from,
                envelope_to,
                rfc822_data,
                &mut logs,
            )
        }
    }
}

/// Authenticate and send after greeting (for SslTls / None).
fn send_after_connect(
    session: &mut SmtpSession,
    params: &SmtpConnectParams,
    envelope_from: &str,
    envelope_to: &[String],
    rfc822_data: &[u8],
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<SmtpSendResult, SmtpClientError> {
    let greeting = session.read_response()?;
    check_smtp_greeting(&greeting)?;
    send_after_connect_no_greeting(
        session,
        params,
        envelope_from,
        envelope_to,
        rfc822_data,
        logs,
    )
}

/// Authenticate and send (greeting already consumed, e.g. after STARTTLS).
fn send_after_connect_no_greeting(
    session: &mut SmtpSession,
    params: &SmtpConnectParams,
    envelope_from: &str,
    envelope_to: &[String],
    rfc822_data: &[u8],
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<SmtpSendResult, SmtpClientError> {
    // EHLO
    session.send_line(&build_ehlo_cmd(params))?;
    let ehlo_response = session.read_response()?;
    if !ehlo_response.starts_with("250") {
        return Err(SmtpClientError::ConnectionFailed(format!(
            "EHLO rejected: {}",
            ehlo_response.lines().next().unwrap_or("")
        )));
    }

    // Authenticate
    smtp_authenticate(session, params, logs)?;

    // MAIL FROM
    let mail_from = format!("MAIL FROM:<{envelope_from}>");
    session.send_line(&mail_from)?;
    let from_resp = session.read_response()?;
    if !from_resp.starts_with("250") {
        return Err(SmtpClientError::ConnectionFailed(format!(
            "MAIL FROM rejected: {}",
            from_resp.trim()
        )));
    }

    // RCPT TO for each recipient
    for rcpt in envelope_to {
        let rcpt_cmd = format!("RCPT TO:<{rcpt}>");
        session.send_line(&rcpt_cmd)?;
        let rcpt_resp = session.read_response()?;
        if !rcpt_resp.starts_with("250") && !rcpt_resp.starts_with("251") {
            return Err(SmtpClientError::ConnectionFailed(format!(
                "RCPT TO rejected for {rcpt}: {}",
                rcpt_resp.trim()
            )));
        }
    }

    // DATA
    session.send_line("DATA")?;
    let data_resp = session.read_response()?;
    if !data_resp.starts_with("354") {
        return Err(SmtpClientError::ConnectionFailed(format!(
            "DATA rejected: {}",
            data_resp.trim()
        )));
    }

    // Send message body with dot-stuffing, then ".\r\n" to end
    smtp_send_data(session, rfc822_data)?;
    let send_resp = session.read_response()?;
    if !send_resp.starts_with("250") {
        return Err(SmtpClientError::ConnectionFailed(format!(
            "Message rejected: {}",
            send_resp.trim()
        )));
    }

    logs.push(ConnectionLogRecord::new(
        params.account_id.clone(),
        ConnectionLogEventType::LoginResult,
        "SMTP message sent successfully".to_string(),
    ));

    let _ = session.send_line("QUIT");

    Ok(SmtpSendResult {
        logs: std::mem::take(logs),
    })
}

/// Authenticate via AUTH LOGIN, AUTH PLAIN, AUTH NTLM, AUTH XOAUTH2, or AUTH EXTERNAL.
fn smtp_authenticate(
    session: &mut SmtpSession,
    params: &SmtpConnectParams,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<(), SmtpClientError> {
    if params.auth_method == AuthMethod::Certificate {
        return smtp_authenticate_external(session, params, logs);
    }
    if params.auth_method == AuthMethod::OAuth2 {
        return smtp_authenticate_xoauth2(session, params, logs);
    }

    let toggles = &params.mechanism_toggles;

    // If realm is provided and NTLM is enabled, try NTLM first for domain authentication
    if toggles.ntlm_enabled {
        if let Some(ref realm) = params.auth_realm {
            if !realm.is_empty() {
                // Attempt NTLM authentication with domain
                if let Ok(()) = smtp_authenticate_ntlm(session, params, realm, logs) {
                    return Ok(());
                }
                // Fall through to standard auth if NTLM is not supported
            }
        }
    }

    // Refuse PLAIN/LOGIN over unencrypted connections unless opted in (FR-30/FR-31).
    let insecure_plain_ok = params.encryption != EncryptionMode::None || params.allow_insecure_auth;
    if !insecure_plain_ok && !toggles.login_enabled && !toggles.plain_enabled {
        // Neither LOGIN nor PLAIN is enabled anyway — fall through to final error.
    } else if !insecure_plain_ok && (toggles.login_enabled || toggles.plain_enabled) {
        // At least one plaintext mechanism is enabled but the connection is unencrypted.
        // Check whether there are any non-plaintext mechanisms left to try.
        // If not, refuse with a clear error.
        let have_non_plaintext = toggles.ntlm_enabled || toggles.cram_md5_enabled;
        if !have_non_plaintext {
            let msg = "Refusing to authenticate: PLAIN/LOGIN not permitted over an unencrypted connection. \
                Enable \"Allow insecure authentication\" in account settings to override.".to_string();
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                msg.clone(),
            ));
            return Err(SmtpClientError::InsecureAuthRefused(msg));
        }
        // Non-plaintext mechanisms exist; skip LOGIN/PLAIN below but don't error yet.
    }

    // Try AUTH LOGIN if enabled by global toggles (FR-26) and connection is secure enough.
    if toggles.login_enabled && insecure_plain_ok {
        session.send_line("AUTH LOGIN")?;
        let auth_response = session.read_response()?;

        if auth_response.starts_with("334") {
            // AUTH LOGIN flow
            let username_b64 = base64_encode(&params.username);
            session.send_line(&username_b64)?;
            let user_response = session.read_response()?;
            if !user_response.starts_with("334") {
                return Err(SmtpClientError::AuthenticationFailed);
            }

            let password_b64 = base64_encode(&params.password);
            session.send_line(&password_b64)?;
            let pass_response = session.read_response()?;
            if pass_response.starts_with("235") {
                logs.push(ConnectionLogRecord::new(
                    params.account_id.clone(),
                    ConnectionLogEventType::LoginResult,
                    format!("SMTP login successful (LOGIN) as {}", params.username),
                ));
                return Ok(());
            }
            return Err(SmtpClientError::AuthenticationFailed);
        }
        // Server rejected AUTH LOGIN; fall through to try PLAIN
    }

    // Try AUTH PLAIN if enabled by global toggles (FR-26) and connection is secure enough.
    if toggles.plain_enabled && insecure_plain_ok {
        let plain_credentials = if let Some(ref realm) = params.auth_realm {
            if !realm.is_empty() {
                build_auth_plain_with_realm(&params.username, &params.password, realm)
            } else {
                build_auth_plain(&params.username, &params.password)
            }
        } else {
            build_auth_plain(&params.username, &params.password)
        };
        let plain_cmd = format!("AUTH PLAIN {plain_credentials}");
        session.send_line(&plain_cmd)?;
        let plain_response = session.read_response()?;
        if plain_response.starts_with("235") {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::LoginResult,
                format!("SMTP login successful (PLAIN) as {}", params.username),
            ));
            return Ok(());
        }
        return Err(SmtpClientError::AuthenticationFailed);
    }

    // No mechanism could be used: distinguish between "disabled by toggles"
    // and "no common mechanism" based on whether any mechanisms were enabled.
    let any_password_enabled = toggles.login_enabled
        || toggles.plain_enabled
        || toggles.ntlm_enabled
        || toggles.cram_md5_enabled;
    if !any_password_enabled {
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::Error,
            "All compatible authentication mechanisms have been disabled in settings".to_string(),
        ));
        Err(SmtpClientError::AllMechanismsDisabled)
    } else {
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::Error,
            "No common authentication mechanism found between client and server".to_string(),
        ));
        Err(SmtpClientError::NoMechanismAvailable)
    }
}

/// Authenticate via AUTH NTLM for Windows domain authentication.
fn smtp_authenticate_ntlm(
    session: &mut SmtpSession,
    params: &SmtpConnectParams,
    domain: &str,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<(), SmtpClientError> {
    use crate::core::ntlm;
    use base64::Engine;

    // Step 1: Send AUTH NTLM
    session.send_line("AUTH NTLM")?;
    let cont = session.read_response()?;
    if !cont.starts_with("334") {
        // Server doesn't support NTLM, return error to allow fallback
        return Err(SmtpClientError::AuthenticationFailed);
    }

    // Step 2: Send Type 1 (Negotiate) message
    let type1 = ntlm::build_type1_message(domain);
    let type1_b64 = base64::engine::general_purpose::STANDARD.encode(&type1);
    session.send_line(&type1_b64)?;

    // Step 3: Read Type 2 (Challenge) from server
    let challenge_response = session.read_response()?;
    if !challenge_response.starts_with("334") {
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::Error,
            format!("NTLM challenge not received: {}", challenge_response.trim()),
        ));
        return Err(SmtpClientError::AuthenticationFailed);
    }

    let challenge_b64 = challenge_response.strip_prefix("334 ").unwrap_or("").trim();
    let challenge_bytes = base64::engine::general_purpose::STANDARD
        .decode(challenge_b64)
        .map_err(|_| SmtpClientError::AuthenticationFailed)?;

    let type2 = ntlm::parse_type2_message(&challenge_bytes).map_err(|e| {
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::Error,
            format!("NTLM Type 2 parse failed: {}", e),
        ));
        SmtpClientError::AuthenticationFailed
    })?;

    // Step 4: Send Type 3 (Authenticate) message
    let type3 = ntlm::build_type3_message(
        domain,
        &params.username,
        &params.password,
        &type2.challenge,
        type2.flags,
    );
    let type3_b64 = base64::engine::general_purpose::STANDARD.encode(&type3);
    session.send_line(&type3_b64)?;

    // Step 5: Read final response
    let final_response = session.read_response()?;
    if final_response.starts_with("235") {
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::LoginResult,
            format!(
                "SMTP login successful (NTLM, domain: {}) as {}",
                domain, params.username
            ),
        ));
        Ok(())
    } else {
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::Error,
            format!("SMTP NTLM auth failed: {}", final_response.trim()),
        ));
        Err(SmtpClientError::AuthenticationFailed)
    }
}

/// Build AUTH PLAIN credentials with realm as authorization identity.
fn build_auth_plain_with_realm(username: &str, password: &str, realm: &str) -> String {
    use base64::Engine;
    let mut token = Vec::new();
    token.extend_from_slice(realm.as_bytes());
    token.push(0);
    token.extend_from_slice(username.as_bytes());
    token.push(0);
    token.extend_from_slice(password.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(&token)
}

/// Authenticate via AUTH XOAUTH2 for OAuth2 accounts.
///
/// Uses the XOAUTH2 SASL mechanism: the `password` field contains the
/// OAuth access token, which is combined with the username into a
/// base64-encoded SASL token.
fn smtp_authenticate_xoauth2(
    session: &mut SmtpSession,
    params: &SmtpConnectParams,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<(), SmtpClientError> {
    let token = crate::core::xoauth2::build_xoauth2_token(&params.username, &params.password);
    let cmd = format!("AUTH XOAUTH2 {token}");
    session.send_line(&cmd)?;
    let response = session.read_response()?;

    if response.starts_with("235") {
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::LoginResult,
            format!("SMTP login successful (XOAUTH2) as {}", params.username),
        ));
        Ok(())
    } else {
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::Error,
            format!("AUTH XOAUTH2 failed: {}", response.trim()),
        ));
        Err(SmtpClientError::AuthenticationFailed)
    }
}

/// Authenticate via AUTH EXTERNAL for client-certificate accounts.
///
/// The EXTERNAL SASL mechanism relies on credentials established by the TLS
/// layer (client certificate). An empty authorization identity is sent.
fn smtp_authenticate_external(
    session: &mut SmtpSession,
    params: &SmtpConnectParams,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<(), SmtpClientError> {
    use base64::Engine;

    // EXTERNAL with empty authorization identity → base64("") = "="
    let encoded = base64::engine::general_purpose::STANDARD.encode(b"");
    let cmd = format!("AUTH EXTERNAL {encoded}");
    session.send_line(&cmd)?;
    let response = session.read_response()?;

    if response.starts_with("235") {
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::LoginResult,
            format!(
                "SMTP login successful (EXTERNAL/certificate) as {}",
                params.username
            ),
        ));
        Ok(())
    } else {
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::Error,
            format!("AUTH EXTERNAL failed: {}", response.trim()),
        ));
        Err(SmtpClientError::AuthenticationFailed)
    }
}

/// Send message data with SMTP dot-stuffing, ending with CRLF.CRLF.
fn smtp_send_data(session: &mut SmtpSession, data: &[u8]) -> Result<(), SmtpClientError> {
    // Convert data to lines and apply dot-stuffing
    let text = String::from_utf8_lossy(data);
    for line in text.split('\n') {
        let line = line.trim_end_matches('\r');
        if line.starts_with('.') {
            // Dot-stuffing: prepend extra dot
            let stuffed = format!(".{line}\r\n");
            match &mut session.stream {
                StreamKind::Plain(r) => {
                    r.get_mut()
                        .write_all(stuffed.as_bytes())
                        .map_err(|e| map_io_error(&e))?;
                }
                StreamKind::Tls(r) => {
                    r.get_mut()
                        .write_all(stuffed.as_bytes())
                        .map_err(|e| map_io_error(&e))?;
                }
            }
        } else {
            let out = format!("{line}\r\n");
            match &mut session.stream {
                StreamKind::Plain(r) => {
                    r.get_mut()
                        .write_all(out.as_bytes())
                        .map_err(|e| map_io_error(&e))?;
                }
                StreamKind::Tls(r) => {
                    r.get_mut()
                        .write_all(out.as_bytes())
                        .map_err(|e| map_io_error(&e))?;
                }
            }
        }
    }
    // End with ".\r\n"
    session.send_line(".")?;
    Ok(())
}

/// Parse SIZE extension from EHLO response lines.
/// Looks for "250-SIZE <number>" or "250 SIZE <number>".
fn parse_max_size_from_ehlo(ehlo_response: &str) -> Option<u64> {
    for line in ehlo_response.lines() {
        let upper = line.to_uppercase();
        if upper.contains("SIZE") {
            // Format: "250-SIZE 26214400" or "250 SIZE 26214400"
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                let upper_part = part.to_uppercase();
                if upper_part == "SIZE" || upper_part.ends_with("-SIZE") {
                    if let Some(size_str) = parts.get(i + 1) {
                        if let Ok(size) = size_str.parse::<u64>() {
                            if size > 0 {
                                return Some(size);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn build_auth_plain(username: &str, password: &str) -> String {
    // AUTH PLAIN format: \0username\0password, base64-encoded
    let mut plain = Vec::new();
    plain.push(0u8);
    plain.extend_from_slice(username.as_bytes());
    plain.push(0u8);
    plain.extend_from_slice(password.as_bytes());
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(&plain)
}

fn base64_encode(s: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

// ---------- SmtpSession: wraps either plain or TLS stream ----------

enum StreamKind {
    Plain(std::io::BufReader<TcpStream>),
    Tls(std::io::BufReader<native_tls::TlsStream<TcpStream>>),
}

struct SmtpSession {
    stream: StreamKind,
}

impl SmtpSession {
    fn new_plain(tcp: TcpStream) -> Self {
        Self {
            stream: StreamKind::Plain(std::io::BufReader::new(tcp)),
        }
    }

    fn new_tls(tls: native_tls::TlsStream<TcpStream>) -> Self {
        Self {
            stream: StreamKind::Tls(std::io::BufReader::new(tls)),
        }
    }

    fn into_plain_stream(self) -> TcpStream {
        match self.stream {
            StreamKind::Plain(reader) => reader.into_inner(),
            StreamKind::Tls(_) => panic!("cannot extract plain stream from TLS session"),
        }
    }

    /// Read a full SMTP response (multi-line responses end with "NNN " prefix).
    fn read_response(&mut self) -> Result<String, SmtpClientError> {
        let mut full_response = String::new();
        loop {
            let mut line = String::new();
            let n = match &mut self.stream {
                StreamKind::Plain(r) => r.read_line(&mut line),
                StreamKind::Tls(r) => r.read_line(&mut line),
            }
            .map_err(|e| map_io_error(&e))?;

            if n == 0 {
                return Err(SmtpClientError::ConnectionFailed(
                    "connection closed by server".to_string(),
                ));
            }

            full_response.push_str(&line);

            // SMTP multi-line: "250-..." continues, "250 ..." is final
            if line.len() >= 4 && line.chars().nth(3) == Some(' ') {
                break;
            }
            // Also break on lines that don't have a continuation marker
            if line.len() < 4 || (!line[3..4].contains('-') && !line[3..4].contains(' ')) {
                break;
            }

            // Safety: prevent unbounded reads
            if full_response.len() > 64 * 1024 {
                return Err(SmtpClientError::ConnectionFailed(
                    "response too large".to_string(),
                ));
            }
        }
        Ok(full_response)
    }

    fn send_line(&mut self, line: &str) -> Result<(), SmtpClientError> {
        let data = format!("{line}\r\n");
        match &mut self.stream {
            StreamKind::Plain(r) => {
                r.get_mut()
                    .write_all(data.as_bytes())
                    .map_err(|e| map_io_error(&e))?;
                r.get_mut().flush().map_err(|e| map_io_error(&e))?;
            }
            StreamKind::Tls(r) => {
                r.get_mut()
                    .write_all(data.as_bytes())
                    .map_err(|e| map_io_error(&e))?;
                r.get_mut().flush().map_err(|e| map_io_error(&e))?;
            }
        }
        Ok(())
    }
}

fn check_smtp_greeting(greeting: &str) -> Result<(), SmtpClientError> {
    if greeting.starts_with("220") {
        Ok(())
    } else if greeting.starts_with("* OK") || greeting.to_uppercase().starts_with("* PREAUTH") {
        Err(SmtpClientError::ProtocolMismatch(
            "Server speaks IMAP, not SMTP".to_string(),
        ))
    } else if greeting.starts_with("+OK") {
        Err(SmtpClientError::ProtocolMismatch(
            "Server speaks POP3, not SMTP".to_string(),
        ))
    } else if greeting.trim().is_empty() {
        Err(SmtpClientError::ConnectionFailed(
            "Empty response from server".to_string(),
        ))
    } else {
        Err(SmtpClientError::ProtocolMismatch(format!(
            "Unexpected server greeting: {}",
            greeting.trim()
        )))
    }
}

fn resolve_addr(host: &str, port: u16) -> Result<std::net::SocketAddr, SmtpClientError> {
    use std::net::ToSocketAddrs;
    let addr_str = format!("{host}:{port}");
    addr_str
        .to_socket_addrs()
        .map_err(|e| {
            let msg = e.to_string();
            if msg.to_lowercase().contains("name or service not known")
                || msg.to_lowercase().contains("no such host")
                || msg.to_lowercase().contains("resolve")
            {
                SmtpClientError::DnsResolution(host.to_string())
            } else {
                SmtpClientError::ConnectionFailed(msg)
            }
        })?
        .next()
        .ok_or_else(|| SmtpClientError::DnsResolution(host.to_string()))
}

fn do_tls_connect(
    tcp: TcpStream,
    params: &SmtpConnectParams,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<native_tls::TlsStream<std::net::TcpStream>, SmtpClientError> {
    let connector = build_tls_connector(params);
    match connector.connect(&params.host, tcp) {
        Ok(stream) => {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::TlsHandshake,
                "SMTP TLS handshake successful (implicit SSL/TLS)".to_string(),
            ));
            Ok(stream)
        }
        Err(_) => Err(handle_tls_error(&params.host, params.port)),
    }
}

fn build_tls_connector(params: &SmtpConnectParams) -> native_tls::TlsConnector {
    let mut builder = native_tls::TlsConnector::builder();
    if params.insecure || params.accepted_fingerprint.is_some() {
        builder.danger_accept_invalid_certs(true);
        builder.danger_accept_invalid_hostnames(true);
    }
    // Load client certificate for mutual TLS if configured.
    if let Some(ref cert_path) = params.client_certificate {
        if let Ok(pkcs12_data) = std::fs::read(cert_path) {
            // Try with empty password first (most common for exported certs).
            if let Ok(identity) = native_tls::Identity::from_pkcs12(&pkcs12_data, "") {
                builder.identity(identity);
            }
        }
    }
    builder.build().unwrap_or_else(|_| {
        native_tls::TlsConnector::builder()
            .build()
            .expect("failed to build default TLS connector")
    })
}

fn handle_tls_error(host: &str, port: u16) -> SmtpClientError {
    if let Some(fp) = try_get_certificate_fingerprint(host, port) {
        SmtpClientError::UntrustedCertificate(CertificateInfo {
            fingerprint: fp,
            dns_names: vec![host.to_string()],
            server_hostname: host.to_string(),
        })
    } else {
        SmtpClientError::TlsHandshake(format!("TLS handshake failed with {host}"))
    }
}

/// Reconnect in danger mode to extract the server's certificate fingerprint.
fn try_get_certificate_fingerprint(host: &str, port: u16) -> Option<String> {
    use sha2::{Digest, Sha256};
    use std::net::TcpStream;

    let addr_str = format!("{host}:{port}");
    let addr: std::net::SocketAddr = addr_str.parse().ok()?;
    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(5)).ok()?;

    let connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .ok()?;

    let tls = connector.connect(host, tcp).ok()?;
    let cert = tls.peer_certificate().ok()??;
    let der = cert.to_der().ok()?;
    let hash = Sha256::digest(&der);
    Some(
        hash.iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(":"),
    )
}

fn classify_connect_error(err: &str, host: &str, port: u16) -> SmtpClientError {
    let lower = err.to_lowercase();
    if lower.contains("name or service not known")
        || lower.contains("no such host")
        || lower.contains("dns")
        || lower.contains("resolve")
    {
        SmtpClientError::DnsResolution(host.to_string())
    } else if lower.contains("connection refused") {
        SmtpClientError::ConnectionRefused {
            host: host.to_string(),
            port,
        }
    } else if lower.contains("timed out") || lower.contains("timeout") {
        SmtpClientError::Timeout
    } else {
        SmtpClientError::ConnectionFailed(err.to_string())
    }
}

fn map_io_error(e: &std::io::Error) -> SmtpClientError {
    if e.kind() == std::io::ErrorKind::TimedOut || e.kind() == std::io::ErrorKind::WouldBlock {
        SmtpClientError::Timeout
    } else {
        SmtpClientError::ConnectionFailed(format!("I/O error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_dns_error() {
        let err = classify_connect_error(
            "failed to lookup address: Name or service not known",
            "bad.example.com",
            587,
        );
        assert!(matches!(err, SmtpClientError::DnsResolution(_)));
    }

    #[test]
    fn classify_refused_error() {
        let err =
            classify_connect_error("Connection refused (os error 111)", "mail.example.com", 587);
        assert!(matches!(err, SmtpClientError::ConnectionRefused { .. }));
    }

    #[test]
    fn classify_timeout_error() {
        let err = classify_connect_error(
            "connection timed out (os error 110)",
            "mail.example.com",
            587,
        );
        assert!(matches!(err, SmtpClientError::Timeout));
    }

    #[test]
    fn check_greeting_ok() {
        assert!(check_smtp_greeting("220 smtp.example.com ESMTP\r\n").is_ok());
    }

    #[test]
    fn check_greeting_imap() {
        let err = check_smtp_greeting("* OK IMAP4rev1 server ready\r\n").unwrap_err();
        assert!(matches!(err, SmtpClientError::ProtocolMismatch(_)));
    }

    #[test]
    fn check_greeting_pop3() {
        let err = check_smtp_greeting("+OK POP3 server ready\r\n").unwrap_err();
        assert!(matches!(err, SmtpClientError::ProtocolMismatch(_)));
    }

    #[test]
    fn check_greeting_empty() {
        let err = check_smtp_greeting("").unwrap_err();
        assert!(matches!(err, SmtpClientError::ConnectionFailed(_)));
    }

    #[test]
    fn parse_ehlo_size() {
        let ehlo = "250-smtp.example.com\r\n250-SIZE 26214400\r\n250 OK\r\n";
        assert_eq!(parse_max_size_from_ehlo(ehlo), Some(26214400));
    }

    #[test]
    fn parse_ehlo_no_size() {
        let ehlo = "250-smtp.example.com\r\n250-PIPELINING\r\n250 OK\r\n";
        assert_eq!(parse_max_size_from_ehlo(ehlo), None);
    }

    #[test]
    fn parse_ehlo_size_zero() {
        let ehlo = "250-SIZE 0\r\n250 OK\r\n";
        assert_eq!(parse_max_size_from_ehlo(ehlo), None);
    }

    #[test]
    fn auth_plain_encoding() {
        let encoded = build_auth_plain("user@example.com", "password123");
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        assert_eq!(decoded[0], 0);
        assert!(decoded[1..].starts_with(b"user@example.com"));
        let user_end = 1 + "user@example.com".len();
        assert_eq!(decoded[user_end], 0);
        assert_eq!(&decoded[user_end + 1..], b"password123");
    }
}
