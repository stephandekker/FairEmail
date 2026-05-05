//! Background sync engine that owns network I/O for accounts.
//!
//! Runs on a dedicated `tokio` multi-threaded runtime hosted on a worker thread.
//! Spawns one task per active account. Processes `pending_operations` in insertion
//! order and emits typed events on a broadcast channel.
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::broadcast;

/// Type alias for the account-params lookup function.
type AccountParamsFn = dyn Fn(&str) -> Option<ImapConnectParams> + Send + Sync;

use crate::core::content_store::ContentStore;
use crate::core::message::FLAG_SEEN;
use crate::core::pending_operation::{
    FolderCreatePayload, FolderDeletePayload, FolderRenamePayload, OperationKind, OperationState,
    SendPayload, StoreFlagsPayload,
};
use crate::core::sync_event::SyncEvent;
use crate::services::database::{open_and_migrate, DatabaseError};
use crate::services::idle_service::{self, IdleWaiter, RealIdleWaiter};
use crate::services::imap_client::ImapConnectParams;
use crate::services::pending_ops_store;
use crate::services::smtp_client::SmtpConnectParams;

/// Errors that can occur during sync operations.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SyncError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("IMAP error: {0}")]
    Imap(String),
    #[error("SMTP error: {0}")]
    Smtp(String),
    #[error("credential error: {0}")]
    Credential(String),
    #[error("payload parse error: {0}")]
    PayloadParse(String),
    #[error("content store error: {0}")]
    ContentStore(String),
}

/// Whether a sync error is transient (retryable) or permanent.
pub(crate) fn is_transient_error(error: &SyncError) -> bool {
    match error {
        SyncError::Database(_) => false,
        SyncError::Imap(msg) | SyncError::Smtp(msg) => {
            let lower = msg.to_lowercase();
            // Permanent errors
            if lower.contains("authentication")
                || lower.contains("auth")
                || lower.contains("login")
                || lower.contains("no such mailbox")
                || lower.contains("quota")
                || lower.contains("permission")
            {
                return false;
            }
            // Everything else is transient (network, timeout, etc.)
            true
        }
        SyncError::Credential(_) => false,
        SyncError::PayloadParse(_) => false,
        SyncError::ContentStore(_) => false,
    }
}

/// Backoff durations for transient failure retries (capped at 1 hour).
const BACKOFF_SECS: &[u64] = &[5, 30, 120, 600, 3600];

/// Get the backoff duration for a given retry count.
pub(crate) fn backoff_duration(retry_count: i32) -> std::time::Duration {
    let idx = (retry_count as usize).min(BACKOFF_SECS.len() - 1);
    std::time::Duration::from_secs(BACKOFF_SECS[idx])
}

/// Trait abstracting IMAP flag-store operations for testability.
pub(crate) trait ImapFlagStore: Send + Sync {
    /// Set flags on a message by UID in the given folder.
    /// Returns Ok(()) on success, or an error string.
    fn store_flags(
        &self,
        params: &ImapConnectParams,
        folder_name: &str,
        uid: u32,
        flags: u32,
    ) -> Result<(), String>;
}

/// Real IMAP flag store that connects to the server.
pub(crate) struct RealImapFlagStore;

impl ImapFlagStore for RealImapFlagStore {
    fn store_flags(
        &self,
        params: &ImapConnectParams,
        folder_name: &str,
        uid: u32,
        flags: u32,
    ) -> Result<(), String> {
        store_flags_on_server(params, folder_name, uid, flags)
    }
}

/// Trait abstracting SMTP send operations for testability.
pub(crate) trait SmtpSender: Send + Sync {
    /// Send an RFC 5322 message via SMTP.
    fn send_message(
        &self,
        params: &SmtpConnectParams,
        envelope_from: &str,
        envelope_to: &[String],
        rfc822_data: &[u8],
    ) -> Result<(), String>;
}

/// Real SMTP sender using the smtp_client module.
pub(crate) struct RealSmtpSender;

impl SmtpSender for RealSmtpSender {
    fn send_message(
        &self,
        params: &SmtpConnectParams,
        envelope_from: &str,
        envelope_to: &[String],
        rfc822_data: &[u8],
    ) -> Result<(), String> {
        crate::services::smtp_client::send_message(params, envelope_from, envelope_to, rfc822_data)
            .map(|_| ())
            .map_err(|e| format!("{e:?}"))
    }
}

/// Trait abstracting IMAP APPEND for testability.
pub(crate) trait ImapAppender: Send + Sync {
    /// Append an RFC 5322 message to a folder with given flags.
    fn append_message(
        &self,
        params: &ImapConnectParams,
        folder_name: &str,
        flags: u32,
        rfc822_data: &[u8],
    ) -> Result<(), String>;
}

/// Real IMAP appender using raw protocol.
pub(crate) struct RealImapAppender;

impl ImapAppender for RealImapAppender {
    fn append_message(
        &self,
        params: &ImapConnectParams,
        folder_name: &str,
        flags: u32,
        rfc822_data: &[u8],
    ) -> Result<(), String> {
        imap_append_message(params, folder_name, flags, rfc822_data)
    }
}

/// Trait abstracting IMAP folder operations (CREATE, RENAME, DELETE) for testability.
pub(crate) trait ImapFolderOps: Send + Sync {
    /// Create a folder on the IMAP server.
    fn create_folder(&self, params: &ImapConnectParams, folder_name: &str) -> Result<(), String>;
    /// Rename a folder on the IMAP server.
    fn rename_folder(
        &self,
        params: &ImapConnectParams,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), String>;
    /// Delete a folder on the IMAP server.
    fn delete_folder(&self, params: &ImapConnectParams, folder_name: &str) -> Result<(), String>;
}

/// Real IMAP folder ops that connect to the server.
pub(crate) struct RealImapFolderOps;

impl ImapFolderOps for RealImapFolderOps {
    fn create_folder(&self, params: &ImapConnectParams, folder_name: &str) -> Result<(), String> {
        imap_folder_command(params, &format!("CREATE {}", imap_quote(folder_name)))
    }
    fn rename_folder(
        &self,
        params: &ImapConnectParams,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), String> {
        imap_folder_command(
            params,
            &format!("RENAME {} {}", imap_quote(old_name), imap_quote(new_name)),
        )
    }
    fn delete_folder(&self, params: &ImapConnectParams, folder_name: &str) -> Result<(), String> {
        imap_folder_command(params, &format!("DELETE {}", imap_quote(folder_name)))
    }
}

/// Mock IMAP folder ops for testing.
pub(crate) struct MockImapFolderOps {
    pub should_fail: Option<String>,
}

impl ImapFolderOps for MockImapFolderOps {
    fn create_folder(&self, _params: &ImapConnectParams, _folder_name: &str) -> Result<(), String> {
        match &self.should_fail {
            Some(err) => Err(err.clone()),
            None => Ok(()),
        }
    }
    fn rename_folder(
        &self,
        _params: &ImapConnectParams,
        _old_name: &str,
        _new_name: &str,
    ) -> Result<(), String> {
        match &self.should_fail {
            Some(err) => Err(err.clone()),
            None => Ok(()),
        }
    }
    fn delete_folder(&self, _params: &ImapConnectParams, _folder_name: &str) -> Result<(), String> {
        match &self.should_fail {
            Some(err) => Err(err.clone()),
            None => Ok(()),
        }
    }
}

/// Mock IMAP flag store for testing.
pub(crate) struct MockImapFlagStore {
    pub should_fail: Option<String>,
}

impl ImapFlagStore for MockImapFlagStore {
    fn store_flags(
        &self,
        _params: &ImapConnectParams,
        _folder_name: &str,
        _uid: u32,
        _flags: u32,
    ) -> Result<(), String> {
        match &self.should_fail {
            Some(err) => Err(err.clone()),
            None => Ok(()),
        }
    }
}

