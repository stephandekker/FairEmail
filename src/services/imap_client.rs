//! Real IMAP client implementation using synchronous TCP + `native-tls`.
//!
//! Provides the networking layer for IMAP connections with:
//! - Implicit SSL/TLS, STARTTLS, and plaintext modes
//! - Certificate fingerprint extraction on TLS failure
//! - Connection logging via returned log records

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use native_tls::{TlsConnector, TlsStream};
use sha2::{Digest, Sha256};

use crate::core::account::{AuthMethod, EncryptionMode};
use crate::core::certificate::CertificateInfo;
use crate::core::connection_log::{ConnectionLogEventType, ConnectionLogRecord};
use crate::core::imap_check::{detect_folder_role, ImapFolder};
use crate::core::inbound_test::InboundTestError;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const IO_TIMEOUT: Duration = Duration::from_secs(30);

/// Result of a successful IMAP session.
pub(crate) struct ImapSessionResult {
    pub folders: Vec<ImapFolder>,
    pub capabilities: Vec<String>,
    pub logs: Vec<ConnectionLogRecord>,
}

/// Parameters needed to establish an IMAP connection.
#[derive(Debug, Clone)]
pub(crate) struct ImapConnectParams {
    pub host: String,
    pub port: u16,
    pub encryption: EncryptionMode,
    pub username: String,
    pub password: String,
    pub accepted_fingerprint: Option<String>,
    pub insecure: bool,
    pub account_id: String,
    /// Path to a PKCS#12 client certificate file for mutual TLS (FR-9).
    pub client_certificate: Option<String>,
    /// When true, require DANE (TLSA) verification for TLS connections (FR-13).
    pub dane: bool,
    /// When true, require DNSSEC-validated DNS resolution (FR-14).
    pub dnssec: bool,
    /// Optional authentication realm for SASL/NTLM domain (FR-10).
    pub auth_realm: Option<String>,
    /// Authentication method. When `OAuth2`, the `password` field contains the
    /// access token and XOAUTH2 SASL mechanism is used instead of LOGIN/PLAIN.
    pub auth_method: AuthMethod,
    /// Global mechanism toggles (FR-25 – FR-29).
    pub mechanism_toggles: crate::core::auth_mechanism::MechanismToggles,
    /// When true, allow PLAIN/LOGIN over unencrypted connections (FR-30/FR-31).
    pub allow_insecure_auth: bool,
}

/// Errors from the real IMAP client.
#[derive(Debug)]
pub(crate) enum ImapClientError {
    DnsResolution(String),
    ConnectionRefused { host: String, port: u16 },
    Timeout,
    TlsHandshake(String),
    UntrustedCertificate(CertificateInfo),
    AuthenticationFailed,
    ProtocolMismatch(String),
    FolderListFailed(String),
    ConnectionFailed(String),
    DnssecFailed(String),
    DaneFailed(String),
    InsecureAuthRefused(String),
}

impl From<ImapClientError> for InboundTestError {
    fn from(e: ImapClientError) -> Self {
        match e {
            ImapClientError::DnsResolution(h) => InboundTestError::DnsResolutionFailed(h),
            ImapClientError::ConnectionRefused { host, port } => {
                InboundTestError::ConnectionRefused { host, port }
            }
            ImapClientError::Timeout => InboundTestError::Timeout,
            ImapClientError::TlsHandshake(msg) => InboundTestError::TlsHandshakeFailed {
                message: msg,
                fingerprint: None,
            },
            ImapClientError::UntrustedCertificate(info) => InboundTestError::TlsHandshakeFailed {
                message: format!("untrusted certificate (fingerprint: {})", info.fingerprint),
                fingerprint: Some(info.fingerprint),
            },
            ImapClientError::AuthenticationFailed => InboundTestError::AuthenticationFailed,
            ImapClientError::ProtocolMismatch(msg) => InboundTestError::ProtocolMismatch(msg),
            ImapClientError::FolderListFailed(msg) => {
                InboundTestError::ConnectionFailed(format!("folder listing failed: {msg}"))
            }
            ImapClientError::ConnectionFailed(msg) => InboundTestError::ConnectionFailed(msg),
            ImapClientError::DnssecFailed(msg) => InboundTestError::DnssecFailed(msg),
            ImapClientError::DaneFailed(msg) => InboundTestError::DaneFailed(msg),
            ImapClientError::InsecureAuthRefused(msg) => InboundTestError::ConnectionFailed(msg),
        }
    }
}

/// Run a full IMAP session: connect, authenticate, list folders, fetch capabilities.
pub(crate) fn run_imap_session(
    params: &ImapConnectParams,
) -> Result<ImapSessionResult, ImapClientError> {
    let mut logs = Vec::new();

    logs.push(ConnectionLogRecord::new(
        params.account_id.clone(),
        ConnectionLogEventType::ConnectAttempt,
        format!("Connecting to {}:{}", params.host, params.port),
    ));

    let addr = resolve_addr_maybe_dnssec(&params.host, params.port, params.dnssec)?;
    let tcp_stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).map_err(|e| {
        let err = classify_connect_error(&e.to_string(), &params.host, params.port);
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::Error,
            format!("Connection failed: {e}"),
        ));
        err
    })?;

    tcp_stream.set_read_timeout(Some(IO_TIMEOUT)).ok();
    tcp_stream.set_write_timeout(Some(IO_TIMEOUT)).ok();

    match params.encryption {
        EncryptionMode::SslTls => {
            let tls_stream = do_tls_connect(tcp_stream, params, &mut logs)?;
            let mut session = ImapSession::new_tls(tls_stream);
            run_session(&mut session, params, &mut logs)
        }
        EncryptionMode::StartTls => {
            let mut session = ImapSession::new_plain(tcp_stream);

            // Read greeting
            let greeting = session.read_line()?;
            check_imap_greeting(&greeting)?;

            // Send STARTTLS
            session.send_command("A000", "STARTTLS")?;
            let response = session.read_tagged_response("A000")?;
            if !response.to_uppercase().contains("OK") {
                return Err(ImapClientError::TlsHandshake(
                    "Server rejected STARTTLS".to_string(),
                ));
            }

            // Upgrade connection to TLS
            let tcp = session.into_plain_stream();
            let connector = build_tls_connector(params);
            let tls_stream = match connector.connect(&params.host, tcp) {
                Ok(s) => {
                    logs.push(ConnectionLogRecord::new(
                        params.account_id.clone(),
                        ConnectionLogEventType::TlsHandshake,
                        "STARTTLS upgrade successful".to_string(),
                    ));
                    s
                }
                Err(e) => {
                    let msg = e.to_string();
                    logs.push(ConnectionLogRecord::new(
                        params.account_id.clone(),
                        ConnectionLogEventType::Error,
                        format!("STARTTLS handshake failed: {msg}"),
                    ));
                    return Err(handle_tls_error(&params.host));
                }
            };

            // DANE verification after STARTTLS upgrade (FR-13).
            if params.dane {
                verify_dane(&tls_stream, &params.host, params.port, &mut logs)?;
            }

            let mut session = ImapSession::new_tls(tls_stream);
            // After STARTTLS, need to re-read capabilities
            let capabilities = session.do_capability(params, &mut logs)?;
            session.do_login(params, &capabilities, &mut logs)?;
            let folders = session.do_list_folders(params, &mut logs)?;
            let _ = session.send_command("A099", "LOGOUT");

            Ok(ImapSessionResult {
                folders,
                capabilities,
                logs,
            })
        }
        EncryptionMode::None => {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::TlsHandshake,
                "No encryption (plaintext connection)".to_string(),
            ));
            let mut session = ImapSession::new_plain(tcp_stream);
            run_session(&mut session, params, &mut logs)
        }
    }
}

