//! Real IMAP client implementation using synchronous TCP + `native-tls`.
//!
//! Provides the networking layer for IMAP connections with:
//! - Implicit SSL/TLS, STARTTLS, and plaintext modes
//! - Certificate fingerprint extraction on TLS failure
//! - Connection logging via returned log records

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use native_tls::{TlsConnector, TlsStream};
use sha2::{Digest, Sha256};

use crate::core::account::EncryptionMode;
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
}

impl From<ImapClientError> for InboundTestError {
    fn from(e: ImapClientError) -> Self {
        match e {
            ImapClientError::DnsResolution(h) => InboundTestError::DnsResolutionFailed(h),
            ImapClientError::ConnectionRefused { host, port } => {
                InboundTestError::ConnectionRefused { host, port }
            }
            ImapClientError::Timeout => InboundTestError::Timeout,
            ImapClientError::TlsHandshake(msg) => InboundTestError::TlsHandshakeFailed(msg),
            ImapClientError::UntrustedCertificate(info) => InboundTestError::TlsHandshakeFailed(
                format!("untrusted certificate (fingerprint: {})", info.fingerprint),
            ),
            ImapClientError::AuthenticationFailed => InboundTestError::AuthenticationFailed,
            ImapClientError::ProtocolMismatch(msg) => InboundTestError::ProtocolMismatch(msg),
            ImapClientError::FolderListFailed(msg) => {
                InboundTestError::ConnectionFailed(format!("folder listing failed: {msg}"))
            }
            ImapClientError::ConnectionFailed(msg) => InboundTestError::ConnectionFailed(msg),
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

    let addr = resolve_addr(&params.host, params.port)?;
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

            let mut session = ImapSession::new_tls(tls_stream);
            // After STARTTLS, need to re-read capabilities
            let capabilities = session.do_capability(params, &mut logs)?;
            session.do_login(params, &mut logs)?;
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
    session.do_login(params, logs)?;
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
        logs: &mut Vec<ConnectionLogRecord>,
    ) -> Result<(), ImapClientError> {
        // Quote username and password for IMAP LOGIN
        let username = imap_quote(&params.username);
        let password = imap_quote(&params.password);
        let cmd = format!("LOGIN {username} {password}");
        self.send_command("A002", &cmd)?;
        let response = self.read_tagged_response("A002")?;

        // Check if the tagged response indicates success
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
    builder.build().unwrap_or_else(|_| {
        TlsConnector::builder()
            .build()
            .expect("failed to build default TLS connector")
    })
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
}