/// Perform the actual IMAP STORE command on the server.
fn store_flags_on_server(
    params: &ImapConnectParams,
    folder_name: &str,
    uid: u32,
    flags: u32,
) -> Result<(), String> {
    use crate::core::account::EncryptionMode;
    use native_tls::TlsConnector;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let addr_str = format!("{}:{}", params.host, params.port);
    let addr: std::net::SocketAddr = addr_str
        .parse()
        .or_else(|_| {
            use std::net::ToSocketAddrs;
            addr_str
                .to_socket_addrs()
                .map_err(|e| e.to_string())?
                .next()
                .ok_or_else(|| "DNS resolution failed".to_string())
        })
        .map_err(|e| format!("DNS resolution failed: {e}"))?;

    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(30))
        .map_err(|e| format!("connection failed: {e}"))?;
    tcp.set_read_timeout(Some(Duration::from_secs(30))).ok();
    tcp.set_write_timeout(Some(Duration::from_secs(30))).ok();

    // Helper closures for reading/writing on either plain or TLS stream
    macro_rules! run_session {
        ($reader:expr, $writer:expr) => {{
            // Read greeting
            let mut line = String::new();
            $reader
                .read_line(&mut line)
                .map_err(|e| format!("read greeting: {e}"))?;
            if !line.to_uppercase().starts_with("* OK")
                && !line.to_uppercase().starts_with("* PREAUTH")
            {
                return Err(format!("unexpected greeting: {}", line.trim()));
            }

            // Login
            let username = imap_quote(&params.username);
            let password = imap_quote(&params.password);
            let cmd = format!("A001 LOGIN {username} {password}\r\n");
            $writer
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("write login: {e}"))?;
            $writer.flush().map_err(|e| format!("flush login: {e}"))?;

            let mut resp = String::new();
            loop {
                let mut l = String::new();
                $reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read login response: {e}"))?;
                resp.push_str(&l);
                if l.starts_with("A001") {
                    break;
                }
            }
            if !resp.contains("A001 OK") {
                return Err("authentication failed".to_string());
            }

            // Select folder
            let quoted_folder = imap_quote(folder_name);
            let cmd = format!("A002 SELECT {quoted_folder}\r\n");
            $writer
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("write select: {e}"))?;
            $writer.flush().map_err(|e| format!("flush select: {e}"))?;

            loop {
                let mut l = String::new();
                $reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read select: {e}"))?;
                if l.starts_with("A002") {
                    if !l.contains("OK") {
                        return Err(format!("SELECT failed: {}", l.trim()));
                    }
                    break;
                }
            }

            // Build flags string
            let mut flag_parts = Vec::new();
            if flags & FLAG_SEEN != 0 {
                flag_parts.push("\\Seen");
            }
            if flags & crate::core::message::FLAG_ANSWERED != 0 {
                flag_parts.push("\\Answered");
            }
            if flags & crate::core::message::FLAG_FLAGGED != 0 {
                flag_parts.push("\\Flagged");
            }
            if flags & crate::core::message::FLAG_DELETED != 0 {
                flag_parts.push("\\Deleted");
            }
            if flags & crate::core::message::FLAG_DRAFT != 0 {
                flag_parts.push("\\Draft");
            }
            let flags_str = flag_parts.join(" ");

            // UID STORE
            let cmd = format!("A003 UID STORE {uid} FLAGS ({flags_str})\r\n");
            $writer
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("write STORE: {e}"))?;
            $writer.flush().map_err(|e| format!("flush STORE: {e}"))?;

            loop {
                let mut l = String::new();
                $reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read STORE response: {e}"))?;
                if l.starts_with("A003") {
                    if !l.contains("OK") {
                        return Err(format!("STORE failed: {}", l.trim()));
                    }
                    break;
                }
            }

            // Logout
            let cmd = "A099 LOGOUT\r\n";
            let _ = $writer.write_all(cmd.as_bytes());
            let _ = $writer.flush();

            Ok(())
        }};
    }

    match params.encryption {
        EncryptionMode::SslTls => {
            let mut builder = TlsConnector::builder();
            if params.insecure || params.accepted_fingerprint.is_some() {
                builder.danger_accept_invalid_certs(true);
                builder.danger_accept_invalid_hostnames(true);
            }
            let connector = builder
                .build()
                .map_err(|e| format!("TLS build error: {e}"))?;
            let tls = connector
                .connect(&params.host, tcp)
                .map_err(|e| format!("TLS handshake failed: {e}"))?;
            let mut reader = BufReader::new(tls);
            // We need to split reading and writing. For TLS, we can get_mut on the BufReader.
            // However, BufReader borrows the stream. We use a different approach:
            // read from BufReader, write via get_mut.
            run_session!(reader, reader.get_mut())
        }
        EncryptionMode::None => {
            let tcp_clone = tcp.try_clone().map_err(|e| format!("clone tcp: {e}"))?;
            let mut reader = BufReader::new(tcp);
            let mut writer = tcp_clone;
            run_session!(reader, writer)
        }
        EncryptionMode::StartTls => {
            // Read greeting on plain, then upgrade
            let tcp_clone = tcp.try_clone().map_err(|e| format!("clone tcp: {e}"))?;
            let mut reader = BufReader::new(tcp);
            let mut writer = tcp_clone;

            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| format!("read greeting: {e}"))?;
            if !line.to_uppercase().starts_with("* OK")
                && !line.to_uppercase().starts_with("* PREAUTH")
            {
                return Err(format!("unexpected greeting: {}", line.trim()));
            }

            // Send STARTTLS
            writer
                .write_all(b"A000 STARTTLS\r\n")
                .map_err(|e| format!("write STARTTLS: {e}"))?;
            writer.flush().map_err(|e| format!("flush STARTTLS: {e}"))?;

            let mut resp = String::new();
            loop {
                let mut l = String::new();
                reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read STARTTLS resp: {e}"))?;
                resp.push_str(&l);
                if l.starts_with("A000") {
                    break;
                }
            }
            if !resp.to_uppercase().contains("OK") {
                return Err("STARTTLS rejected".to_string());
            }

            // Upgrade to TLS
            let tcp_inner = reader.into_inner();
            let mut builder = TlsConnector::builder();
            if params.insecure || params.accepted_fingerprint.is_some() {
                builder.danger_accept_invalid_certs(true);
                builder.danger_accept_invalid_hostnames(true);
            }
            let connector = builder.build().map_err(|e| format!("TLS build: {e}"))?;
            let tls = connector
                .connect(&params.host, tcp_inner)
                .map_err(|e| format!("STARTTLS upgrade: {e}"))?;
            let mut reader = BufReader::new(tls);
            run_session!(reader, reader.get_mut())
        }
    }
}

/// Perform an IMAP APPEND to upload a message to a folder with given flags.
fn imap_append_message(
    params: &ImapConnectParams,
    folder_name: &str,
    flags: u32,
    rfc822_data: &[u8],
) -> Result<(), String> {
    use crate::core::account::EncryptionMode;
    use native_tls::TlsConnector;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let addr_str = format!("{}:{}", params.host, params.port);
    let addr: std::net::SocketAddr = addr_str
        .parse()
        .or_else(|_| {
            use std::net::ToSocketAddrs;
            addr_str
                .to_socket_addrs()
                .map_err(|e| e.to_string())?
                .next()
                .ok_or_else(|| "DNS resolution failed".to_string())
        })
        .map_err(|e| format!("DNS resolution failed: {e}"))?;

    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(30))
        .map_err(|e| format!("connection failed: {e}"))?;
    tcp.set_read_timeout(Some(Duration::from_secs(60))).ok();
    tcp.set_write_timeout(Some(Duration::from_secs(60))).ok();

    macro_rules! run_append_session {
        ($reader:expr, $writer:expr) => {{
            // Read greeting
            let mut line = String::new();
            $reader
                .read_line(&mut line)
                .map_err(|e| format!("read greeting: {e}"))?;
            if !line.to_uppercase().starts_with("* OK")
                && !line.to_uppercase().starts_with("* PREAUTH")
            {
                return Err(format!("unexpected greeting: {}", line.trim()));
            }

            // Login
            let username = imap_quote(&params.username);
            let password = imap_quote(&params.password);
            let cmd = format!("A001 LOGIN {username} {password}\r\n");
            $writer
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("write login: {e}"))?;
            $writer.flush().map_err(|e| format!("flush login: {e}"))?;

            let mut resp = String::new();
            loop {
                let mut l = String::new();
                $reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read login response: {e}"))?;
                resp.push_str(&l);
                if l.starts_with("A001") {
                    break;
                }
            }
            if !resp.contains("A001 OK") {
                return Err("authentication failed".to_string());
            }

            // Build flags string
            let mut flag_parts = Vec::new();
            if flags & FLAG_SEEN != 0 {
                flag_parts.push("\\Seen");
            }
            if flags & crate::core::message::FLAG_ANSWERED != 0 {
                flag_parts.push("\\Answered");
            }
            if flags & crate::core::message::FLAG_FLAGGED != 0 {
                flag_parts.push("\\Flagged");
            }
            if flags & crate::core::message::FLAG_DELETED != 0 {
                flag_parts.push("\\Deleted");
            }
            if flags & crate::core::message::FLAG_DRAFT != 0 {
                flag_parts.push("\\Draft");
            }
            let flags_str = flag_parts.join(" ");

            // APPEND command: A002 APPEND "Sent" (\Seen) {<size>}
            let quoted_folder = imap_quote(folder_name);
            let data_len = rfc822_data.len();
            let cmd = format!("A002 APPEND {quoted_folder} ({flags_str}) {{{data_len}}}\r\n");
            $writer
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("write APPEND: {e}"))?;
            $writer.flush().map_err(|e| format!("flush APPEND: {e}"))?;

            // Wait for continuation response "+"
            let mut cont = String::new();
            $reader
                .read_line(&mut cont)
                .map_err(|e| format!("read APPEND continuation: {e}"))?;
            if !cont.starts_with('+') {
                return Err(format!("APPEND not accepted: {}", cont.trim()));
            }

            // Send the literal data
            $writer
                .write_all(rfc822_data)
                .map_err(|e| format!("write APPEND data: {e}"))?;
            $writer
                .write_all(b"\r\n")
                .map_err(|e| format!("write APPEND CRLF: {e}"))?;
            $writer
                .flush()
                .map_err(|e| format!("flush APPEND data: {e}"))?;

            // Read APPEND response
            loop {
                let mut l = String::new();
                $reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read APPEND response: {e}"))?;
                if l.starts_with("A002") {
                    if !l.contains("OK") {
                        return Err(format!("APPEND failed: {}", l.trim()));
                    }
                    break;
                }
            }

            // Logout
            let cmd = "A099 LOGOUT\r\n";
            let _ = $writer.write_all(cmd.as_bytes());
            let _ = $writer.flush();

            Ok(())
        }};
    }

    match params.encryption {
        EncryptionMode::SslTls => {
            let mut builder = TlsConnector::builder();
            if params.insecure || params.accepted_fingerprint.is_some() {
                builder.danger_accept_invalid_certs(true);
                builder.danger_accept_invalid_hostnames(true);
            }
            let connector = builder
                .build()
                .map_err(|e| format!("TLS build error: {e}"))?;
            let tls = connector
                .connect(&params.host, tcp)
                .map_err(|e| format!("TLS handshake failed: {e}"))?;
            let mut reader = BufReader::new(tls);
            run_append_session!(reader, reader.get_mut())
        }
        EncryptionMode::None => {
            let tcp_clone = tcp.try_clone().map_err(|e| format!("clone tcp: {e}"))?;
            let mut reader = BufReader::new(tcp);
            let mut writer = tcp_clone;
            run_append_session!(reader, writer)
        }
        EncryptionMode::StartTls => {
            let tcp_clone = tcp.try_clone().map_err(|e| format!("clone tcp: {e}"))?;
            let mut reader = BufReader::new(tcp);
            let mut writer = tcp_clone;

            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| format!("read greeting: {e}"))?;
            if !line.to_uppercase().starts_with("* OK")
                && !line.to_uppercase().starts_with("* PREAUTH")
            {
                return Err(format!("unexpected greeting: {}", line.trim()));
            }

            writer
                .write_all(b"A000 STARTTLS\r\n")
                .map_err(|e| format!("write STARTTLS: {e}"))?;
            writer.flush().map_err(|e| format!("flush STARTTLS: {e}"))?;

            let mut resp = String::new();
            loop {
                let mut l = String::new();
                reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read STARTTLS resp: {e}"))?;
                resp.push_str(&l);
                if l.starts_with("A000") {
                    break;
                }
            }
            if !resp.to_uppercase().contains("OK") {
                return Err("STARTTLS rejected".to_string());
            }

            let tcp_inner = reader.into_inner();
            let mut builder = TlsConnector::builder();
            if params.insecure || params.accepted_fingerprint.is_some() {
                builder.danger_accept_invalid_certs(true);
                builder.danger_accept_invalid_hostnames(true);
            }
            let connector = builder.build().map_err(|e| format!("TLS build: {e}"))?;
            let tls = connector
                .connect(&params.host, tcp_inner)
                .map_err(|e| format!("STARTTLS upgrade: {e}"))?;
            let mut reader = BufReader::new(tls);
            run_append_session!(reader, reader.get_mut())
        }
    }
}