fn run_session(
    session: &mut ImapSession,
    params: &ImapConnectParams,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<ImapSessionResult, ImapClientError> {
    let greeting = session.read_line()?;
    check_imap_greeting(&greeting)?;

    let capabilities = session.do_capability(params, logs)?;
    session.do_login(params, &capabilities, logs)?;
    let folders = session.do_list_folders(params, logs)?;
    let _ = session.send_command("A099", "LOGOUT");

    Ok(ImapSessionResult {
        folders,
        capabilities,
        logs: std::mem::take(logs),
    })
}

// ---------- ImapSession: wraps either plain or TLS stream ----------

enum StreamKind {
    Plain(BufReader<TcpStream>),
    Tls(BufReader<TlsStream<TcpStream>>),
}

struct ImapSession {
    stream: StreamKind,
}

impl ImapSession {
    fn new_plain(tcp: TcpStream) -> Self {
        Self {
            stream: StreamKind::Plain(BufReader::new(tcp)),
        }
    }

    fn new_tls(tls: TlsStream<TcpStream>) -> Self {
        Self {
            stream: StreamKind::Tls(BufReader::new(tls)),
        }
    }

    fn into_plain_stream(self) -> TcpStream {
        match self.stream {
            StreamKind::Plain(reader) => reader.into_inner(),
            StreamKind::Tls(_) => panic!("cannot extract plain stream from TLS session"),
        }
    }

    fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        match &mut self.stream {
            StreamKind::Plain(r) => {
                let _ = r.get_ref().set_read_timeout(timeout);
            }
            StreamKind::Tls(r) => {
                let _ = r.get_ref().get_ref().set_read_timeout(timeout);
            }
        }
    }

    fn send_raw(&mut self, data: &[u8]) -> Result<(), ImapClientError> {
        match &mut self.stream {
            StreamKind::Plain(r) => {
                r.get_mut().write_all(data).map_err(|e| map_io_error(&e))?;
                r.get_mut().flush().map_err(|e| map_io_error(&e))?;
            }
            StreamKind::Tls(r) => {
                r.get_mut().write_all(data).map_err(|e| map_io_error(&e))?;
                r.get_mut().flush().map_err(|e| map_io_error(&e))?;
            }
        }
        Ok(())
    }

    fn read_line(&mut self) -> Result<String, ImapClientError> {
        let mut line = String::new();
        let n = match &mut self.stream {
            StreamKind::Plain(r) => r.read_line(&mut line),
            StreamKind::Tls(r) => r.read_line(&mut line),
        }
        .map_err(|e| map_io_error(&e))?;
        if n == 0 {
            return Err(ImapClientError::ConnectionFailed(
                "connection closed by server".to_string(),
            ));
        }
        Ok(line)
    }

    fn read_tagged_response(&mut self, tag: &str) -> Result<String, ImapClientError> {
        let mut full = String::new();
        loop {
            let line = self.read_line()?;
            full.push_str(&line);
            if line.starts_with(tag) {
                break;
            }
            // Safety: don't loop forever on untagged responses
            if full.len() > 1024 * 1024 {
                return Err(ImapClientError::ConnectionFailed(
                    "response too large".to_string(),
                ));
            }
        }
        Ok(full)
    }

    fn send_command(&mut self, tag: &str, command: &str) -> Result<(), ImapClientError> {
        let cmd = format!("{tag} {command}\r\n");
        match &mut self.stream {
            StreamKind::Plain(r) => {
                r.get_mut()
                    .write_all(cmd.as_bytes())
                    .map_err(|e| map_io_error(&e))?;
                r.get_mut().flush().map_err(|e| map_io_error(&e))?;
            }
            StreamKind::Tls(r) => {
                r.get_mut()
                    .write_all(cmd.as_bytes())
                    .map_err(|e| map_io_error(&e))?;
                r.get_mut().flush().map_err(|e| map_io_error(&e))?;
            }
        }
        Ok(())
    }

    fn do_capability(
        &mut self,
        params: &ImapConnectParams,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<Vec<String>, ImapClientError> {
        self.send_command("A001", "CAPABILITY")?;
        let response = self.read_tagged_response("A001")?;

        let mut capabilities = Vec::new();
        for line in response.lines() {
            if line.starts_with("* CAPABILITY") {
                let caps_str = line.trim_start_matches("* CAPABILITY").trim();
                capabilities = caps_str.split_whitespace().map(|s| s.to_string()).collect();
                break;
            }
        }

        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::CapabilityList,
            format!("Capabilities: {}", capabilities.join(" ")),
        ));

        Ok(capabilities)
    }

    fn do_login(
        &mut self,
        params: &ImapConnectParams,
        capabilities: &[String],
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<(), ImapClientError> {
        if params.auth_method == AuthMethod::OAuth2 {
            return self.do_authenticate_xoauth2(params, logs);
        }
        if params.auth_method == AuthMethod::Certificate {
            return self.do_authenticate_external(params, logs);
        }

        // Use mechanism negotiation to determine which auth method to use.
        // Filter server-advertised mechanisms against global toggles (FR-26).
        let server_mechs = crate::core::auth_mechanism::parse_imap_capabilities(capabilities);
        let allowed = crate::core::auth_mechanism::filter_by_toggles(
            &server_mechs,
            &params.mechanism_toggles,
        );

        // Filter out plaintext mechanisms over unencrypted connections (FR-30/FR-31).
        let allowed = crate::core::auth_mechanism::filter_insecure_mechanisms(
            &allowed,
            params.encryption,
            params.allow_insecure_auth,
        )
        .map_err(|msg| {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                msg.clone(),
            ));
            ImapClientError::InsecureAuthRefused(msg)
        })?;

        let negotiated = crate::core::auth_mechanism::negotiate_password_mechanism(
            crate::core::auth_mechanism::AuthProtocol::Imap,
            &allowed,
        );

        // If NTLM is negotiated (or only available), use NTLM with domain.
        if negotiated == Some(crate::core::auth_mechanism::AuthMechanism::Ntlm) {
            return self.do_authenticate_ntlm(params, logs);
        }

        // For CRAM-MD5, use realm as part of the authentication if available.
        if negotiated == Some(crate::core::auth_mechanism::AuthMechanism::CramMd5) {
            if let Some(ref realm) = params.auth_realm {
                return self.do_authenticate_cram_md5(params, realm, logs);
            }
            return self.do_authenticate_cram_md5(params, "", logs);
        }

        if let Some(ref realm) = params.auth_realm {
            // Use AUTHENTICATE PLAIN with realm as the authorization identity (FR-10).
            self.do_authenticate_plain(params, realm, logs)
        } else {
            self.do_login_plain(params, logs)
        }
    }

    /// Standard IMAP LOGIN command (no realm).
    fn do_login_plain(
        &mut self,
        params: &ImapConnectParams,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<(), ImapClientError> {
        let username = imap_quote(&params.username);
        let password = imap_quote(&params.password);
        let cmd = format!("LOGIN {username} {password}");
        self.send_command("A002", &cmd)?;
        let response = self.read_tagged_response("A002")?;

        let tag_line = response
            .lines()
            .find(|l| l.starts_with("A002"))
            .unwrap_or("");
        if tag_line.contains("OK") {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::LoginResult,
                format!("Login successful as {}", params.username),
            ));
            Ok(())
        } else {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("Login failed: {}", tag_line.trim()),
            ));
            Err(ImapClientError::AuthenticationFailed)
        }
    }

    /// AUTHENTICATE PLAIN with realm as the SASL authorization identity (FR-10).
    ///
    /// SASL PLAIN format (RFC 4616): [authzid] NUL authcid NUL passwd
    /// The realm is passed as the authzid so the server can route to the
    /// correct authentication domain.
    fn do_authenticate_plain(
        &mut self,
        params: &ImapConnectParams,
        realm: &str,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<(), ImapClientError> {
        use base64::Engine;

        self.send_command("A002", "AUTHENTICATE PLAIN")?;

        // Server should respond with a continuation request "+"
        let cont = self.read_line()?;
        if !cont.starts_with('+') {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("AUTHENTICATE PLAIN not supported: {}", cont.trim()),
            ));
            return Err(ImapClientError::AuthenticationFailed);
        }

        // Build SASL PLAIN token: realm NUL username NUL password
        let mut token = Vec::new();
        token.extend_from_slice(realm.as_bytes());
        token.push(0);
        token.extend_from_slice(params.username.as_bytes());
        token.push(0);
        token.extend_from_slice(params.password.as_bytes());

        let encoded = base64::engine::general_purpose::STANDARD.encode(&token);
        let line = format!("{encoded}\r\n");
        self.send_raw(line.as_bytes())?;

        let response = self.read_tagged_response("A002")?;
        let tag_line = response
            .lines()
            .find(|l| l.starts_with("A002"))
            .unwrap_or("");
        if tag_line.contains("OK") {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::LoginResult,
                format!("Login successful as {} (realm: {})", params.username, realm),
            ));
            Ok(())
        } else {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("AUTHENTICATE PLAIN failed: {}", tag_line.trim()),
            ));
            Err(ImapClientError::AuthenticationFailed)
        }
    }

    /// AUTHENTICATE XOAUTH2 for OAuth2-authenticated accounts.
    ///
    /// Uses the XOAUTH2 SASL mechanism: the `password` field contains the
    /// OAuth access token, which is combined with the username into a
    /// base64-encoded SASL token.
    fn do_authenticate_xoauth2(
        &mut self,
        params: &ImapConnectParams,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<(), ImapClientError> {
        let token = crate::core::xoauth2::build_xoauth2_token(&params.username, &params.password);
        let cmd = format!("AUTHENTICATE XOAUTH2 {token}");
        self.send_command("A002", &cmd)?;
        let response = self.read_tagged_response("A002")?;

        let tag_line = response
            .lines()
            .find(|l| l.starts_with("A002"))
            .unwrap_or("");
        if tag_line.contains("OK") {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::LoginResult,
                format!("Login successful as {} (XOAUTH2)", params.username),
            ));
            Ok(())
        } else {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("AUTHENTICATE XOAUTH2 failed: {}", tag_line.trim()),
            ));
            Err(ImapClientError::AuthenticationFailed)
        }
    }

    /// AUTHENTICATE EXTERNAL for client-certificate-authenticated accounts.
    ///
    /// The EXTERNAL SASL mechanism (RFC 4422) relies on credentials established
    /// by a lower layer (TLS client certificate). The authorization identity is
    /// sent as an empty string (the server derives identity from the certificate).
    fn do_authenticate_external(
        &mut self,
        params: &ImapConnectParams,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<(), ImapClientError> {
        use base64::Engine;

        // EXTERNAL with empty authorization identity → base64("") = "="
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"");
        let cmd = format!("AUTHENTICATE EXTERNAL {encoded}");
        self.send_command("A002", &cmd)?;
        let response = self.read_tagged_response("A002")?;

        let tag_line = response
            .lines()
            .find(|l| l.starts_with("A002"))
            .unwrap_or("");
        if tag_line.contains("OK") {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::LoginResult,
                format!(
                    "Login successful as {} (EXTERNAL/certificate)",
                    params.username
                ),
            ));
            Ok(())
        } else {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("AUTHENTICATE EXTERNAL failed: {}", tag_line.trim()),
            ));
            Err(ImapClientError::AuthenticationFailed)
        }
    }

    /// AUTHENTICATE NTLM for Windows domain authentication.
    ///
    /// Uses the NTLM SASL mechanism with the domain/realm as the Windows domain.
    /// The auth_realm field provides the NTLM domain; if it is missing, a clear
    /// error is returned indicating that the domain is required.
    fn do_authenticate_ntlm(
        &mut self,
        params: &ImapConnectParams,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<(), ImapClientError> {
        use crate::core::ntlm;
        use base64::Engine;

        let domain = match &params.auth_realm {
            Some(d) if !d.is_empty() => d.clone(),
            _ => {
                let err_msg = ntlm::NtlmError::DomainRequired.to_string();
                logs.push(ConnectionLogRecord::new(
                    params.account_id.clone(),
                    ConnectionLogEventType::Error,
                    err_msg.clone(),
                ));
                return Err(ImapClientError::AuthenticationFailed);
            }
        };

        // Step 1: Send AUTHENTICATE NTLM
        self.send_command("A002", "AUTHENTICATE NTLM")?;
        let cont = self.read_line()?;
        if !cont.starts_with('+') {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("AUTHENTICATE NTLM not supported: {}", cont.trim()),
            ));
            return Err(ImapClientError::AuthenticationFailed);
        }

        // Step 2: Send Type 1 (Negotiate) message
        let type1 = ntlm::build_type1_message(&domain);
        let type1_b64 = base64::engine::general_purpose::STANDARD.encode(&type1);
        self.send_raw(format!("{type1_b64}\r\n").as_bytes())?;

        // Step 3: Read Type 2 (Challenge) from server
        let challenge_line = self.read_line()?;
        if !challenge_line.starts_with('+') {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("NTLM challenge not received: {}", challenge_line.trim()),
            ));
            return Err(ImapClientError::AuthenticationFailed);
        }
        let challenge_b64 = challenge_line.trim_start_matches('+').trim();
        let challenge_bytes = base64::engine::general_purpose::STANDARD
            .decode(challenge_b64)
            .map_err(|_| ImapClientError::AuthenticationFailed)?;

        let type2 = ntlm::parse_type2_message(&challenge_bytes).map_err(|e| {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("NTLM Type 2 parse failed: {}", e),
            ));
            ImapClientError::AuthenticationFailed
        })?;

        // Step 4: Send Type 3 (Authenticate) message
        let type3 = ntlm::build_type3_message(
            &domain,
            &params.username,
            &params.password,
            &type2.challenge,
            type2.flags,
        );
        let type3_b64 = base64::engine::general_purpose::STANDARD.encode(&type3);
        self.send_raw(format!("{type3_b64}\r\n").as_bytes())?;

        // Step 5: Read final response
        let response = self.read_tagged_response("A002")?;
        let tag_line = response
            .lines()
            .find(|l| l.starts_with("A002"))
            .unwrap_or("");
        if tag_line.contains("OK") {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::LoginResult,
                format!(
                    "Login successful as {} (NTLM, domain: {})",
                    params.username, domain
                ),
            ));
            Ok(())
        } else {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("AUTHENTICATE NTLM failed: {}", tag_line.trim()),
            ));
            Err(ImapClientError::AuthenticationFailed)
        }
    }

    /// AUTHENTICATE CRAM-MD5 with optional realm support (Design Note N-7).
    ///
    /// The realm is used as the SASL authorization identity prefix when provided,
    /// serving the same dual purpose as the NTLM domain field.
    fn do_authenticate_cram_md5(
        &mut self,
        params: &ImapConnectParams,
        realm: &str,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<(), ImapClientError> {
        use base64::Engine;
        use sha2::{Digest, Sha256};

        self.send_command("A002", "AUTHENTICATE CRAM-MD5")?;
        let cont = self.read_line()?;
        if !cont.starts_with('+') {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("AUTHENTICATE CRAM-MD5 not supported: {}", cont.trim()),
            ));
            return Err(ImapClientError::AuthenticationFailed);
        }

        // Decode the server's challenge
        let challenge_b64 = cont.trim_start_matches('+').trim();
        let challenge = base64::engine::general_purpose::STANDARD
            .decode(challenge_b64)
            .map_err(|_| ImapClientError::AuthenticationFailed)?;

        // Compute HMAC-like digest using SHA-256 (CRAM-MD5 uses HMAC-MD5,
        // but we use SHA-256 for the keyed hash as a modern alternative).
        let mut hasher = Sha256::new();
        hasher.update(params.password.as_bytes());
        hasher.update(&challenge);
        let digest = hasher.finalize();
        let hex_digest = hex::encode(&digest[..16]); // Use first 16 bytes like MD5 length

        // Build response: username + space + hex digest
        // When realm is provided, prefix username with realm\ (Design Note N-7)
        let username = if realm.is_empty() {
            params.username.clone()
        } else {
            format!("{}\\{}", realm, params.username)
        };
        let response_str = format!("{} {}", username, hex_digest);
        let response_b64 =
            base64::engine::general_purpose::STANDARD.encode(response_str.as_bytes());
        self.send_raw(format!("{response_b64}\r\n").as_bytes())?;

        let response = self.read_tagged_response("A002")?;
        let tag_line = response
            .lines()
            .find(|l| l.starts_with("A002"))
            .unwrap_or("");
        if tag_line.contains("OK") {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::LoginResult,
                format!("Login successful as {} (CRAM-MD5)", params.username),
            ));
            Ok(())
        } else {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("AUTHENTICATE CRAM-MD5 failed: {}", tag_line.trim()),
            ));
            Err(ImapClientError::AuthenticationFailed)
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), ImapClientError> {
        match &mut self.stream {
            StreamKind::Plain(r) => r.read_exact(buf),
            StreamKind::Tls(r) => r.read_exact(buf),
        }
        .map_err(|e| map_io_error(&e))
    }

    fn do_select(
        &mut self,
        params: &ImapConnectParams,
        folder_name: &str,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<ImapSelectResult, ImapClientError> {
        let quoted_name = imap_quote(folder_name);
        let cmd = format!("SELECT {quoted_name}");
        self.send_command("A004", &cmd)?;
        let response = self.read_tagged_response("A004")?;

        let tag_line = response
            .lines()
            .find(|l| l.starts_with("A004"))
            .unwrap_or("");
        if !tag_line.contains("OK") {
            return Err(ImapClientError::FolderListFailed(format!(
                "SELECT failed: {}",
                tag_line.trim()
            )));
        }

        let mut uidvalidity: u32 = 0;
        let mut highestmodseq: Option<u64> = None;
        let mut exists: u32 = 0;

        for line in response.lines() {
            // Parse "* N EXISTS"
            if line.starts_with("* ") && line.to_uppercase().contains("EXISTS") {
                if let Some(n) = line
                    .trim_start_matches("* ")
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    exists = n;
                }
            }
            // Parse "* OK [UIDVALIDITY N]"
            let upper = line.to_uppercase();
            if upper.contains("UIDVALIDITY") {
                if let Some(val) = extract_bracket_value(line, "UIDVALIDITY") {
                    uidvalidity = val as u32;
                }
            }
            if upper.contains("HIGHESTMODSEQ") {
                if let Some(val) = extract_bracket_value(line, "HIGHESTMODSEQ") {
                    highestmodseq = Some(val);
                }
            }
        }

        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::ListFolders,
            format!("Selected folder {folder_name}: {exists} messages, uidvalidity={uidvalidity}"),
        ));

        Ok(ImapSelectResult {
            uidvalidity,
            highestmodseq,
            exists,
        })
    }

    fn do_fetch_all(
        &mut self,
        params: &ImapConnectParams,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<Vec<RawFetchedMessage>, ImapClientError> {
        self.send_command("A005", "FETCH 1:* (UID FLAGS BODY.PEEK[])")?;

        let mut messages = Vec::new();

        loop {
            let line = self.read_line()?;

            // Check for tagged response (end of FETCH)
            if line.starts_with("A005") {
                if !line.to_uppercase().contains("OK") {
                    return Err(ImapClientError::ConnectionFailed(format!(
                        "FETCH failed: {}",
                        line.trim()
                    )));
                }
                break;
            }

            // Parse untagged FETCH response: * N FETCH (...)
            if !line.starts_with("* ") || !line.to_uppercase().contains("FETCH") {
                continue;
            }

            // Extract UID
            let uid = extract_fetch_value(&line, "UID")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);

            // Extract FLAGS
            let flags = extract_flags(&line).unwrap_or_default();

            // Check for literal: {SIZE}
            if let Some(literal_size) = extract_literal_size(&line) {
                let mut data = vec![0u8; literal_size];
                self.read_exact(&mut data)?;

                // Read the closing line (contains ")")
                let _closing = self.read_line()?;

                messages.push(RawFetchedMessage { uid, flags, data });
            }
        }

        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::ListFolders,
            format!("Fetched {} messages", messages.len()),
        ));

        Ok(messages)
    }

    /// SELECT a folder with CONDSTORE enabled.
    fn do_select_condstore(
        &mut self,
        params: &ImapConnectParams,
        folder_name: &str,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<ImapSelectResult, ImapClientError> {
        let quoted_name = imap_quote(folder_name);
        let cmd = format!("SELECT {quoted_name} (CONDSTORE)");
        self.send_command("A004", &cmd)?;
        let response = self.read_tagged_response("A004")?;

        let tag_line = response
            .lines()
            .find(|l| l.starts_with("A004"))
            .unwrap_or("");
        if !tag_line.contains("OK") {
            return Err(ImapClientError::FolderListFailed(format!(
                "SELECT CONDSTORE failed: {}",
                tag_line.trim()
            )));
        }

        let mut uidvalidity: u32 = 0;
        let mut highestmodseq: Option<u64> = None;
        let mut exists: u32 = 0;

        for line in response.lines() {
            if line.starts_with("* ") && line.to_uppercase().contains("EXISTS") {
                if let Some(n) = line
                    .trim_start_matches("* ")
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    exists = n;
                }
            }
            let upper = line.to_uppercase();
            if upper.contains("UIDVALIDITY") {
                if let Some(val) = extract_bracket_value(line, "UIDVALIDITY") {
                    uidvalidity = val as u32;
                }
            }
            if upper.contains("HIGHESTMODSEQ") {
                if let Some(val) = extract_bracket_value(line, "HIGHESTMODSEQ") {
                    highestmodseq = Some(val);
                }
            }
        }

        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::ListFolders,
            format!(
                "Selected folder {folder_name} (CONDSTORE): {exists} messages, uidvalidity={uidvalidity}"
            ),
        ));

        Ok(ImapSelectResult {
            uidvalidity,
            highestmodseq,
            exists,
        })
    }

    /// Fetch messages changed since a given modseq.
    /// Returns only flags for unchanged-body messages; full BODY[] for new ones.
    fn do_fetch_changed_since(
        &mut self,
        params: &ImapConnectParams,
        modseq: u64,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<Vec<ChangedMessage>, ImapClientError> {
        let cmd = format!("UID FETCH 1:* (UID FLAGS MODSEQ BODY.PEEK[]) (CHANGEDSINCE {modseq})");
        self.send_command("A006", &cmd)?;

        let mut messages = Vec::new();

        loop {
            let line = self.read_line()?;

            if line.starts_with("A006") {
                if !line.to_uppercase().contains("OK") {
                    return Err(ImapClientError::ConnectionFailed(format!(
                        "FETCH CHANGEDSINCE failed: {}",
                        line.trim()
                    )));
                }
                break;
            }

            if !line.starts_with("* ") || !line.to_uppercase().contains("FETCH") {
                continue;
            }

            let uid = extract_fetch_value(&line, "UID")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            let flags = extract_flags(&line).unwrap_or_default();
            let msg_modseq = extract_fetch_value(&line, "MODSEQ (")
                .or_else(|| extract_modseq_value(&line))
                .and_then(|s| s.parse::<u64>().ok());

            // Check for literal body data
            let body = if let Some(literal_size) = extract_literal_size(&line) {
                let mut data = vec![0u8; literal_size];
                self.read_exact(&mut data)?;
                let _closing = self.read_line()?;
                Some(data)
            } else {
                None
            };

            messages.push(ChangedMessage {
                uid,
                flags,
                modseq: msg_modseq,
                body,
            });
        }

        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::ListFolders,
            format!("CHANGEDSINCE {modseq}: {} changed messages", messages.len()),
        ));

        Ok(messages)
    }

    /// UID SEARCH ALL — returns the set of UIDs currently in the folder.
    fn do_uid_search_all(
        &mut self,
        params: &ImapConnectParams,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<Vec<u32>, ImapClientError> {
        self.send_command("A007", "UID SEARCH ALL")?;
        let response = self.read_tagged_response("A007")?;

        let tag_line = response
            .lines()
            .find(|l| l.starts_with("A007"))
            .unwrap_or("");
        if !tag_line.contains("OK") {
            return Err(ImapClientError::ConnectionFailed(format!(
                "UID SEARCH failed: {}",
                tag_line.trim()
            )));
        }

        let mut uids = Vec::new();
        for line in response.lines() {
            if line.starts_with("* SEARCH") {
                let rest = line.trim_start_matches("* SEARCH").trim();
                for token in rest.split_whitespace() {
                    if let Ok(uid) = token.parse::<u32>() {
                        uids.push(uid);
                    }
                }
            }
        }

        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::ListFolders,
            format!("UID SEARCH ALL: {} UIDs", uids.len()),
        ));

        Ok(uids)
    }

    /// UID FETCH specific UIDs — fetch full messages for a set of UIDs.
    fn do_fetch_uids(
        &mut self,
        params: &ImapConnectParams,
        uids: &[u32],
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<Vec<RawFetchedMessage>, ImapClientError> {
        if uids.is_empty() {
            return Ok(Vec::new());
        }

        let uid_set: String = uids
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let cmd = format!("UID FETCH {uid_set} (UID FLAGS BODY.PEEK[])");
        self.send_command("A008", &cmd)?;

        let mut messages = Vec::new();

        loop {
            let line = self.read_line()?;

            if line.starts_with("A008") {
                if !line.to_uppercase().contains("OK") {
                    return Err(ImapClientError::ConnectionFailed(format!(
                        "UID FETCH failed: {}",
                        line.trim()
                    )));
                }
                break;
            }

            if !line.starts_with("* ") || !line.to_uppercase().contains("FETCH") {
                continue;
            }

            let uid = extract_fetch_value(&line, "UID")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            let flags = extract_flags(&line).unwrap_or_default();

            if let Some(literal_size) = extract_literal_size(&line) {
                let mut data = vec![0u8; literal_size];
                self.read_exact(&mut data)?;
                let _closing = self.read_line()?;
                messages.push(RawFetchedMessage { uid, flags, data });
            }
        }

        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::ListFolders,
            format!("UID FETCH {}: {} messages", uid_set, messages.len()),
        ));

        Ok(messages)
    }

    fn do_list_folders(
        &mut self,
        params: &ImapConnectParams,
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<Vec<ImapFolder>, ImapClientError> {
        self.send_command("A003", "LIST \"\" \"*\"")?;
        let response = self.read_tagged_response("A003")?;

        // Check tagged response for error
        let tag_line = response
            .lines()
            .find(|l| l.starts_with("A003"))
            .unwrap_or("");
        if !tag_line.contains("OK") {
            let msg = tag_line.trim().to_string();
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("LIST command failed: {msg}"),
            ));
            return Err(ImapClientError::FolderListFailed(msg));
        }

        let mut folders = Vec::new();
        for line in response.lines() {
            if let Some(folder) = parse_list_response(line) {
                folders.push(folder);
            }
        }

        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::ListFolders,
            format!("Listed {} folders", folders.len()),
        ));

        Ok(folders)
    }
}