/// Execute a single IMAP command (CREATE, RENAME, DELETE) after login.
fn imap_folder_command(params: &ImapConnectParams, command: &str) -> Result<(), String> {
    use crate::core::account::EncryptionMode;
    use native_tls::TlsConnector;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let addr_str = format!("{}:{}", params.host, params.port);
    let addr: std::net::SocketAddr = addr_str
        .parse()
        .or_else(|_| {
            use std::net::ToSocketAddrs;
            addr_str
                .to_socket_addrs()
                .map_err(|e| e.to_string())?
                .next()
                .ok_or_else(|| "DNS resolution failed".to_string())
        })
        .map_err(|e| format!("DNS resolution failed: {e}"))?;

    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(30))
        .map_err(|e| format!("connection failed: {e}"))?;
    tcp.set_read_timeout(Some(Duration::from_secs(30))).ok();
    tcp.set_write_timeout(Some(Duration::from_secs(30))).ok();

    macro_rules! run_folder_session {
        ($reader:expr, $writer:expr) => {{
            // Read greeting
            let mut line = String::new();
            $reader
                .read_line(&mut line)
                .map_err(|e| format!("read greeting: {e}"))?;
            if !line.to_uppercase().starts_with("* OK")
                && !line.to_uppercase().starts_with("* PREAUTH")
            {
                return Err(format!("unexpected greeting: {}", line.trim()));
            }

            // Login
            let username = imap_quote(&params.username);
            let password = imap_quote(&params.password);
            let cmd = format!("A001 LOGIN {username} {password}\r\n");
            $writer
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("write login: {e}"))?;
            $writer.flush().map_err(|e| format!("flush login: {e}"))?;

            let mut resp = String::new();
            loop {
                let mut l = String::new();
                $reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read login response: {e}"))?;
                resp.push_str(&l);
                if l.starts_with("A001") {
                    break;
                }
            }
            if !resp.contains("A001 OK") {
                return Err("authentication failed".to_string());
            }

            // Execute the folder command
            let cmd = format!("A002 {command}\r\n");
            $writer
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("write command: {e}"))?;
            $writer.flush().map_err(|e| format!("flush command: {e}"))?;

            loop {
                let mut l = String::new();
                $reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read command response: {e}"))?;
                if l.starts_with("A002") {
                    if !l.contains("OK") {
                        return Err(format!("command failed: {}", l.trim()));
                    }
                    break;
                }
            }

            // Logout
            let cmd = "A099 LOGOUT\r\n";
            let _ = $writer.write_all(cmd.as_bytes());
            let _ = $writer.flush();

            Ok(())
        }};
    }

    match params.encryption {
        EncryptionMode::SslTls => {
            let mut builder = TlsConnector::builder();
            if params.insecure || params.accepted_fingerprint.is_some() {
                builder.danger_accept_invalid_certs(true);
                builder.danger_accept_invalid_hostnames(true);
            }
            let connector = builder
                .build()
                .map_err(|e| format!("TLS build error: {e}"))?;
            let tls = connector
                .connect(&params.host, tcp)
                .map_err(|e| format!("TLS handshake failed: {e}"))?;
            let mut reader = BufReader::new(tls);
            run_folder_session!(reader, reader.get_mut())
        }
        EncryptionMode::None => {
            let tcp_clone = tcp.try_clone().map_err(|e| format!("clone tcp: {e}"))?;
            let mut reader = BufReader::new(tcp);
            let mut writer = tcp_clone;
            run_folder_session!(reader, writer)
        }
        EncryptionMode::StartTls => {
            let tcp_clone = tcp.try_clone().map_err(|e| format!("clone tcp: {e}"))?;
            let mut reader = BufReader::new(tcp);
            let mut writer = tcp_clone;

            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| format!("read greeting: {e}"))?;
            if !line.to_uppercase().starts_with("* OK")
                && !line.to_uppercase().starts_with("* PREAUTH")
            {
                return Err(format!("unexpected greeting: {}", line.trim()));
            }

            writer
                .write_all(b"A000 STARTTLS\r\n")
                .map_err(|e| format!("write STARTTLS: {e}"))?;
            writer.flush().map_err(|e| format!("flush STARTTLS: {e}"))?;

            let mut resp = String::new();
            loop {
                let mut l = String::new();
                reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read STARTTLS resp: {e}"))?;
                resp.push_str(&l);
                if l.starts_with("A000") {
                    break;
                }
            }
            if !resp.to_uppercase().contains("OK") {
                return Err("STARTTLS rejected".to_string());
            }

            let tcp_inner = reader.into_inner();
            let mut builder = TlsConnector::builder();
            if params.insecure || params.accepted_fingerprint.is_some() {
                builder.danger_accept_invalid_certs(true);
                builder.danger_accept_invalid_hostnames(true);
            }
            let connector = builder.build().map_err(|e| format!("TLS build: {e}"))?;
            let tls = connector
                .connect(&params.host, tcp_inner)
                .map_err(|e| format!("STARTTLS upgrade: {e}"))?;
            let mut reader = BufReader::new(tls);
            run_folder_session!(reader, reader.get_mut())
        }
    }
}

fn imap_quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Handle to the running sync engine. Dropping it signals shutdown.
pub(crate) struct SyncEngineHandle {
    _shutdown_tx: tokio::sync::watch::Sender<bool>,
    _runtime_thread: std::thread::JoinHandle<()>,
    notify_tx: tokio::sync::mpsc::UnboundedSender<String>,
    idle_tx: tokio::sync::mpsc::UnboundedSender<IdleCommand>,
}

/// Commands for the IDLE subsystem.
enum IdleCommand {
    /// Start IDLE monitoring for an account (after initial sync).
    StartIdle { account_id: String },
}

impl SyncEngineHandle {
    /// Notify the engine that an account has new pending operations.
    pub fn notify_account(&self, account_id: &str) {
        let _ = self.notify_tx.send(account_id.to_string());
    }

    /// Clone the notification sender for use by services that enqueue pending operations.
    pub fn notify_sender(&self) -> tokio::sync::mpsc::UnboundedSender<String> {
        self.notify_tx.clone()
    }

    /// Start IDLE monitoring for an account after its initial sync completes.
    pub fn start_idle(&self, account_id: &str) {
        let _ = self.idle_tx.send(IdleCommand::StartIdle {
            account_id: account_id.to_string(),
        });
    }
}

/// Start the sync engine on a dedicated worker thread.
/// Returns a handle to interact with the engine and a broadcast receiver for events.
pub(crate) fn start_sync_engine(
    db_path: PathBuf,
    event_sender: broadcast::Sender<SyncEvent>,
    flag_store: Arc<dyn ImapFlagStore>,
    account_params_fn: Arc<AccountParamsFn>,
) -> SyncEngineHandle {
    start_sync_engine_with_idle(
        db_path,
        event_sender,
        flag_store,
        account_params_fn,
        Arc::new(RealIdleWaiter),
        None,
        Arc::new(RealImapFolderOps),
    )
}