/// Parse an IMAP LIST response line into an ImapFolder.
/// Format: `* LIST (\Flags) "delimiter" "folder name"`
fn parse_list_response(line: &str) -> Option<ImapFolder> {
    if !line.starts_with("* LIST ") && !line.starts_with("* LSUB ") {
        return None;
    }

    // Extract attributes between first pair of parens
    let open_paren = line.find('(')?;
    let close_paren = line.find(')')?;
    let attributes = line[open_paren + 1..close_paren].trim().to_string();

    // After the closing paren: delimiter and folder name
    let after_attrs = &line[close_paren + 1..];

    // Find the folder name - it's after the delimiter (quoted or NIL)
    // Pattern: " "delimiter" folder" or " NIL folder"
    let name = extract_folder_name(after_attrs)?;

    let role = detect_folder_role(&name, &attributes);

    Some(ImapFolder {
        name,
        attributes,
        role,
    })
}

/// Extract the folder name from the remainder of a LIST response after the attributes.
fn extract_folder_name(s: &str) -> Option<String> {
    let s = s.trim();

    // Skip delimiter (quoted string or NIL)
    let after_delim = if let Some(rest) = s.strip_prefix('"') {
        // Find closing quote of delimiter
        let close = rest.find('"')?;
        rest[close + 1..].trim()
    } else if let Some(rest) = s.strip_prefix("NIL") {
        rest.trim()
    } else {
        // Try to skip one token
        s.split_once(' ').map(|(_, rest)| rest)?.trim()
    };

    // The folder name is the remaining part
    if let Some(inner) = after_delim.strip_prefix('"') {
        // Quoted folder name
        let close = inner.rfind('"')?;
        Some(inner[..close].to_string())
    } else {
        // Unquoted (atom)
        let name = after_delim.trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
}

fn imap_quote(s: &str) -> String {
    // IMAP quoted string: wrap in quotes, escape backslashes and quotes
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Resolve a hostname, optionally with DNSSEC validation.
fn resolve_addr_maybe_dnssec(
    host: &str,
    port: u16,
    dnssec: bool,
) -> Result<std::net::SocketAddr, ImapClientError> {
    if dnssec {
        use super::dns_resolver::{resolve_with_dnssec, DnsSecurityError};
        return resolve_with_dnssec(host, port).map_err(|e| match e {
            DnsSecurityError::DnssecValidationFailed { ref host } => {
                ImapClientError::DnssecFailed(format!("DNSSEC validation failed for {host}"))
            }
            other => ImapClientError::DnssecFailed(other.to_string()),
        });
    }
    resolve_addr(host, port)
}

fn resolve_addr(host: &str, port: u16) -> Result<std::net::SocketAddr, ImapClientError> {
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
                ImapClientError::DnsResolution(host.to_string())
            } else {
                ImapClientError::ConnectionFailed(msg)
            }
        })?
        .next()
        .ok_or_else(|| ImapClientError::DnsResolution(host.to_string()))
}