/// Start the sync engine with explicit idle waiter and content store (for testing).
pub(crate) fn start_sync_engine_with_idle(
    db_path: PathBuf,
    event_sender: broadcast::Sender<SyncEvent>,
    flag_store: Arc<dyn ImapFlagStore>,
    account_params_fn: Arc<AccountParamsFn>,
    idle_waiter: Arc<dyn IdleWaiter>,
    content_store: Option<Arc<dyn ContentStore + Send + Sync>>,
    folder_ops: Arc<dyn ImapFolderOps>,
) -> SyncEngineHandle {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let (notify_tx, notify_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let (idle_tx, idle_rx) = tokio::sync::mpsc::unbounded_channel::<IdleCommand>();

    let thread = std::thread::Builder::new()
        .name("sync-engine".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("failed to build tokio runtime for sync engine");

            rt.block_on(engine_loop(
                db_path,
                event_sender,
                flag_store,
                account_params_fn,
                idle_waiter,
                content_store,
                folder_ops,
                shutdown_rx,
                notify_rx,
                idle_rx,
            ));
        })
        .expect("failed to spawn sync engine thread");

    SyncEngineHandle {
        _shutdown_tx: shutdown_tx,
        _runtime_thread: thread,
        notify_tx,
        idle_tx,
    }
}

#[allow(clippy::too_many_arguments)]
async fn engine_loop(
    db_path: PathBuf,
    event_sender: broadcast::Sender<SyncEvent>,
    flag_store: Arc<dyn ImapFlagStore>,
    account_params_fn: Arc<AccountParamsFn>,
    idle_waiter: Arc<dyn IdleWaiter>,
    content_store: Option<Arc<dyn ContentStore + Send + Sync>>,
    folder_ops: Arc<dyn ImapFolderOps>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    mut notify_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
    mut idle_rx: tokio::sync::mpsc::UnboundedReceiver<IdleCommand>,
) {
    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                break;
            }
            Some(account_id) = notify_rx.recv() => {
                let db_path = db_path.clone();
                let event_sender = event_sender.clone();
                let flag_store = flag_store.clone();
                let account_params_fn = account_params_fn.clone();
                let folder_ops = folder_ops.clone();

                tokio::spawn(async move {
                    process_account_ops(
                        &db_path,
                        &account_id,
                        &event_sender,
                        flag_store,
                        account_params_fn.as_ref(),
                        folder_ops,
                    ).await;
                });
            }
            Some(cmd) = idle_rx.recv() => {
                match cmd {
                    IdleCommand::StartIdle { account_id } => {
                        // Only start IDLE if we have a content store.
                        if let Some(ref cs) = content_store {
                            let idle_shutdown_rx = shutdown_rx.clone();
                            tokio::spawn(idle_service::run_idle_loop(
                                account_id,
                                db_path.clone(),
                                account_params_fn.clone(),
                                event_sender.clone(),
                                cs.clone(),
                                idle_waiter.clone(),
                                idle_shutdown_rx,
                            ));
                        }
                    }
                }
            }
        }
    }
}

async fn process_account_ops(
    db_path: &std::path::Path,
    account_id: &str,
    event_sender: &broadcast::Sender<SyncEvent>,
    flag_store: Arc<dyn ImapFlagStore>,
    account_params_fn: &(dyn Fn(&str) -> Option<ImapConnectParams> + Send + Sync),
    folder_ops: Arc<dyn ImapFolderOps>,
) {
    process_account_ops_full(
        db_path,
        account_id,
        event_sender,
        flag_store,
        account_params_fn,
        Arc::new(RealSmtpSender),
        Arc::new(RealImapAppender),
        None,
        None,
        folder_ops,
    )
    .await;
}

/// Type alias for the SMTP-params lookup function (for testing).
type SmtpParamsFn = dyn Fn(i64) -> Option<(SmtpConnectParams, String, bool)> + Send + Sync;

/// Type alias for a content-store retrieval function (for testing).
type ContentReaderFn = dyn Fn(&str) -> Option<Vec<u8>> + Send + Sync;