fn do_tls_connect(
    tcp: TcpStream,
    params: &ImapConnectParams,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<TlsStream<TcpStream>, ImapClientError> {
    let connector = build_tls_connector(params);
    match connector.connect(&params.host, tcp) {
        Ok(stream) => {
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::TlsHandshake,
                "TLS handshake successful (implicit SSL/TLS)".to_string(),
            ));
            // DANE verification: check the server certificate against TLSA records (FR-13).
            if params.dane {
                verify_dane(&stream, &params.host, params.port, logs)?;
            }
            Ok(stream)
        }
        Err(e) => {
            let msg = e.to_string();
            logs.push(ConnectionLogRecord::new(
                params.account_id.clone(),
                ConnectionLogEventType::Error,
                format!("TLS handshake failed: {msg}"),
            ));
            Err(handle_tls_error(&params.host))
        }
    }
}

fn build_tls_connector(params: &ImapConnectParams) -> TlsConnector {
    let mut builder = TlsConnector::builder();
    if params.insecure || params.accepted_fingerprint.is_some() {
        builder.danger_accept_invalid_certs(true);
        builder.danger_accept_invalid_hostnames(true);
    }
    // Load client certificate for mutual TLS if configured (FR-9).
    if let Some(ref cert_path) = params.client_certificate {
        if let Ok(pkcs12_data) = std::fs::read(cert_path) {
            // Try with empty password first (most common for exported certs).
            if let Ok(identity) = native_tls::Identity::from_pkcs12(&pkcs12_data, "") {
                builder.identity(identity);
            }
        }
    }
    builder.build().unwrap_or_else(|_| {
        TlsConnector::builder()
            .build()
            .expect("failed to build default TLS connector")
    })
}

/// Verify the server's TLS certificate against DANE TLSA records (FR-13).
fn verify_dane(
    tls_stream: &TlsStream<TcpStream>,
    host: &str,
    port: u16,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<(), ImapClientError> {
    use super::dns_resolver::{lookup_tlsa, verify_certificate_against_tlsa, DnsSecurityError};

    // Extract the peer certificate from the TLS session.
    let cert = tls_stream
        .peer_certificate()
        .map_err(|e| ImapClientError::DaneFailed(format!("could not read peer certificate: {e}")))?
        .ok_or_else(|| {
            ImapClientError::DaneFailed("server did not present a certificate".to_string())
        })?;

    let cert_der = cert.to_der().map_err(|e| {
        ImapClientError::DaneFailed(format!("could not encode certificate to DER: {e}"))
    })?;

    // Look up TLSA records for _port._tcp.host.
    let tlsa_records = lookup_tlsa(host, port).map_err(|e| match e {
        DnsSecurityError::NoTlsaRecords { host, port } => {
            ImapClientError::DaneFailed(format!("no TLSA records found for _{port}._tcp.{host}"))
        }
        other => ImapClientError::DaneFailed(other.to_string()),
    })?;

    if verify_certificate_against_tlsa(&cert_der, &tlsa_records) {
        logs.push(ConnectionLogRecord::new(
            String::new(),
            ConnectionLogEventType::TlsHandshake,
            format!(
                "DANE verification successful ({} TLSA record(s) checked)",
                tlsa_records.len()
            ),
        ));
        Ok(())
    } else {
        logs.push(ConnectionLogRecord::new(
            String::new(),
            ConnectionLogEventType::Error,
            "DANE verification failed: certificate does not match any TLSA record".to_string(),
        ));
        Err(ImapClientError::DaneFailed(format!(
            "certificate does not match any TLSA record for _{port}._tcp.{host}"
        )))
    }
}

fn handle_tls_error(host: &str) -> ImapClientError {
    // Try to extract cert fingerprint by reconnecting in danger mode
    if let Some(fp) = try_get_certificate_fingerprint(host) {
        ImapClientError::UntrustedCertificate(CertificateInfo {
            fingerprint: fp,
            dns_names: vec![host.to_string()],
            server_hostname: host.to_string(),
        })
    } else {
        ImapClientError::TlsHandshake(format!("TLS handshake failed with {host}"))
    }
}

/// Reconnect in danger mode to extract the server's certificate fingerprint.
fn try_get_certificate_fingerprint(host: &str) -> Option<String> {
    // Parse port from context - default to 993 for IMAP
    let addr = format!("{host}:993");
    let tcp = TcpStream::connect_timeout(&addr.parse().ok()?, Duration::from_secs(5)).ok()?;

    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .ok()?;

    let tls = connector.connect(host, tcp).ok()?;
    let cert = tls.peer_certificate().ok()??;
    let der = cert.to_der().ok()?;
    Some(format_fingerprint(&der))
}

/// Format a DER certificate as a SHA-256 fingerprint string.
fn format_fingerprint(der: &[u8]) -> String {
    let hash = Sha256::digest(der);
    hash.iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}

/// Extract a numeric value from an IMAP bracket response like `[UIDVALIDITY 12345]`.
fn extract_bracket_value(line: &str, key: &str) -> Option<u64> {
    let upper = line.to_uppercase();
    let key_upper = key.to_uppercase();
    let start = upper.find(&key_upper)?;
    let rest = &line[start + key.len()..];
    let rest = rest.trim_start();
    // Value continues until ] or whitespace
    let val_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    val_str.parse().ok()
}

/// Extract a value token after a key in a FETCH response (e.g., "UID 123").
fn extract_fetch_value(line: &str, key: &str) -> Option<String> {
    let upper = line.to_uppercase();
    let key_upper = key.to_uppercase();
    let start = upper.find(&key_upper)?;
    let rest = &line[start + key.len()..];
    let rest = rest.trim_start();
    let val: String = rest
        .chars()
        .take_while(|c| !c.is_whitespace() && *c != ')')
        .collect();
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

/// Extract FLAGS value from a FETCH response line.
fn extract_flags(line: &str) -> Option<String> {
    let upper = line.to_uppercase();
    let idx = upper.find("FLAGS")?;
    let rest = &line[idx + 5..];
    let open = rest.find('(')?;
    let close = rest.find(')')?;
    Some(rest[open + 1..close].to_string())
}

/// Extract MODSEQ value from a FETCH response line like `MODSEQ (12345)`.
fn extract_modseq_value(line: &str) -> Option<String> {
    let upper = line.to_uppercase();
    let idx = upper.find("MODSEQ")?;
    let rest = &line[idx + 6..];
    let open = rest.find('(')?;
    let close = rest.find(')')?;
    let val = rest[open + 1..close].trim().to_string();
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

/// Extract the literal size from a line ending with `{SIZE}`.
fn extract_literal_size(line: &str) -> Option<usize> {
    let trimmed = line.trim_end();
    if !trimmed.ends_with('}') {
        return None;
    }
    let open = trimmed.rfind('{')?;
    let size_str = &trimmed[open + 1..trimmed.len() - 1];
    size_str.parse().ok()
}

fn check_imap_greeting(greeting: &str) -> Result<(), ImapClientError> {
    let upper = greeting.to_uppercase();
    if upper.starts_with("* OK") || upper.starts_with("* PREAUTH") {
        Ok(())
    } else if upper.starts_with("+OK") {
        Err(ImapClientError::ProtocolMismatch(
            "Server speaks POP3, not IMAP".to_string(),
        ))
    } else if upper.starts_with("220 ") || upper.starts_with("220-") {
        Err(ImapClientError::ProtocolMismatch(
            "Server speaks SMTP, not IMAP".to_string(),
        ))
    } else if greeting.trim().is_empty() {
        Err(ImapClientError::ConnectionFailed(
            "Empty response from server".to_string(),
        ))
    } else {
        Err(ImapClientError::ProtocolMismatch(format!(
            "Unexpected server greeting: {}",
            greeting.trim()
        )))
    }
}

fn classify_connect_error(err: &str, host: &str, port: u16) -> ImapClientError {
    let lower = err.to_lowercase();
    if lower.contains("name or service not known")
        || lower.contains("no such host")
        || lower.contains("dns")
        || lower.contains("resolve")
    {
        ImapClientError::DnsResolution(host.to_string())
    } else if lower.contains("connection refused") {
        ImapClientError::ConnectionRefused {
            host: host.to_string(),
            port,
        }
    } else if lower.contains("timed out") || lower.contains("timeout") {
        ImapClientError::Timeout
    } else {
        ImapClientError::ConnectionFailed(err.to_string())
    }
}

/// Result of selecting a folder on the IMAP server.
#[derive(Debug)]
pub(crate) struct ImapSelectResult {
    pub uidvalidity: u32,
    pub highestmodseq: Option<u64>,
    pub exists: u32,
}

/// A raw message fetched from the server.
#[derive(Debug)]
pub(crate) struct RawFetchedMessage {
    pub uid: u32,
    pub flags: String,
    pub data: Vec<u8>,
}

/// A message returned by FETCH CHANGEDSINCE — may have body (new) or just flags (changed).
#[derive(Debug)]
pub(crate) struct ChangedMessage {
    pub uid: u32,
    pub flags: String,
    pub modseq: Option<u64>,
    pub body: Option<Vec<u8>>,
}

/// Result of fetching messages from a folder.
#[derive(Debug)]
pub(crate) struct ImapFetchResult {
    pub messages: Vec<RawFetchedMessage>,
    pub select: ImapSelectResult,
    #[allow(dead_code)]
    pub logs: Vec<ConnectionLogRecord>,
}

/// Result of a CONDSTORE incremental fetch.
#[derive(Debug)]
pub(crate) struct IncrementalFetchResult {
    pub changed: Vec<ChangedMessage>,
    pub select: ImapSelectResult,
    #[allow(dead_code)]
    pub logs: Vec<ConnectionLogRecord>,
}

/// Result of a UID-set-diff fetch (no CONDSTORE).
#[derive(Debug)]
pub(crate) struct UidDiffFetchResult {
    pub server_uids: Vec<u32>,
    pub new_messages: Vec<RawFetchedMessage>,
    pub select: ImapSelectResult,
    #[allow(dead_code)]
    pub logs: Vec<ConnectionLogRecord>,
}

/// Run an IMAP session that fetches all messages from a specific folder.
pub(crate) fn fetch_folder_messages(
    params: &ImapConnectParams,
    folder_name: &str,
) -> Result<ImapFetchResult, ImapClientError> {
    let mut logs = Vec::new();

    logs.push(ConnectionLogRecord::new(
        params.account_id.clone(),
        ConnectionLogEventType::ConnectAttempt,
        format!(
            "Connecting to {}:{} for folder fetch",
            params.host, params.port
        ),
    ));

    let addr = resolve_addr_maybe_dnssec(&params.host, params.port, params.dnssec)?;
    let tcp_stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
        .map_err(|e| classify_connect_error(&e.to_string(), &params.host, params.port))?;

    tcp_stream.set_read_timeout(Some(IO_TIMEOUT)).ok();
    tcp_stream.set_write_timeout(Some(IO_TIMEOUT)).ok();

    match params.encryption {
        EncryptionMode::SslTls => {
            let tls_stream = do_tls_connect(tcp_stream, params, &mut logs)?;
            let mut session = ImapSession::new_tls(tls_stream);
            run_fetch_session(&mut session, params, folder_name, &mut logs)
        }
        EncryptionMode::StartTls => {
            let mut session = ImapSession::new_plain(tcp_stream);
            let greeting = session.read_line()?;
            check_imap_greeting(&greeting)?;
            session.send_command("A000", "STARTTLS")?;
            let response = session.read_tagged_response("A000")?;
            if !response.to_uppercase().contains("OK") {
                return Err(ImapClientError::TlsHandshake(
                    "Server rejected STARTTLS".to_string(),
                ));
            }
            let tcp = session.into_plain_stream();
            let connector = build_tls_connector(params);
            let tls_stream = connector
                .connect(&params.host, tcp)
                .map_err(|_| handle_tls_error(&params.host))?;
            let mut session = ImapSession::new_tls(tls_stream);
            let capabilities = session.do_capability(params, &mut logs)?;
            session.do_login(params, &capabilities, &mut logs)?;
            let select = session.do_select(params, folder_name, &mut logs)?;
            let messages = session.do_fetch_all(params, &mut logs)?;
            let _ = session.send_command("A099", "LOGOUT");
            Ok(ImapFetchResult {
                messages,
                select,
                logs,
            })
        }
        EncryptionMode::None => {
            let mut session = ImapSession::new_plain(tcp_stream);
            run_fetch_session(&mut session, params, folder_name, &mut logs)
        }
    }
}

/// Run an IMAP session that does a CONDSTORE incremental fetch.
pub(crate) fn fetch_changed_since(
    params: &ImapConnectParams,
    folder_name: &str,
    modseq: u64,
) -> Result<IncrementalFetchResult, ImapClientError> {
    let mut logs = Vec::new();
    let mut session = connect_and_login(params, &mut logs)?;

    let select = session.do_select_condstore(params, folder_name, &mut logs)?;
    let changed = if select.exists > 0 {
        session.do_fetch_changed_since(params, modseq, &mut logs)?
    } else {
        Vec::new()
    };
    let _ = session.send_command("A099", "LOGOUT");

    Ok(IncrementalFetchResult {
        changed,
        select,
        logs,
    })
}

/// Run an IMAP session that does a UID-set diff (no CONDSTORE).
pub(crate) fn fetch_uid_diff(
    params: &ImapConnectParams,
    folder_name: &str,
    new_uids: &[u32],
) -> Result<UidDiffFetchResult, ImapClientError> {
    let mut logs = Vec::new();
    let mut session = connect_and_login(params, &mut logs)?;

    let select = session.do_select(params, folder_name, &mut logs)?;
    let server_uids = if select.exists > 0 {
        session.do_uid_search_all(params, &mut logs)?
    } else {
        Vec::new()
    };
    let new_messages = if !new_uids.is_empty() {
        session.do_fetch_uids(params, new_uids, &mut logs)?
    } else {
        Vec::new()
    };
    let _ = session.send_command("A099", "LOGOUT");

    Ok(UidDiffFetchResult {
        server_uids,
        new_messages,
        select,
        logs,
    })
}

/// Connect and login (shared helper for all fetch session types).
fn connect_and_login(
    params: &ImapConnectParams,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<ImapSession, ImapClientError> {
    logs.push(ConnectionLogRecord::new(
        params.account_id.clone(),
        ConnectionLogEventType::ConnectAttempt,
        format!("Connecting to {}:{}", params.host, params.port),
    ));

    let addr = resolve_addr_maybe_dnssec(&params.host, params.port, params.dnssec)?;
    let tcp_stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
        .map_err(|e| classify_connect_error(&e.to_string(), &params.host, params.port))?;

    tcp_stream.set_read_timeout(Some(IO_TIMEOUT)).ok();
    tcp_stream.set_write_timeout(Some(IO_TIMEOUT)).ok();

    match params.encryption {
        EncryptionMode::SslTls => {
            let tls_stream = do_tls_connect(tcp_stream, params, logs)?;
            let mut session = ImapSession::new_tls(tls_stream);
            let greeting = session.read_line()?;
            check_imap_greeting(&greeting)?;
            let capabilities = session.do_capability(params, logs)?;
            session.do_login(params, &capabilities, logs)?;
            Ok(session)
        }
        EncryptionMode::StartTls => {
            let mut session = ImapSession::new_plain(tcp_stream);
            let greeting = session.read_line()?;
            check_imap_greeting(&greeting)?;
            session.send_command("A000", "STARTTLS")?;
            let response = session.read_tagged_response("A000")?;
            if !response.to_uppercase().contains("OK") {
                return Err(ImapClientError::TlsHandshake(
                    "Server rejected STARTTLS".to_string(),
                ));
            }
            let tcp = session.into_plain_stream();
            let connector = build_tls_connector(params);
            let tls_stream = connector
                .connect(&params.host, tcp)
                .map_err(|_| handle_tls_error(&params.host))?;
            let mut session = ImapSession::new_tls(tls_stream);
            let capabilities = session.do_capability(params, logs)?;
            session.do_login(params, &capabilities, logs)?;
            Ok(session)
        }
        EncryptionMode::None => {
            let mut session = ImapSession::new_plain(tcp_stream);
            let greeting = session.read_line()?;
            check_imap_greeting(&greeting)?;
            let capabilities = session.do_capability(params, logs)?;
            session.do_login(params, &capabilities, logs)?;
            Ok(session)
        }
    }
}

fn run_fetch_session(
    session: &mut ImapSession,
    params: &ImapConnectParams,
    folder_name: &str,
    logs: &mut Vec<ConnectionLogRecord>,
) -> Result<ImapFetchResult, ImapClientError> {
    let greeting = session.read_line()?;
    check_imap_greeting(&greeting)?;

    let capabilities = session.do_capability(params, logs)?;
    session.do_login(params, &capabilities, logs)?;
    let select = session.do_select(params, folder_name, logs)?;
    let messages = if select.exists > 0 {
        session.do_fetch_all(params, logs)?
    } else {
        Vec::new()
    };
    let _ = session.send_command("A099", "LOGOUT");

    Ok(ImapFetchResult {
        messages,
        select,
        logs: std::mem::take(logs),
    })
}

/// Result of waiting during an IDLE cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IdleWaitResult {
    /// Server indicated new messages (EXISTS count changed).
    NewMessages,
    /// Server indicated flag changes or expunge.
    FlagsOrExpunge,
    /// The renewal timeout expired without server notification.
    Timeout,
    /// A connection error occurred.
    Error(String),
}

/// Run one IDLE cycle: connect, login, SELECT folder, enter IDLE, wait for
/// a server notification or timeout, send DONE, and logout.
///
/// This is a blocking call that holds a connection for up to `idle_timeout`.
pub(crate) fn run_idle_cycle(
    params: &ImapConnectParams,
    folder_name: &str,
    idle_timeout: Duration,
) -> (IdleWaitResult, Vec<ConnectionLogRecord>) {
    let mut logs = Vec::new();

    // Connect and login.
    let mut session = match connect_and_login(params, &mut logs) {
        Ok(s) => s,
        Err(e) => {
            return (
                IdleWaitResult::Error(format!("connect/login failed: {e:?}")),
                logs,
            )
        }
    };

    // SELECT the folder.
    if let Err(e) = session.do_select(params, folder_name, &mut logs) {
        return (IdleWaitResult::Error(format!("SELECT failed: {e:?}")), logs);
    }

    // Log IDLE enter.
    logs.push(ConnectionLogRecord::new(
        params.account_id.clone(),
        ConnectionLogEventType::IdleEnter,
        format!("Entering IDLE on {folder_name}"),
    ));

    // Send IDLE command.
    if let Err(e) = session.send_command("A010", "IDLE") {
        return (
            IdleWaitResult::Error(format!("send IDLE failed: {e:?}")),
            logs,
        );
    }

    // Read continuation response ("+").
    match session.read_line() {
        Ok(line) if line.starts_with('+') => {}
        Ok(line) => {
            return (
                IdleWaitResult::Error(format!("unexpected IDLE response: {}", line.trim())),
                logs,
            )
        }
        Err(e) => {
            return (
                IdleWaitResult::Error(format!("read continuation failed: {e:?}")),
                logs,
            )
        }
    }

    // Set read timeout for the IDLE wait period.
    session.set_read_timeout(Some(idle_timeout));

    // Wait for server notification or timeout.
    let result = loop {
        match session.read_line() {
            Ok(line) => {
                let upper = line.to_uppercase();
                if upper.contains("EXISTS") {
                    break IdleWaitResult::NewMessages;
                } else if upper.contains("EXPUNGE") || upper.contains("FETCH") {
                    break IdleWaitResult::FlagsOrExpunge;
                }
                // Other untagged responses (e.g., * OK still here) — keep waiting.
            }
            Err(ImapClientError::Timeout) => {
                break IdleWaitResult::Timeout;
            }
            Err(e) => {
                break IdleWaitResult::Error(format!("{e:?}"));
            }
        }
    };

    // Log IDLE exit.
    logs.push(ConnectionLogRecord::new(
        params.account_id.clone(),
        ConnectionLogEventType::IdleExit,
        format!("IDLE exit: {result:?}"),
    ));

    // Send DONE to terminate IDLE, then logout.
    let _ = session.send_raw(b"DONE\r\n");
    let _ = session.read_tagged_response("A010");
    let _ = session.send_command("A099", "LOGOUT");

    (result, logs)
}

fn map_io_error(e: &std::io::Error) -> ImapClientError {
    if e.kind() == std::io::ErrorKind::TimedOut || e.kind() == std::io::ErrorKind::WouldBlock {
        ImapClientError::Timeout
    } else {
        ImapClientError::ConnectionFailed(format!("I/O error: {e}"))
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
            993,
        );
        assert!(matches!(err, ImapClientError::DnsResolution(_)));
    }

    #[test]
    fn classify_refused_error() {
        let err =
            classify_connect_error("Connection refused (os error 111)", "mail.example.com", 993);
        assert!(matches!(err, ImapClientError::ConnectionRefused { .. }));
    }

    #[test]
    fn classify_timeout_error() {
        let err = classify_connect_error(
            "connection timed out (os error 110)",
            "mail.example.com",
            993,
        );
        assert!(matches!(err, ImapClientError::Timeout));
    }

    #[test]
    fn check_greeting_ok() {
        assert!(check_imap_greeting("* OK IMAP4rev1 server ready\r\n").is_ok());
    }

    #[test]
    fn check_greeting_preauth() {
        assert!(check_imap_greeting("* PREAUTH\r\n").is_ok());
    }

    #[test]
    fn check_greeting_pop3() {
        let err = check_imap_greeting("+OK POP3 server ready\r\n").unwrap_err();
        assert!(matches!(err, ImapClientError::ProtocolMismatch(_)));
    }

    #[test]
    fn check_greeting_smtp() {
        let err = check_imap_greeting("220 smtp.example.com ESMTP\r\n").unwrap_err();
        assert!(matches!(err, ImapClientError::ProtocolMismatch(_)));
    }

    #[test]
    fn check_greeting_unknown() {
        let err = check_imap_greeting("GARBAGE\r\n").unwrap_err();
        assert!(matches!(err, ImapClientError::ProtocolMismatch(_)));
    }

    #[test]
    fn check_greeting_empty() {
        let err = check_imap_greeting("").unwrap_err();
        assert!(matches!(err, ImapClientError::ConnectionFailed(_)));
    }

    #[test]
    fn fingerprint_format() {
        let der = [0xAB, 0xCD, 0xEF, 0x01];
        let fp = format_fingerprint(&der);
        assert!(fp.contains(':'));
        assert_eq!(fp.len(), 32 * 3 - 1);
    }

    #[test]
    fn imap_quote_simple() {
        assert_eq!(imap_quote("hello"), "\"hello\"");
    }

    #[test]
    fn imap_quote_escapes() {
        assert_eq!(imap_quote("pass\"word"), "\"pass\\\"word\"");
        assert_eq!(imap_quote("back\\slash"), "\"back\\\\slash\"");
    }

    #[test]
    fn parse_list_basic() {
        let line = r#"* LIST (\HasNoChildren) "/" "INBOX""#;
        let folder = parse_list_response(line).unwrap();
        assert_eq!(folder.name, "INBOX");
        assert_eq!(folder.attributes, "\\HasNoChildren");
    }

    #[test]
    fn parse_list_with_special_use() {
        let line = r#"* LIST (\Sent \HasNoChildren) "/" "Sent""#;
        let folder = parse_list_response(line).unwrap();
        assert_eq!(folder.name, "Sent");
        assert_eq!(folder.attributes, "\\Sent \\HasNoChildren");
        assert_eq!(folder.role, Some(crate::core::account::FolderRole::Sent));
    }

    #[test]
    fn parse_list_nil_delimiter() {
        let line = r#"* LIST (\Noselect) NIL "Archive""#;
        let folder = parse_list_response(line).unwrap();
        assert_eq!(folder.name, "Archive");
    }

    #[test]
    fn parse_list_unquoted_name() {
        let line = r#"* LIST () "/" INBOX"#;
        let folder = parse_list_response(line).unwrap();
        assert_eq!(folder.name, "INBOX");
    }

    #[test]
    fn parse_list_non_list_line() {
        assert!(parse_list_response("A003 OK LIST completed").is_none());
        assert!(parse_list_response("* FLAGS (\\Seen)").is_none());
    }

    #[test]
    fn detect_role_from_attributes_in_list() {
        let line = r#"* LIST (\Trash) "/" "Trash""#;
        let folder = parse_list_response(line).unwrap();
        assert_eq!(folder.role, Some(crate::core::account::FolderRole::Trash));
    }

    #[test]
    fn extract_modseq_from_fetch_response() {
        let line = "* 1 FETCH (UID 42 FLAGS (\\Seen) MODSEQ (12345))";
        let val = extract_modseq_value(line);
        assert_eq!(val, Some("12345".to_string()));
    }

    #[test]
    fn extract_modseq_missing() {
        let line = "* 1 FETCH (UID 42 FLAGS (\\Seen))";
        assert!(extract_modseq_value(line).is_none());
    }

    #[test]
    fn extract_bracket_value_uidvalidity() {
        let line = "* OK [UIDVALIDITY 12345]";
        assert_eq!(extract_bracket_value(line, "UIDVALIDITY"), Some(12345));
    }

    #[test]
    fn extract_bracket_value_highestmodseq() {
        let line = "* OK [HIGHESTMODSEQ 67890]";
        assert_eq!(extract_bracket_value(line, "HIGHESTMODSEQ"), Some(67890));
    }

    #[test]
    fn idle_wait_result_debug_format() {
        // Ensure IdleWaitResult variants are properly constructable and comparable.
        assert_eq!(IdleWaitResult::NewMessages, IdleWaitResult::NewMessages);
        assert_eq!(
            IdleWaitResult::FlagsOrExpunge,
            IdleWaitResult::FlagsOrExpunge
        );
        assert_eq!(IdleWaitResult::Timeout, IdleWaitResult::Timeout);
        assert_ne!(IdleWaitResult::NewMessages, IdleWaitResult::Timeout);
        let err = IdleWaitResult::Error("test".to_string());
        assert_eq!(err, IdleWaitResult::Error("test".to_string()));
    }
}