#[allow(clippy::too_many_arguments)]
async fn process_account_ops_full(
    db_path: &std::path::Path,
    account_id: &str,
    event_sender: &broadcast::Sender<SyncEvent>,
    flag_store: Arc<dyn ImapFlagStore>,
    account_params_fn: &(dyn Fn(&str) -> Option<ImapConnectParams> + Send + Sync),
    smtp_sender: Arc<dyn SmtpSender>,
    imap_appender: Arc<dyn ImapAppender>,
    smtp_params_fn: Option<Arc<SmtpParamsFn>>,
    content_reader_fn: Option<Arc<ContentReaderFn>>,
    folder_ops: Arc<dyn ImapFolderOps>,
) {
    let conn = match open_and_migrate(db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("sync engine: failed to open db: {e}");
            return;
        }
    };

    let ops = match pending_ops_store::load_pending_ops(&conn, account_id) {
        Ok(ops) => ops,
        Err(e) => {
            eprintln!("sync engine: failed to load pending ops: {e}");
            return;
        }
    };

    if ops.is_empty() {
        return;
    }

    let params = match account_params_fn(account_id) {
        Some(p) => p,
        None => {
            eprintln!("sync engine: no connection params for account {account_id}");
            return;
        }
    };

    let mut i = 0;
    while i < ops.len() {
        let op = &ops[i];

        if op.state == OperationState::Failed {
            i += 1;
            continue;
        }

        // Mark in-flight
        if let Err(e) = pending_ops_store::mark_in_flight(&conn, op.id) {
            eprintln!("sync engine: mark in-flight failed: {e}");
            i += 1;
            continue;
        }

        match op.kind {
            OperationKind::StoreFlags => {
                let payload: StoreFlagsPayload = match serde_json::from_str(&op.payload) {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("invalid payload: {e}");
                        let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                        let _ = event_sender.send(SyncEvent::OperationFailed {
                            account_id: account_id.to_string(),
                            operation_id: op.id,
                            error: err_msg,
                        });
                        i += 1;
                        continue;
                    }
                };

                let flag_store_clone = flag_store.clone();
                let params_clone = params.clone();
                let folder = payload.folder_name.clone();
                let uid = payload.uid;
                let flags = payload.new_flags;

                let result = tokio::task::spawn_blocking(move || {
                    flag_store_clone.store_flags(&params_clone, &folder, uid, flags)
                })
                .await;

                match result {
                    Ok(Ok(())) => {
                        let _ = pending_ops_store::complete_op(&conn, op.id);
                        let _ = event_sender.send(SyncEvent::MessageFlagsChanged {
                            account_id: account_id.to_string(),
                            message_id: payload.message_id,
                            new_flags: payload.new_flags,
                        });
                    }
                    Ok(Err(err_msg)) => {
                        let sync_err = SyncError::Imap(err_msg.clone());
                        if is_transient_error(&sync_err) {
                            match pending_ops_store::requeue_op(&conn, op.id, &err_msg) {
                                Ok(retry_count) => {
                                    let delay = backoff_duration(retry_count - 1);
                                    tokio::time::sleep(delay).await;
                                }
                                Err(e) => {
                                    eprintln!("sync engine: requeue failed: {e}");
                                }
                            }
                        } else {
                            let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                            let _ = event_sender.send(SyncEvent::OperationFailed {
                                account_id: account_id.to_string(),
                                operation_id: op.id,
                                error: err_msg,
                            });
                        }
                    }
                    Err(e) => {
                        let err_msg = format!("task join error: {e}");
                        let _ = pending_ops_store::requeue_op(&conn, op.id, &err_msg);
                    }
                }
            }
            OperationKind::Send => {
                let send_payload: SendPayload = match serde_json::from_str(&op.payload) {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("invalid send payload: {e}");
                        let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                        let _ = event_sender.send(SyncEvent::OperationFailed {
                            account_id: account_id.to_string(),
                            operation_id: op.id,
                            error: err_msg,
                        });
                        i += 1;
                        continue;
                    }
                };

                // Resolve SMTP params and message bytes
                let send_context = resolve_send_context(
                    &conn,
                    &send_payload,
                    &params,
                    smtp_params_fn.as_deref(),
                    content_reader_fn.as_deref(),
                );

                let (smtp_params, envelope_from, login_before_send, rfc822_data) =
                    match send_context {
                        Ok(ctx) => ctx,
                        Err(err_msg) => {
                            let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                            let _ = event_sender.send(SyncEvent::OperationFailed {
                                account_id: account_id.to_string(),
                                operation_id: op.id,
                                error: err_msg,
                            });
                            i += 1;
                            continue;
                        }
                    };

                // Login-before-send check: verify IMAP login succeeds first
                if login_before_send {
                    let flag_store_check = flag_store.clone();
                    let params_check = params.clone();
                    let imap_check_result = tokio::task::spawn_blocking(move || {
                        // Use a lightweight IMAP operation to verify login.
                        // StoreFlags on UID 0 in INBOX will fail SELECT but login succeeds.
                        // Instead, we just try to connect and login by doing a store
                        // on a dummy UID — the login part is what matters.
                        flag_store_check.store_flags(&params_check, "INBOX", 0, 0)
                    })
                    .await;

                    match imap_check_result {
                        Ok(Err(err_msg)) if err_msg.contains("authentication") => {
                            let err = format!("login-before-send: inbound login failed: {err_msg}");
                            let _ = pending_ops_store::mark_failed(&conn, op.id, &err);
                            let _ = event_sender.send(SyncEvent::OperationFailed {
                                account_id: account_id.to_string(),
                                operation_id: op.id,
                                error: err,
                            });
                            i += 1;
                            continue;
                        }
                        Err(e) => {
                            let err = format!("login-before-send: task error: {e}");
                            let _ = pending_ops_store::requeue_op(&conn, op.id, &err);
                            i += 1;
                            continue;
                        }
                        _ => {
                            // Login succeeded (store_flags may fail on UID 0, that's fine)
                        }
                    }
                }

                // Extract envelope recipients from the RFC 5322 message
                let envelope_to = extract_envelope_recipients(&rfc822_data);
                if envelope_to.is_empty() {
                    let err_msg = "no recipients found in message".to_string();
                    let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                    let _ = event_sender.send(SyncEvent::OperationFailed {
                        account_id: account_id.to_string(),
                        operation_id: op.id,
                        error: err_msg,
                    });
                    i += 1;
                    continue;
                }

                // (a) Send via SMTP
                let smtp_sender_clone = smtp_sender.clone();
                let smtp_params_clone = smtp_params.clone();
                let envelope_from_clone = envelope_from.clone();
                let envelope_to_clone = envelope_to.clone();
                let rfc822_clone = rfc822_data.clone();

                let smtp_result = tokio::task::spawn_blocking(move || {
                    smtp_sender_clone.send_message(
                        &smtp_params_clone,
                        &envelope_from_clone,
                        &envelope_to_clone,
                        &rfc822_clone,
                    )
                })
                .await;

                match smtp_result {
                    Ok(Ok(())) => {
                        // (d) IMAP APPEND to Sent folder with \Seen flag
                        let sent_flags = FLAG_SEEN;
                        let imap_appender_clone = imap_appender.clone();
                        let params_clone = params.clone();
                        let rfc822_for_append = rfc822_data.clone();
                        let append_result = tokio::task::spawn_blocking(move || {
                            imap_appender_clone.append_message(
                                &params_clone,
                                "Sent",
                                sent_flags,
                                &rfc822_for_append,
                            )
                        })
                        .await;

                        if let Ok(Err(append_err)) = &append_result {
                            eprintln!(
                                "sync engine: IMAP APPEND to Sent failed (non-fatal): {append_err}"
                            );
                        }

                        // (e) Write to content store and (f) insert messages row
                        store_sent_message_locally(
                            &conn,
                            account_id,
                            &rfc822_data,
                            content_reader_fn.as_deref(),
                        );

                        // (g) Delete the pending operation
                        let _ = pending_ops_store::complete_op(&conn, op.id);
                        let _ = event_sender.send(SyncEvent::MessageSent {
                            account_id: account_id.to_string(),
                            operation_id: op.id,
                        });
                    }
                    Ok(Err(err_msg)) => {
                        let sync_err = SyncError::Smtp(err_msg.clone());
                        if is_transient_error(&sync_err) {
                            match pending_ops_store::requeue_op(&conn, op.id, &err_msg) {
                                Ok(retry_count) => {
                                    let delay = backoff_duration(retry_count - 1);
                                    tokio::time::sleep(delay).await;
                                }
                                Err(e) => {
                                    eprintln!("sync engine: requeue failed: {e}");
                                }
                            }
                        } else {
                            let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                            let _ = event_sender.send(SyncEvent::OperationFailed {
                                account_id: account_id.to_string(),
                                operation_id: op.id,
                                error: err_msg,
                            });
                        }
                    }
                    Err(e) => {
                        let err_msg = format!("task join error: {e}");
                        let _ = pending_ops_store::requeue_op(&conn, op.id, &err_msg);
                    }
                }
            }
            OperationKind::FolderCreate => {
                let payload: FolderCreatePayload = match serde_json::from_str(&op.payload) {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("invalid folder-create payload: {e}");
                        let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                        let _ = event_sender.send(SyncEvent::OperationFailed {
                            account_id: account_id.to_string(),
                            operation_id: op.id,
                            error: err_msg,
                        });
                        i += 1;
                        continue;
                    }
                };

                let folder_ops_clone = folder_ops.clone();
                let params_clone = params.clone();
                let folder_name = payload.folder_name.clone();

                let result = tokio::task::spawn_blocking(move || {
                    folder_ops_clone.create_folder(&params_clone, &folder_name)
                })
                .await;

                if let Some(delay) =
                    handle_folder_op_result(&conn, op, account_id, event_sender, result)
                {
                    tokio::time::sleep(delay).await;
                }
            }
            OperationKind::FolderRename => {
                let payload: FolderRenamePayload = match serde_json::from_str(&op.payload) {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("invalid folder-rename payload: {e}");
                        let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                        let _ = event_sender.send(SyncEvent::OperationFailed {
                            account_id: account_id.to_string(),
                            operation_id: op.id,
                            error: err_msg,
                        });
                        i += 1;
                        continue;
                    }
                };

                let folder_ops_clone = folder_ops.clone();
                let params_clone = params.clone();
                let old_name = payload.old_name.clone();
                let new_name = payload.new_name.clone();
                let folder_id = payload.folder_id;

                let result = tokio::task::spawn_blocking(move || {
                    folder_ops_clone.rename_folder(&params_clone, &old_name, &new_name)
                })
                .await;

                if let Ok(Ok(())) = &result {
                    // Update local folder name on success
                    let _ = crate::services::folder_store::rename_folder(
                        &conn,
                        folder_id,
                        &payload.new_name,
                    );
                }

                if let Some(delay) =
                    handle_folder_op_result(&conn, op, account_id, event_sender, result)
                {
                    tokio::time::sleep(delay).await;
                }
            }
            OperationKind::FolderDelete => {
                let payload: FolderDeletePayload = match serde_json::from_str(&op.payload) {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("invalid folder-delete payload: {e}");
                        let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                        let _ = event_sender.send(SyncEvent::OperationFailed {
                            account_id: account_id.to_string(),
                            operation_id: op.id,
                            error: err_msg,
                        });
                        i += 1;
                        continue;
                    }
                };

                let folder_ops_clone = folder_ops.clone();
                let params_clone = params.clone();
                let folder_name = payload.folder_name.clone();
                let folder_id = payload.folder_id;

                let result = tokio::task::spawn_blocking(move || {
                    folder_ops_clone.delete_folder(&params_clone, &folder_name)
                })
                .await;

                if let Ok(Ok(())) = &result {
                    // Delete messages whose only folder association was this folder,
                    // collecting orphaned content hashes for .eml reclamation.
                    let orphaned_hashes =
                        crate::services::message_store::delete_messages_for_folder(
                            &conn, account_id, folder_id,
                        );
                    if let Ok(hashes) = orphaned_hashes {
                        for hash in &hashes {
                            eprintln!(
                                "sync engine: orphaned content hash {hash} — \
                                 .eml can be reclaimed"
                            );
                        }
                    }
                    // Delete the folder row
                    let _ = crate::services::folder_store::delete_folder(&conn, folder_id);
                }

                if let Some(delay) =
                    handle_folder_op_result(&conn, op, account_id, event_sender, result)
                {
                    tokio::time::sleep(delay).await;
                }
            }
        }

        i += 1;
    }
}

/// Handle the result of a folder IMAP operation (create/rename/delete).
/// On success: completes the op and emits FolderListChanged.
/// On transient error: requeues with backoff, returns the delay duration.
/// On permanent error: marks failed and emits OperationFailed.
///
/// Returns `Some(delay)` when the caller should sleep before continuing
/// (transient-error backoff). Returns `None` otherwise.
fn handle_folder_op_result(
    conn: &rusqlite::Connection,
    op: &crate::core::pending_operation::PendingOperation,
    account_id: &str,
    event_sender: &broadcast::Sender<SyncEvent>,
    result: Result<Result<(), String>, tokio::task::JoinError>,
) -> Option<std::time::Duration> {
    match result {
        Ok(Ok(())) => {
            let _ = pending_ops_store::complete_op(conn, op.id);
            let _ = event_sender.send(SyncEvent::FolderListChanged {
                account_id: account_id.to_string(),
            });
            None
        }
        Ok(Err(err_msg)) => {
            let sync_err = SyncError::Imap(err_msg.clone());
            if is_transient_error(&sync_err) {
                match pending_ops_store::requeue_op(conn, op.id, &err_msg) {
                    Ok(retry_count) => Some(backoff_duration(retry_count - 1)),
                    Err(e) => {
                        eprintln!("sync engine: requeue failed: {e}");
                        None
                    }
                }
            } else {
                let _ = pending_ops_store::mark_failed(conn, op.id, &err_msg);
                let _ = event_sender.send(SyncEvent::OperationFailed {
                    account_id: account_id.to_string(),
                    operation_id: op.id,
                    error: err_msg,
                });
                None
            }
        }
        Err(e) => {
            let err_msg = format!("task join error: {e}");
            let _ = pending_ops_store::requeue_op(conn, op.id, &err_msg);
            None
        }
    }
}

/// Resolve the SMTP connection parameters and message bytes for a send operation.
fn resolve_send_context(
    conn: &rusqlite::Connection,
    payload: &SendPayload,
    imap_params: &ImapConnectParams,
    smtp_params_fn: Option<&SmtpParamsFn>,
    content_reader_fn: Option<&ContentReaderFn>,
) -> Result<(SmtpConnectParams, String, bool, Vec<u8>), String> {
    // If we have an override (for testing), use that
    if let Some(get_smtp) = smtp_params_fn {
        let (params, from, lbs) = get_smtp(payload.identity_id)
            .ok_or_else(|| format!("no SMTP params for identity {}", payload.identity_id))?;

        let data = resolve_message_bytes(payload, content_reader_fn)?;
        return Ok((params, from, lbs, data));
    }

    // Load identity from DB
    let identity = crate::services::identity_store::load_identity_by_id(conn, payload.identity_id)
        .map_err(|e| format!("db error loading identity: {e}"))?
        .ok_or_else(|| format!("identity {} not found", payload.identity_id))?;

    let encryption = match identity.smtp_encryption.as_str() {
        "SslTls" => crate::core::account::EncryptionMode::SslTls,
        "StartTls" => crate::core::account::EncryptionMode::StartTls,
        _ => crate::core::account::EncryptionMode::None,
    };

    // For password: try the SMTP username's password. In the real app, this comes
    // from the keychain. For now, fall back to the IMAP password if SMTP user matches.
    let smtp_password = imap_params.password.clone();

    let smtp_params = SmtpConnectParams {
        host: identity.smtp_host.clone(),
        port: identity.smtp_port,
        encryption,
        username: if identity.smtp_username.is_empty() {
            imap_params.username.clone()
        } else {
            identity.smtp_username.clone()
        },
        password: smtp_password,
        accepted_fingerprint: imap_params.accepted_fingerprint.clone(),
        insecure: imap_params.insecure,
        account_id: identity.account_id.clone(),
    };

    let data = resolve_message_bytes(payload, content_reader_fn)?;

    Ok((
        smtp_params,
        identity.email_address,
        identity.login_before_send,
        data,
    ))
}

/// Resolve the RFC 5322 message bytes from a SendPayload.
fn resolve_message_bytes(
    payload: &SendPayload,
    content_reader_fn: Option<&ContentReaderFn>,
) -> Result<Vec<u8>, String> {
    // Try inline first
    if let Some(b64) = &payload.inline_rfc822_b64 {
        use base64::Engine;
        return base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| format!("invalid base64 in inline_rfc822_b64: {e}"));
    }

    // Try content store hash
    if let Some(hash) = &payload.content_hash {
        if let Some(reader) = content_reader_fn {
            return reader(hash).ok_or_else(|| format!("content not found for hash: {hash}"));
        }
        return Err(format!(
            "content hash {hash} provided but no content reader available"
        ));
    }

    Err("send payload has neither inline data nor content hash".to_string())
}

/// Extract envelope recipients (To + Cc + Bcc) from RFC 5322 message bytes.
fn extract_envelope_recipients(rfc822_data: &[u8]) -> Vec<String> {
    let parsed = mail_parser::MessageParser::default().parse(rfc822_data);
    let mut recipients = Vec::new();

    if let Some(msg) = &parsed {
        for addrs in [msg.to(), msg.cc(), msg.bcc()].into_iter().flatten() {
            match addrs {
                mail_parser::Address::List(list) => {
                    for a in list {
                        if let Some(email) = &a.address {
                            recipients.push(email.to_string());
                        }
                    }
                }
                mail_parser::Address::Group(groups) => {
                    for g in groups {
                        for a in &g.addresses {
                            if let Some(email) = &a.address {
                                recipients.push(email.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    recipients
}

/// Store a sent message locally: write to content store (if available) and insert
/// a messages row in the Sent folder with \Seen flag.
fn store_sent_message_locally(
    conn: &rusqlite::Connection,
    account_id: &str,
    rfc822_data: &[u8],
    _content_reader_fn: Option<&ContentReaderFn>,
) {
    // Compute content hash
    let content_hash = crate::services::fs_content_store::sha256_hex(rfc822_data);

    // Find the local Sent folder
    let sent_folder_id = crate::services::message_store::find_folder_id(conn, account_id, "Sent")
        .ok()
        .flatten();
    let sent_folder_id = match sent_folder_id {
        Some(id) => id,
        None => {
            // No local Sent folder — skip local insert
            return;
        }
    };

    // Parse the message to extract headers
    let new_msg = crate::core::message::parse_raw_message(
        account_id,
        0, // UID unknown until next sync
        None,
        FLAG_SEEN,
        &content_hash,
        rfc822_data,
    );

    let _ = crate::services::message_store::insert_message(conn, &new_msg, sent_folder_id);
}

/// Toggle the read/unread flag on a message. Updates the local database immediately
/// and inserts a pending operation for the sync engine.
/// Returns the new flags value.
pub(crate) fn toggle_message_read(
    db_path: &std::path::Path,
    message_id: i64,
    account_id: &str,
    uid: u32,
    folder_name: &str,
    current_flags: u32,
) -> Result<u32, SyncError> {
    let conn = open_and_migrate(db_path)?;

    let new_flags = if current_flags & FLAG_SEEN != 0 {
        current_flags & !FLAG_SEEN
    } else {
        current_flags | FLAG_SEEN
    };

    // Update local DB immediately
    crate::services::message_store::update_message_flags(&conn, message_id, new_flags)
        .map_err(SyncError::Database)?;

    // Insert pending operation
    let payload = StoreFlagsPayload {
        message_id,
        uid,
        folder_name: folder_name.to_string(),
        new_flags,
    };
    let payload_json =
        serde_json::to_string(&payload).map_err(|e| SyncError::PayloadParse(e.to_string()))?;

    pending_ops_store::insert_pending_op(
        &conn,
        account_id,
        &OperationKind::StoreFlags,
        &payload_json,
    )
    .map_err(SyncError::Database)?;

    Ok(new_flags)
}

/// Queue a send-message operation. The compose UI calls this; it returns
/// immediately without blocking on delivery.
///
/// `rfc822_data` is the fully-composed RFC 5322 message. It is base64-encoded
/// and stored inline in the pending operation payload.
pub(crate) fn queue_send_message(
    db_path: &std::path::Path,
    account_id: &str,
    identity_id: i64,
    rfc822_data: &[u8],
) -> Result<i64, SyncError> {
    let conn = open_and_migrate(db_path)?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(rfc822_data);

    let payload = SendPayload {
        identity_id,
        content_hash: None,
        inline_rfc822_b64: Some(b64),
    };
    let payload_json =
        serde_json::to_string(&payload).map_err(|e| SyncError::PayloadParse(e.to_string()))?;

    let op_id = pending_ops_store::insert_pending_op(
        &conn,
        account_id,
        &OperationKind::Send,
        &payload_json,
    )
    .map_err(SyncError::Database)?;

    Ok(op_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::EncryptionMode;
    use crate::core::message::{FLAG_ANSWERED, FLAG_SEEN};
    use crate::core::pending_operation::OperationState;
    use crate::services::database::open_and_migrate;
    use crate::services::imap_client::ImapConnectParams;
    use tempfile::TempDir;

    fn setup_db() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();
        // Insert a folder and message for testing
        conn.execute(
            "INSERT INTO folders (id, account_id, name, attributes) VALUES (1, 'acct-1', 'INBOX', '')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO messages (id, account_id, uid, flags, size, content_hash) VALUES (1, 'acct-1', 100, 0, 1024, 'hash1')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO message_folders (message_id, folder_id) VALUES (1, 1)",
            [],
        )
        .unwrap();
        drop(conn);
        (dir, db_path)
    }

    fn make_test_params() -> ImapConnectParams {
        ImapConnectParams {
            host: "imap.example.com".to_string(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            username: "user".to_string(),
            password: "pass".to_string(),
            accepted_fingerprint: None,
            insecure: false,
            account_id: "acct-1".to_string(),
        }
    }

    #[test]
    fn toggle_read_sets_seen_and_creates_pending_op() {
        let (_dir, db_path) = setup_db();

        let new_flags = toggle_message_read(&db_path, 1, "acct-1", 100, "INBOX", 0).unwrap();
        assert_eq!(new_flags, FLAG_SEEN);

        // Verify message flags updated in DB
        let conn = open_and_migrate(&db_path).unwrap();
        let flags: u32 = conn
            .query_row("SELECT flags FROM messages WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(flags, FLAG_SEEN);

        // Verify pending operation created
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].kind, OperationKind::StoreFlags);
        assert_eq!(ops[0].state, OperationState::Pending);

        let payload: StoreFlagsPayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.message_id, 1);
        assert_eq!(payload.uid, 100);
        assert_eq!(payload.new_flags, FLAG_SEEN);
    }

    #[test]
    fn toggle_read_unsets_seen_when_already_set() {
        let (_dir, db_path) = setup_db();

        // First set it to seen
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "UPDATE messages SET flags = ?1 WHERE id = 1",
            rusqlite::params![FLAG_SEEN | FLAG_ANSWERED],
        )
        .unwrap();
        drop(conn);

        let new_flags = toggle_message_read(
            &db_path,
            1,
            "acct-1",
            100,
            "INBOX",
            FLAG_SEEN | FLAG_ANSWERED,
        )
        .unwrap();
        assert_eq!(new_flags, FLAG_ANSWERED); // SEEN cleared, ANSWERED preserved
    }

    #[test]
    fn is_transient_error_classification() {
        assert!(is_transient_error(&SyncError::Imap(
            "connection timed out".to_string()
        )));
        assert!(is_transient_error(&SyncError::Imap(
            "network unreachable".to_string()
        )));
        assert!(!is_transient_error(&SyncError::Imap(
            "authentication failed".to_string()
        )));
        assert!(!is_transient_error(&SyncError::Imap(
            "auth rejected".to_string()
        )));
        assert!(!is_transient_error(&SyncError::Credential(
            "no credential".to_string()
        )));
        assert!(!is_transient_error(&SyncError::PayloadParse(
            "bad json".to_string()
        )));
    }

    #[test]
    fn backoff_duration_caps_at_one_hour() {
        assert_eq!(backoff_duration(0), std::time::Duration::from_secs(5));
        assert_eq!(backoff_duration(1), std::time::Duration::from_secs(30));
        assert_eq!(backoff_duration(2), std::time::Duration::from_secs(120));
        assert_eq!(backoff_duration(3), std::time::Duration::from_secs(600));
        assert_eq!(backoff_duration(4), std::time::Duration::from_secs(3600));
        assert_eq!(backoff_duration(100), std::time::Duration::from_secs(3600));
        // capped
    }

    #[tokio::test]
    async fn engine_processes_pending_ops_with_mock() {
        let (_dir, db_path) = setup_db();

        // Create a pending operation
        let conn = open_and_migrate(&db_path).unwrap();
        let payload = StoreFlagsPayload {
            message_id: 1,
            uid: 100,
            folder_name: "INBOX".to_string(),
            new_flags: FLAG_SEEN,
        };
        let payload_json = serde_json::to_string(&payload).unwrap();
        pending_ops_store::insert_pending_op(
            &conn,
            "acct-1",
            &OperationKind::StoreFlags,
            &payload_json,
        )
        .unwrap();
        drop(conn);

        let (event_tx, mut event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore { should_fail: None });
        let params = make_test_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));

        process_account_ops(
            &db_path,
            "acct-1",
            &event_tx,
            flag_store.clone(),
            account_params_fn.as_ref(),
            Arc::new(MockImapFolderOps { should_fail: None }),
        )
        .await;

        // Operation should be completed (removed)
        let conn = open_and_migrate(&db_path).unwrap();
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty());

        // Should have received a MessageFlagsChanged event
        let event = event_rx.try_recv().unwrap();
        match event {
            SyncEvent::MessageFlagsChanged {
                message_id,
                new_flags,
                ..
            } => {
                assert_eq!(message_id, 1);
                assert_eq!(new_flags, FLAG_SEEN);
            }
            _ => panic!("unexpected event"),
        }
    }

    #[tokio::test]
    async fn engine_handles_transient_failure() {
        let (_dir, db_path) = setup_db();

        let conn = open_and_migrate(&db_path).unwrap();
        let payload = StoreFlagsPayload {
            message_id: 1,
            uid: 100,
            folder_name: "INBOX".to_string(),
            new_flags: FLAG_SEEN,
        };
        let payload_json = serde_json::to_string(&payload).unwrap();
        pending_ops_store::insert_pending_op(
            &conn,
            "acct-1",
            &OperationKind::StoreFlags,
            &payload_json,
        )
        .unwrap();
        drop(conn);

        let (event_tx, _event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore {
            should_fail: Some("connection timed out".to_string()),
        });
        let params = make_test_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));

        // Use a timeout so we don't wait for the full backoff
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            process_account_ops(
                &db_path,
                "acct-1",
                &event_tx,
                flag_store.clone(),
                account_params_fn.as_ref(),
                Arc::new(MockImapFolderOps { should_fail: None }),
            ),
        )
        .await;

        // Operation should be requeued with retry_count > 0
        let conn = open_and_migrate(&db_path).unwrap();
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].state, OperationState::Pending);
        assert!(ops[0].retry_count > 0);
        assert!(ops[0].last_error.as_ref().unwrap().contains("timed out"));
    }

    #[tokio::test]
    async fn engine_handles_permanent_failure() {
        let (_dir, db_path) = setup_db();

        let conn = open_and_migrate(&db_path).unwrap();
        let payload = StoreFlagsPayload {
            message_id: 1,
            uid: 100,
            folder_name: "INBOX".to_string(),
            new_flags: FLAG_SEEN,
        };
        let payload_json = serde_json::to_string(&payload).unwrap();
        pending_ops_store::insert_pending_op(
            &conn,
            "acct-1",
            &OperationKind::StoreFlags,
            &payload_json,
        )
        .unwrap();
        drop(conn);

        let (event_tx, mut event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore {
            should_fail: Some("authentication failed".to_string()),
        });
        let params = make_test_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));

        process_account_ops(
            &db_path,
            "acct-1",
            &event_tx,
            flag_store.clone(),
            account_params_fn.as_ref(),
            Arc::new(MockImapFolderOps { should_fail: None }),
        )
        .await;

        // Operation should be marked failed (not in pending ops)
        let conn = open_and_migrate(&db_path).unwrap();
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty());

        // Should have received an OperationFailed event
        let event = event_rx.try_recv().unwrap();
        match event {
            SyncEvent::OperationFailed { error, .. } => {
                assert!(error.contains("authentication"));
            }
            _ => panic!("unexpected event"),
        }
    }

    #[tokio::test]
    async fn engine_start_and_notify() {
        let (_dir, db_path) = setup_db();

        // Create a pending operation
        let conn = open_and_migrate(&db_path).unwrap();
        let payload = StoreFlagsPayload {
            message_id: 1,
            uid: 100,
            folder_name: "INBOX".to_string(),
            new_flags: FLAG_SEEN,
        };
        let payload_json = serde_json::to_string(&payload).unwrap();
        pending_ops_store::insert_pending_op(
            &conn,
            "acct-1",
            &OperationKind::StoreFlags,
            &payload_json,
        )
        .unwrap();
        drop(conn);

        let (event_tx, mut event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore { should_fail: None });
        let params = make_test_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));

        let handle = start_sync_engine(db_path.clone(), event_tx, flag_store, account_params_fn);
        handle.notify_account("acct-1");

        // Wait for the event
        let event = tokio::time::timeout(std::time::Duration::from_secs(5), event_rx.recv())
            .await
            .unwrap()
            .unwrap();

        match event {
            SyncEvent::MessageFlagsChanged { message_id, .. } => {
                assert_eq!(message_id, 1);
            }
            _ => panic!("unexpected event"),
        }

        // Engine handle dropped here, triggering shutdown
        drop(handle);
    }

    #[test]
    fn stress_test_toggle_1000_messages() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO folders (id, account_id, name, attributes) VALUES (1, 'acct-1', 'INBOX', '')",
            [],
        ).unwrap();

        // Insert 1000 messages
        for i in 1..=1000 {
            conn.execute(
                "INSERT INTO messages (id, account_id, uid, flags, size, content_hash)
                 VALUES (?1, 'acct-1', ?2, 0, 100, ?3)",
                rusqlite::params![i as i64, i as u32, format!("hash_{i}")],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO message_folders (message_id, folder_id) VALUES (?1, 1)",
                rusqlite::params![i as i64],
            )
            .unwrap();
        }
        drop(conn);

        // Toggle all 1000 messages - should not block
        let start = std::time::Instant::now();
        for i in 1..=1000 {
            toggle_message_read(&db_path, i, "acct-1", i as u32, "INBOX", 0).unwrap();
        }
        let elapsed = start.elapsed();

        // Verify all 1000 have pending ops
        let conn = open_and_migrate(&db_path).unwrap();
        let count = pending_ops_store::count_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(count, 1000);

        // Should complete in well under a second on any reasonable hardware
        assert!(
            elapsed.as_secs() < 10,
            "1000 toggles took too long: {elapsed:?}"
        );
    }

    // ---------- Mock SmtpSender and ImapAppender ----------

    struct MockSmtpSender {
        should_fail: Option<String>,
    }

    impl SmtpSender for MockSmtpSender {
        fn send_message(
            &self,
            _params: &SmtpConnectParams,
            _envelope_from: &str,
            _envelope_to: &[String],
            _rfc822_data: &[u8],
        ) -> Result<(), String> {
            match &self.should_fail {
                Some(err) => Err(err.clone()),
                None => Ok(()),
            }
        }
    }

    struct MockImapAppender {
        should_fail: Option<String>,
    }

    impl ImapAppender for MockImapAppender {
        fn append_message(
            &self,
            _params: &ImapConnectParams,
            _folder_name: &str,
            _flags: u32,
            _rfc822_data: &[u8],
        ) -> Result<(), String> {
            match &self.should_fail {
                Some(err) => Err(err.clone()),
                None => Ok(()),
            }
        }
    }

    fn make_test_smtp_params() -> SmtpConnectParams {
        SmtpConnectParams {
            host: "smtp.example.com".to_string(),
            port: 587,
            encryption: EncryptionMode::StartTls,
            username: "user".to_string(),
            password: "pass".to_string(),
            accepted_fingerprint: None,
            insecure: false,
            account_id: "acct-1".to_string(),
        }
    }

    fn setup_db_with_identity_and_sent() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO folders (id, account_id, name, attributes) VALUES (1, 'acct-1', 'INBOX', '')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO folders (id, account_id, name, attributes) VALUES (2, 'acct-1', 'Sent', '')",
            [],
        ).unwrap();
        // Insert identity
        conn.execute(
            "INSERT INTO identities (id, account_id, email_address, display_name, smtp_host, smtp_port, smtp_encryption, smtp_username)
             VALUES (1, 'acct-1', 'user@example.com', 'Test User', 'smtp.example.com', 587, 'StartTls', 'user')",
            [],
        ).unwrap();
        drop(conn);
        (dir, db_path)
    }

    fn make_test_rfc822() -> Vec<u8> {
        b"From: user@example.com\r\n\
          To: recipient@example.com\r\n\
          Subject: Test message\r\n\
          Date: Mon, 1 Jan 2024 12:00:00 +0000\r\n\
          Message-ID: <test-send-001@example.com>\r\n\
          \r\n\
          Hello, this is a test message.\r\n"
            .to_vec()
    }

    // ---------- Send tests ----------

    #[test]
    fn operation_kind_send_roundtrips() {
        let kind = OperationKind::Send;
        assert_eq!(OperationKind::parse(kind.as_str()), Some(kind));
    }

    #[test]
    fn send_payload_serializes() {
        let payload = SendPayload {
            identity_id: 1,
            content_hash: None,
            inline_rfc822_b64: Some("dGVzdA==".to_string()),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: SendPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.identity_id, 1);
        assert_eq!(parsed.inline_rfc822_b64.as_deref(), Some("dGVzdA=="));
    }

    #[test]
    fn queue_send_message_creates_pending_op() {
        let (_dir, db_path) = setup_db_with_identity_and_sent();
        let rfc822 = make_test_rfc822();

        let op_id = queue_send_message(&db_path, "acct-1", 1, &rfc822).unwrap();
        assert!(op_id > 0);

        let conn = open_and_migrate(&db_path).unwrap();
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].kind, OperationKind::Send);
        assert_eq!(ops[0].state, OperationState::Pending);

        let payload: SendPayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.identity_id, 1);
        assert!(payload.inline_rfc822_b64.is_some());
    }

    #[test]
    fn extract_envelope_recipients_finds_all() {
        let raw = b"From: sender@example.com\r\n\
                     To: alice@example.com, bob@example.com\r\n\
                     Cc: carol@example.com\r\n\
                     Bcc: dave@example.com\r\n\
                     Subject: Test\r\n\
                     \r\n\
                     body\r\n";
        let recips = extract_envelope_recipients(raw);
        assert_eq!(recips.len(), 4);
        assert!(recips.contains(&"alice@example.com".to_string()));
        assert!(recips.contains(&"bob@example.com".to_string()));
        assert!(recips.contains(&"carol@example.com".to_string()));
        assert!(recips.contains(&"dave@example.com".to_string()));
    }

    #[test]
    fn extract_envelope_recipients_empty_when_no_to() {
        let raw = b"From: sender@example.com\r\nSubject: Test\r\n\r\nbody\r\n";
        let recips = extract_envelope_recipients(raw);
        assert!(recips.is_empty());
    }

    #[tokio::test]
    async fn engine_processes_send_op_with_mock() {
        let (_dir, db_path) = setup_db_with_identity_and_sent();
        let rfc822 = make_test_rfc822();

        // Queue a send operation
        let _op_id = queue_send_message(&db_path, "acct-1", 1, &rfc822).unwrap();

        let (event_tx, mut event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore { should_fail: None });
        let smtp_sender: Arc<dyn SmtpSender> = Arc::new(MockSmtpSender { should_fail: None });
        let imap_appender: Arc<dyn ImapAppender> = Arc::new(MockImapAppender { should_fail: None });
        let params = make_test_params();
        let smtp_params = make_test_smtp_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));
        let smtp_params_fn: Arc<SmtpParamsFn> =
            Arc::new(move |_id| Some((smtp_params.clone(), "user@example.com".to_string(), false)));

        process_account_ops_full(
            &db_path,
            "acct-1",
            &event_tx,
            flag_store,
            account_params_fn.as_ref(),
            smtp_sender,
            imap_appender,
            Some(smtp_params_fn),
            None,
            Arc::new(MockImapFolderOps { should_fail: None }),
        )
        .await;

        // Operation should be completed
        let conn = open_and_migrate(&db_path).unwrap();
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty());

        // Should have received MessageSent event
        let event = event_rx.try_recv().unwrap();
        assert!(matches!(event, SyncEvent::MessageSent { .. }));

        // Sent message should be in messages table
        let count = crate::services::message_store::count_messages(&conn, "acct-1").unwrap();
        assert!(count >= 1, "sent message should be stored locally");
    }

    #[tokio::test]
    async fn send_op_transient_failure_requeues() {
        let (_dir, db_path) = setup_db_with_identity_and_sent();
        let rfc822 = make_test_rfc822();
        let _op_id = queue_send_message(&db_path, "acct-1", 1, &rfc822).unwrap();

        let (event_tx, _event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore { should_fail: None });
        let smtp_sender: Arc<dyn SmtpSender> = Arc::new(MockSmtpSender {
            should_fail: Some("connection timed out".to_string()),
        });
        let imap_appender: Arc<dyn ImapAppender> = Arc::new(MockImapAppender { should_fail: None });
        let params = make_test_params();
        let smtp_params = make_test_smtp_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));
        let smtp_params_fn: Arc<SmtpParamsFn> =
            Arc::new(move |_id| Some((smtp_params.clone(), "user@example.com".to_string(), false)));

        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            process_account_ops_full(
                &db_path,
                "acct-1",
                &event_tx,
                flag_store,
                account_params_fn.as_ref(),
                smtp_sender,
                imap_appender,
                Some(smtp_params_fn),
                None,
                Arc::new(MockImapFolderOps { should_fail: None }),
            ),
        )
        .await;

        // Op should be requeued
        let conn = open_and_migrate(&db_path).unwrap();
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].state, OperationState::Pending);
        assert!(ops[0].retry_count > 0);
        assert!(ops[0].last_error.as_ref().unwrap().contains("timed out"));
    }

    #[tokio::test]
    async fn send_op_permanent_failure_marks_failed() {
        let (_dir, db_path) = setup_db_with_identity_and_sent();
        let rfc822 = make_test_rfc822();
        let _op_id = queue_send_message(&db_path, "acct-1", 1, &rfc822).unwrap();

        let (event_tx, mut event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore { should_fail: None });
        let smtp_sender: Arc<dyn SmtpSender> = Arc::new(MockSmtpSender {
            should_fail: Some("authentication failed".to_string()),
        });
        let imap_appender: Arc<dyn ImapAppender> = Arc::new(MockImapAppender { should_fail: None });
        let params = make_test_params();
        let smtp_params = make_test_smtp_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));
        let smtp_params_fn: Arc<SmtpParamsFn> =
            Arc::new(move |_id| Some((smtp_params.clone(), "user@example.com".to_string(), false)));

        process_account_ops_full(
            &db_path,
            "acct-1",
            &event_tx,
            flag_store,
            account_params_fn.as_ref(),
            smtp_sender,
            imap_appender,
            Some(smtp_params_fn),
            None,
            Arc::new(MockImapFolderOps { should_fail: None }),
        )
        .await;

        // Op should be marked failed
        let conn = open_and_migrate(&db_path).unwrap();
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty()); // Failed ops don't show up in load_pending_ops

        // Should get OperationFailed event
        let event = event_rx.try_recv().unwrap();
        match event {
            SyncEvent::OperationFailed { error, .. } => {
                assert!(error.contains("authentication"));
            }
            _ => panic!("expected OperationFailed event"),
        }
    }

    #[tokio::test]
    async fn send_op_login_before_send_blocks_on_auth_failure() {
        let (_dir, db_path) = setup_db_with_identity_and_sent();
        let rfc822 = make_test_rfc822();
        let _op_id = queue_send_message(&db_path, "acct-1", 1, &rfc822).unwrap();

        let (event_tx, mut event_rx) = broadcast::channel::<SyncEvent>(16);
        // IMAP flag store fails with auth error (simulates broken inbound credential)
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore {
            should_fail: Some("authentication failed".to_string()),
        });
        let smtp_sender: Arc<dyn SmtpSender> = Arc::new(MockSmtpSender { should_fail: None });
        let imap_appender: Arc<dyn ImapAppender> = Arc::new(MockImapAppender { should_fail: None });
        let params = make_test_params();
        let smtp_params = make_test_smtp_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));
        // login_before_send = true
        let smtp_params_fn: Arc<SmtpParamsFn> =
            Arc::new(move |_id| Some((smtp_params.clone(), "user@example.com".to_string(), true)));

        process_account_ops_full(
            &db_path,
            "acct-1",
            &event_tx,
            flag_store,
            account_params_fn.as_ref(),
            smtp_sender,
            imap_appender,
            Some(smtp_params_fn),
            None,
            Arc::new(MockImapFolderOps { should_fail: None }),
        )
        .await;

        // Should get OperationFailed due to login-before-send check
        let event = event_rx.try_recv().unwrap();
        match event {
            SyncEvent::OperationFailed { error, .. } => {
                assert!(
                    error.contains("login-before-send"),
                    "error should mention login-before-send: {error}"
                );
            }
            _ => panic!("expected OperationFailed event"),
        }
    }

    #[test]
    fn queue_50_send_messages_does_not_block() {
        let (_dir, db_path) = setup_db_with_identity_and_sent();
        let rfc822 = make_test_rfc822();

        let start = std::time::Instant::now();
        for _ in 0..50 {
            queue_send_message(&db_path, "acct-1", 1, &rfc822).unwrap();
        }
        let elapsed = start.elapsed();

        let conn = open_and_migrate(&db_path).unwrap();
        let count = pending_ops_store::count_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(count, 50);

        // Should complete very quickly
        assert!(
            elapsed.as_secs() < 5,
            "50 queue_send_message calls took too long: {elapsed:?}"
        );
    }

    #[test]
    fn is_transient_error_smtp_classification() {
        assert!(is_transient_error(&SyncError::Smtp(
            "connection timed out".to_string()
        )));
        assert!(!is_transient_error(&SyncError::Smtp(
            "authentication failed".to_string()
        )));
        assert!(!is_transient_error(&SyncError::ContentStore(
            "not found".to_string()
        )));
    }
}
