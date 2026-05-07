use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::provider::MaxTlsVersion;

fn default_true() -> bool {
    true
}

/// Supported mail protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Imap,
    Pop3,
}

/// Connection encryption mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionMode {
    None,
    SslTls,
    StartTls,
}

/// Authentication method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    Plain,
    Login,
    OAuth2,
    /// Client certificate authentication via SASL EXTERNAL mechanism.
    /// When selected, the client presents a TLS client certificate and
    /// no password-based mechanism is attempted.
    Certificate,
}

/// Actions that can be assigned to swipe-left or swipe-right gestures (FR-37).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwipeAction {
    /// No action configured.
    #[default]
    None,
    /// Archive the message.
    Archive,
    /// Delete / move to trash.
    Delete,
    /// Mark the message as read.
    MarkRead,
    /// Mark the message as unread.
    MarkUnread,
    /// Move to a specific folder (folder name stored in the variant).
    MoveToFolder(String),
}

impl std::fmt::Display for SwipeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Archive => write!(f, "Archive"),
            Self::Delete => write!(f, "Delete"),
            Self::MarkRead => write!(f, "Mark as read"),
            Self::MarkUnread => write!(f, "Mark as unread"),
            Self::MoveToFolder(name) => write!(f, "Move to {name}"),
        }
    }
}

/// Per-account swipe and move defaults (FR-37, FR-38, US-37).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwipeDefaults {
    /// Action for swipe-left gesture (FR-37).
    pub swipe_left: SwipeAction,
    /// Action for swipe-right gesture (FR-37).
    pub swipe_right: SwipeAction,
    /// Default "move-to" target folder name (FR-38).
    pub default_move_to: Option<String>,
}

/// System folder roles that can be assigned to server folders (FR-35, US-36).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FolderRole {
    Drafts,
    Sent,
    Archive,
    Trash,
    Junk,
}

impl FolderRole {
    /// Returns all defined folder roles.
    pub fn all() -> &'static [FolderRole] {
        &[
            FolderRole::Drafts,
            FolderRole::Sent,
            FolderRole::Archive,
            FolderRole::Trash,
            FolderRole::Junk,
        ]
    }
}

impl std::fmt::Display for FolderRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Drafts => write!(f, "Drafts"),
            Self::Sent => write!(f, "Sent"),
            Self::Archive => write!(f, "Archive"),
            Self::Trash => write!(f, "Trash"),
            Self::Junk => write!(f, "Junk"),
        }
    }
}

/// Per-account mapping of system folder roles to server folder names (FR-35, FR-36, US-36).
/// Each field is `None` when not assigned (or not auto-detected).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemFolders {
    pub drafts: Option<String>,
    pub sent: Option<String>,
    pub archive: Option<String>,
    pub trash: Option<String>,
    pub junk: Option<String>,
}

impl SystemFolders {
    /// Get the folder name assigned to a given role.
    pub fn get(&self, role: FolderRole) -> Option<&str> {
        match role {
            FolderRole::Drafts => self.drafts.as_deref(),
            FolderRole::Sent => self.sent.as_deref(),
            FolderRole::Archive => self.archive.as_deref(),
            FolderRole::Trash => self.trash.as_deref(),
            FolderRole::Junk => self.junk.as_deref(),
        }
    }

    /// Set the folder name for a given role.
    pub fn set(&mut self, role: FolderRole, folder_name: Option<String>) {
        match role {
            FolderRole::Drafts => self.drafts = folder_name,
            FolderRole::Sent => self.sent = folder_name,
            FolderRole::Archive => self.archive = folder_name,
            FolderRole::Trash => self.trash = folder_name,
            FolderRole::Junk => self.junk = folder_name,
        }
    }

    /// Returns true if all roles are unassigned.
    pub fn is_empty(&self) -> bool {
        self.drafts.is_none()
            && self.sent.is_none()
            && self.archive.is_none()
            && self.trash.is_none()
            && self.junk.is_none()
    }
}

/// Auto-detect system folder assignments from IMAP SPECIAL-USE metadata (FR-36).
/// Takes a list of `(folder_name, special_use_attribute)` pairs as reported by the server.
/// Known SPECIAL-USE attributes: `\Drafts`, `\Sent`, `\Archive`, `\Trash`, `\Junk`.
pub fn detect_system_folders(folder_attributes: &[(String, String)]) -> SystemFolders {
    let mut result = SystemFolders::default();
    for (name, attr) in folder_attributes {
        match attr.as_str() {
            "\\Drafts" => result.drafts = Some(name.clone()),
            "\\Sent" => result.sent = Some(name.clone()),
            "\\Archive" => result.archive = Some(name.clone()),
            "\\Trash" => result.trash = Some(name.clone()),
            "\\Junk" => result.junk = Some(name.clone()),
            _ => {}
        }
    }
    result
}

/// Mailbox quota information as reported by the server (FR-42, FR-43, AC-17).
/// Stores usage and limit in bytes. When the server does not report quota,
/// the account's `quota` field is `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaInfo {
    /// Storage currently used, in bytes.
    pub used_bytes: u64,
    /// Storage limit (quota cap), in bytes.
    pub limit_bytes: u64,
}

/// The usage percentage at or above which a high-usage warning is shown (FR-43).
pub const QUOTA_HIGH_THRESHOLD_PERCENT: f64 = 95.0;

impl QuotaInfo {
    /// Create a new `QuotaInfo`. `limit_bytes` must be > 0.
    pub fn new(used_bytes: u64, limit_bytes: u64) -> Option<Self> {
        if limit_bytes == 0 {
            return None;
        }
        Some(Self {
            used_bytes,
            limit_bytes,
        })
    }

    /// Returns usage as a percentage (0.0–100.0+).
    pub fn usage_percent(&self) -> f64 {
        (self.used_bytes as f64 / self.limit_bytes as f64) * 100.0
    }

    /// Returns `true` when usage is at or above the high threshold (FR-43).
    pub fn is_high_usage(&self) -> bool {
        self.usage_percent() >= QUOTA_HIGH_THRESHOLD_PERCENT
    }

    /// Format bytes as a human-readable string (e.g. "1.23 GB").
    pub fn format_bytes(bytes: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
        let b = bytes as f64;
        if b >= GB {
            format!("{:.2} GB", b / GB)
        } else if b >= MB {
            format!("{:.1} MB", b / MB)
        } else if b >= KB {
            format!("{:.0} KB", b / KB)
        } else {
            format!("{bytes} B")
        }
    }
}

/// Advanced connection security settings per account (FR-4, FR-53, US-8, US-9, US-10).
/// All fields are optional and default to their "off" / unset state so that
/// accounts created before this feature was added deserialize cleanly.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Require DNSSEC validation for DNS lookups to this account's servers (US-8).
    #[serde(default)]
    pub dnssec: bool,
    /// Require DANE (TLSA) verification for TLS connections (US-8).
    #[serde(default)]
    pub dane: bool,
    /// Allow insecure (unverified-certificate) connections for this account (US-9).
    #[serde(default)]
    pub insecure: bool,
    /// Allow sending passwords via PLAIN/LOGIN over unencrypted connections (FR-30/FR-31).
    /// Defaults to `false` (secure by default). When `true`, the insecure-connection
    /// protection is bypassed for this account.
    #[serde(default)]
    pub allow_insecure_auth: bool,
    /// SHA-256 fingerprint of the server certificate to pin (US-9).
    /// When set, the client must reject any certificate whose fingerprint
    /// does not match, regardless of CA trust.
    #[serde(default)]
    pub certificate_fingerprint: Option<String>,
    /// Path or alias referencing a client certificate for mutual TLS (US-10).
    /// On Linux this typically points into an NSS or PKCS#11 store.
    #[serde(default)]
    pub client_certificate: Option<String>,
    /// Authentication realm override (FR-4).
    #[serde(default)]
    pub auth_realm: Option<String>,
    /// Maximum TLS version allowed for connections to this account's servers (FR-28).
    /// `None` means no restriction (use highest available).
    #[serde(default)]
    pub max_tls_version: Option<MaxTlsVersion>,
    /// Disable IP-address-based connections for this account (FR-29).
    /// When true, connections must use hostnames, not raw IP addresses.
    #[serde(default)]
    pub disable_ip_connections: bool,
}

impl SecuritySettings {
    /// Clear the client certificate reference, returning `true` if a certificate
    /// was previously set (US-8-clear).
    pub fn clear_client_certificate(&mut self) -> bool {
        self.client_certificate.take().is_some()
    }

    /// Returns `true` when all fields are at their default (off/unset) values,
    /// meaning these settings carry no information and can be replaced with `None`.
    pub fn is_empty(&self) -> bool {
        !self.dnssec
            && !self.dane
            && !self.insecure
            && !self.allow_insecure_auth
            && self.certificate_fingerprint.is_none()
            && self.client_certificate.is_none()
            && self.auth_realm.is_none()
    }
}

/// Preference for which date source to use when displaying message dates (FR-51).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateHeaderPreference {
    /// Use the date/time reported by the server (default).
    #[default]
    ServerTime,
    /// Use the `Date` header from the message itself.
    DateHeader,
    /// Use the `Received` header timestamp.
    ReceivedHeader,
}

impl std::fmt::Display for DateHeaderPreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServerTime => write!(f, "Server time"),
            Self::DateHeader => write!(f, "Date header"),
            Self::ReceivedHeader => write!(f, "Received header"),
        }
    }
}

/// Policy that governs how message bodies are downloaded during sync (US-10, FR-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DownloadPolicy {
    /// Download the full message (envelope + body + attachments).
    #[default]
    Full,
    /// Download only the envelope / headers; body fetched on user request.
    HeadersOnly,
    /// No automatic download at all; everything fetched on explicit user action.
    OnDemand,
}

/// Advanced fetch settings per account (FR-51, FR-53).
/// All fields default to their "off" / unset state so that
/// accounts created before this feature was added deserialize cleanly.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchSettings {
    /// Enable partial (body structure) fetch mode for large messages (FR-51).
    #[serde(default)]
    pub partial_fetch: bool,
    /// Fetch raw message data instead of parsed MIME (FR-51).
    #[serde(default)]
    pub raw_fetch: bool,
    /// Ignore server-reported size limits when fetching (FR-51).
    #[serde(default)]
    pub ignore_size_limits: bool,
    /// Which date source to prefer when displaying message timestamps (FR-51).
    #[serde(default)]
    pub date_header_preference: DateHeaderPreference,
    /// Enable UTF-8 (IMAP UTF8=ACCEPT) support for this account (FR-51).
    #[serde(default)]
    pub utf8_support: bool,
    /// Body download policy for new messages during sync (US-10, FR-6).
    #[serde(default)]
    pub download_policy: DownloadPolicy,
}

/// Advanced keep-alive settings per account (FR-52, FR-53).
/// The polling interval itself is stored as a top-level account field
/// (shared with story 10); this struct holds additional keep-alive tuning.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeepAliveSettings {
    /// Send NOOP commands instead of using IMAP IDLE for keep-alive (FR-52).
    /// Useful for servers that do not support or misbehave with IDLE.
    #[serde(default)]
    pub use_noop_instead_of_idle: bool,
}

/// POP3-specific behaviour settings (US-31, US-32, US-33, US-34, FR-9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pop3Settings {
    /// When enabled, downloaded messages remain on the server (US-31, AC-14).
    pub leave_on_server: bool,
    /// When enabled, deleting a message on the device also deletes it from the server (US-32).
    pub delete_from_server_when_deleted_on_device: bool,
    /// When enabled, messages deleted from the server are kept locally on the device (US-33).
    pub keep_on_device_when_deleted_from_server: bool,
    /// Optional cap on the number of messages to download per sync (US-34).
    /// `None` means unlimited.
    pub max_messages_to_download: Option<u32>,
    /// When enabled, use APOP authentication if the server greeting contains
    /// a valid timestamp (FR-23, FR-24, US-13). Disabled by default because
    /// APOP relies on MD5 and causes unnecessary negotiation attempts on
    /// servers that handle it poorly (Design Note N-3).
    #[serde(default)]
    pub apop_enabled: bool,
}

impl Default for Pop3Settings {
    fn default() -> Self {
        Self {
            leave_on_server: true,
            delete_from_server_when_deleted_on_device: false,
            keep_on_device_when_deleted_from_server: true,
            max_messages_to_download: None,
            apop_enabled: false,
        }
    }
}

/// An RGB colour associated with an account (FR-5, FR-12).
/// Components are in the 0.0–1.0 range, matching `gdk::RGBA` conventions so the
/// UI layer can convert without loss.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AccountColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl AccountColor {
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }

    /// Convert to a CSS hex colour string (e.g. `#ff8800`).
    pub fn to_hex(&self) -> String {
        let r = (self.red.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (self.green.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (self.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}

/// Resolve the effective colour when multiple colour levels exist (FR-15).
/// Precedence: identity > folder > account.  The most specific non-`None`
/// value wins; returns `None` if all levels are unset.
pub fn resolve_color(
    account_color: Option<AccountColor>,
    folder_color: Option<AccountColor>,
    identity_color: Option<AccountColor>,
) -> Option<AccountColor> {
    identity_color.or(folder_color).or(account_color)
}

/// SMTP (outgoing) server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub encryption: EncryptionMode,
    pub auth_method: AuthMethod,
    pub username: String,
    pub credential: String,
}

/// Parameters for creating a new account (avoids too-many-arguments).
pub struct NewAccountParams {
    pub display_name: String,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub encryption: EncryptionMode,
    pub auth_method: AuthMethod,
    pub username: String,
    pub credential: String,
    pub smtp: Option<SmtpConfig>,
    /// POP3-specific settings. Should be `Some` for POP3 accounts, `None` for IMAP.
    pub pop3_settings: Option<Pop3Settings>,
    /// Optional account colour (FR-5, FR-12).
    pub color: Option<AccountColor>,
    /// Optional avatar image path (FR-5, FR-13).
    pub avatar_path: Option<String>,
    /// Optional category label for organizing accounts (FR-17, US-17).
    pub category: Option<String>,
    /// Whether synchronization is enabled for this account (FR-25, US-24).
    pub sync_enabled: bool,
    /// Whether this account syncs only on explicit user request (FR-6, US-27, AC-12).
    pub on_demand: bool,
    /// Polling / keep-alive interval in minutes (FR-6, US-28).
    /// `None` means use the application default.
    pub polling_interval_minutes: Option<u32>,
    /// Suppress sync on metered (cellular/tethered) connections (FR-7, US-29).
    pub unmetered_only: bool,
    /// Suppress sync when no VPN tunnel is active (FR-7, US-29, AC-13).
    pub vpn_only: bool,
    /// Exempt this account from the global sync schedule (FR-7, US-30).
    pub schedule_exempt: bool,
    /// IMAP system folder designations (FR-35, FR-36, US-36).
    /// Should be `None` for POP3 accounts.
    pub system_folders: Option<SystemFolders>,
    /// Per-account swipe and move defaults (FR-37, FR-38, US-37).
    pub swipe_defaults: Option<SwipeDefaults>,
    /// Whether notifications are enabled for this account (FR-39, AC-19).
    pub notifications_enabled: bool,
    /// Advanced connection security settings (FR-4, FR-53, US-8, US-9, US-10).
    pub security_settings: Option<SecuritySettings>,
    /// Advanced fetch settings (FR-51, FR-53).
    pub fetch_settings: Option<FetchSettings>,
    /// Advanced keep-alive settings (FR-52, FR-53).
    pub keep_alive_settings: Option<KeepAliveSettings>,
    /// OAuth tenant identifier for multi-tenant providers (FR-10, US-4).
    /// Stored so that re-authorization uses the same tenant.
    pub oauth_tenant: Option<String>,
    /// Shared mailbox address for providers that support delegation (FR-40, N-8).
    /// When set, the application encodes the username as `shared@domain\user@domain`
    /// so the user authenticates with their own credentials but accesses the shared mailbox.
    pub shared_mailbox: Option<String>,
}

/// Parameters for updating an existing account. Same fields as creation
/// (the unique identifier is preserved automatically).
pub struct UpdateAccountParams {
    pub display_name: String,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub encryption: EncryptionMode,
    pub auth_method: AuthMethod,
    pub username: String,
    pub credential: String,
    pub smtp: Option<SmtpConfig>,
    /// POP3-specific settings. Should be `Some` for POP3 accounts, `None` for IMAP.
    pub pop3_settings: Option<Pop3Settings>,
    /// Optional account colour (FR-5, FR-12).
    pub color: Option<AccountColor>,
    /// Optional avatar image path (FR-5, FR-13).
    pub avatar_path: Option<String>,
    /// Optional category label for organizing accounts (FR-17, US-17).
    pub category: Option<String>,
    /// Whether synchronization is enabled for this account (FR-25, US-24).
    pub sync_enabled: bool,
    /// Whether this account syncs only on explicit user request (FR-6, US-27, AC-12).
    pub on_demand: bool,
    /// Polling / keep-alive interval in minutes (FR-6, US-28).
    /// `None` means use the application default.
    pub polling_interval_minutes: Option<u32>,
    /// Suppress sync on metered (cellular/tethered) connections (FR-7, US-29).
    pub unmetered_only: bool,
    /// Suppress sync when no VPN tunnel is active (FR-7, US-29, AC-13).
    pub vpn_only: bool,
    /// Exempt this account from the global sync schedule (FR-7, US-30).
    pub schedule_exempt: bool,
    /// IMAP system folder designations (FR-35, FR-36, US-36).
    /// Should be `None` for POP3 accounts.
    pub system_folders: Option<SystemFolders>,
    /// Per-account swipe and move defaults (FR-37, FR-38, US-37).
    pub swipe_defaults: Option<SwipeDefaults>,
    /// Whether notifications are enabled for this account (FR-39, AC-19).
    pub notifications_enabled: bool,
    /// Advanced connection security settings (FR-4, FR-53, US-8, US-9, US-10).
    pub security_settings: Option<SecuritySettings>,
    /// Advanced fetch settings (FR-51, FR-53).
    pub fetch_settings: Option<FetchSettings>,
    /// Advanced keep-alive settings (FR-52, FR-53).
    pub keep_alive_settings: Option<KeepAliveSettings>,
    /// OAuth tenant identifier for multi-tenant providers (FR-10, US-4).
    pub oauth_tenant: Option<String>,
    /// Shared mailbox address for providers that support delegation (FR-40, N-8).
    pub shared_mailbox: Option<String>,
}

/// A mail account with connection settings and a stable unique identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    id: Uuid,
    display_name: String,
    protocol: Protocol,
    host: String,
    port: u16,
    encryption: EncryptionMode,
    auth_method: AuthMethod,
    username: String,
    /// Password or OAuth token, depending on `auth_method`.
    credential: String,
    /// Optional SMTP (outgoing) server configuration.
    smtp: Option<SmtpConfig>,
    /// POP3-specific settings (only meaningful for POP3 accounts).
    #[serde(default)]
    pop3_settings: Option<Pop3Settings>,
    /// Optional account colour (FR-5, FR-12).
    #[serde(default)]
    color: Option<AccountColor>,
    /// Optional avatar image path (FR-5, FR-13).
    #[serde(default)]
    avatar_path: Option<String>,
    /// Optional category label for organizing accounts (FR-17, US-17).
    #[serde(default)]
    category: Option<String>,
    /// Whether synchronization is enabled (FR-25, US-24). Defaults to true.
    #[serde(default = "default_true")]
    sync_enabled: bool,
    /// Whether this account syncs only on explicit user request (FR-6, US-27, AC-12).
    #[serde(default)]
    on_demand: bool,
    /// Polling / keep-alive interval in minutes (FR-6, US-28).
    /// `None` means use the application default.
    #[serde(default)]
    polling_interval_minutes: Option<u32>,
    /// Suppress sync on metered (cellular/tethered) connections (FR-7, US-29).
    #[serde(default)]
    unmetered_only: bool,
    /// Suppress sync when no VPN tunnel is active (FR-7, US-29, AC-13).
    #[serde(default)]
    vpn_only: bool,
    /// Exempt this account from the global sync schedule (FR-7, US-30).
    #[serde(default)]
    schedule_exempt: bool,
    /// Whether this is the primary account (FR-24, FR-26, FR-27).
    #[serde(default)]
    is_primary: bool,
    /// Active error or warning message for this account (e.g. sync failure).
    /// Cleared automatically when synchronization is disabled (FR-32, AC-11).
    #[serde(default)]
    error_state: Option<String>,
    /// IMAP system folder designations (FR-35, FR-36, US-36).
    /// `None` for POP3 accounts or when not yet configured.
    #[serde(default)]
    system_folders: Option<SystemFolders>,
    /// Per-account swipe and move defaults (FR-37, FR-38, US-37).
    #[serde(default)]
    swipe_defaults: Option<SwipeDefaults>,
    /// Whether notifications are enabled for this account (FR-39, AC-19).
    /// Independent of sync enablement — toggling sync does not affect this.
    #[serde(default = "default_true")]
    notifications_enabled: bool,
    /// Mailbox quota reported by the server (FR-42, FR-43, AC-17).
    /// `None` when the server does not report quota information.
    #[serde(default)]
    quota: Option<QuotaInfo>,
    /// Advanced connection security settings (FR-4, FR-53, US-8, US-9, US-10).
    #[serde(default)]
    security_settings: Option<SecuritySettings>,
    /// Advanced fetch settings (FR-51, FR-53).
    #[serde(default)]
    fetch_settings: Option<FetchSettings>,
    /// Advanced keep-alive settings (FR-52, FR-53).
    #[serde(default)]
    keep_alive_settings: Option<KeepAliveSettings>,
    /// Flag indicating that POP3 "leave on server" setting changed and
    /// the local message store should be re-evaluated at next sync (FR-55).
    #[serde(default)]
    pop3_needs_store_reevaluation: bool,
    /// OAuth tenant identifier for multi-tenant providers like Microsoft (FR-10, US-4).
    /// Stored with the account so re-authorization uses the same tenant.
    #[serde(default)]
    oauth_tenant: Option<String>,
    /// Shared mailbox address for providers that support delegation (FR-40, N-8).
    /// When set, the effective username for IMAP/SMTP is encoded as
    /// `shared@domain\user@domain` so the user's own credentials access the shared mailbox.
    #[serde(default)]
    shared_mailbox: Option<String>,
}

/// A local folder associated with an account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    name: String,
    local_only: bool,
}

impl Folder {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_local_only(&self) -> bool {
        self.local_only
    }
}

impl Protocol {
    /// Returns the default folder set for this protocol.
    /// POP3 accounts have a fixed set of local-only folders (FR-10).
    /// IMAP accounts get a minimal default set; server-side folders are
    /// discovered at sync time.
    pub fn default_folders(&self) -> Vec<Folder> {
        match self {
            Protocol::Pop3 => vec![
                Folder {
                    name: "Inbox".into(),
                    local_only: true,
                },
                Folder {
                    name: "Drafts".into(),
                    local_only: true,
                },
                Folder {
                    name: "Sent".into(),
                    local_only: true,
                },
                Folder {
                    name: "Trash".into(),
                    local_only: true,
                },
            ],
            Protocol::Imap => vec![Folder {
                name: "Inbox".into(),
                local_only: false,
            }],
        }
    }

    /// Returns a list of limitation descriptions for this protocol.
    /// POP3 has significant limitations compared to IMAP (US-35).
    pub fn limitations(&self) -> Vec<&'static str> {
        match self {
            Protocol::Pop3 => vec![
                "No server-side folders — all folders are local-only",
                "No server-side search",
                "No remote flag synchronisation",
                "Sent, Drafts, and Trash are stored locally only",
            ],
            Protocol::Imap => vec![],
        }
    }
}

/// Collect the distinct set of category labels currently in use across accounts (FR-22, FR-23).
/// Categories are case-sensitive (N-4). Empty/whitespace-only labels are ignored.
pub fn collect_categories(accounts: &[Account]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    for acct in accounts {
        if let Some(cat) = &acct.category {
            let trimmed = cat.trim();
            if !trimmed.is_empty() {
                seen.insert(trimmed.to_string());
            }
        }
    }
    seen.into_iter().collect()
}

/// Errors that can occur when building an account.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AccountValidationError {
    #[error("display name must not be empty")]
    EmptyDisplayName,
    #[error("host must not be empty")]
    EmptyHost,
    #[error("username must not be empty")]
    EmptyUsername,
    #[error("credential must not be empty")]
    EmptyCredential,
}

impl Account {
    /// Create a new account after validating required fields.
    /// Assigns a new UUID automatically (FR-2: globally-unique, stable identifier).
    pub fn new(params: NewAccountParams) -> Result<Self, AccountValidationError> {
        if params.display_name.trim().is_empty() {
            return Err(AccountValidationError::EmptyDisplayName);
        }
        if params.host.trim().is_empty() {
            return Err(AccountValidationError::EmptyHost);
        }
        if params.username.trim().is_empty() {
            return Err(AccountValidationError::EmptyUsername);
        }
        if params.credential.trim().is_empty() {
            return Err(AccountValidationError::EmptyCredential);
        }

        Ok(Self {
            id: Uuid::new_v4(),
            display_name: params.display_name,
            protocol: params.protocol,
            host: params.host,
            port: params.port,
            encryption: params.encryption,
            auth_method: params.auth_method,
            username: params.username,
            credential: params.credential,
            smtp: params.smtp,
            pop3_settings: params.pop3_settings,
            color: params.color,
            avatar_path: params.avatar_path,
            category: params.category,
            sync_enabled: params.sync_enabled,
            on_demand: params.on_demand,
            polling_interval_minutes: params.polling_interval_minutes,
            unmetered_only: params.unmetered_only,
            vpn_only: params.vpn_only,
            schedule_exempt: params.schedule_exempt,
            is_primary: false,
            error_state: None,
            system_folders: params.system_folders,
            swipe_defaults: params.swipe_defaults,
            notifications_enabled: params.notifications_enabled,
            quota: None,
            security_settings: params.security_settings,
            fetch_settings: params.fetch_settings,
            keep_alive_settings: params.keep_alive_settings,
            pop3_needs_store_reevaluation: false,
            oauth_tenant: params.oauth_tenant,
            shared_mailbox: params.shared_mailbox,
        })
    }

    /// Create a new account with a specific UUID (used for importing accounts).
    /// Validates required fields the same way `new()` does.
    pub(crate) fn new_with_id(
        id: Uuid,
        params: NewAccountParams,
    ) -> Result<Self, AccountValidationError> {
        if params.display_name.trim().is_empty() {
            return Err(AccountValidationError::EmptyDisplayName);
        }
        if params.host.trim().is_empty() {
            return Err(AccountValidationError::EmptyHost);
        }
        if params.username.trim().is_empty() {
            return Err(AccountValidationError::EmptyUsername);
        }
        if params.credential.trim().is_empty() {
            return Err(AccountValidationError::EmptyCredential);
        }

        Ok(Self {
            id,
            display_name: params.display_name,
            protocol: params.protocol,
            host: params.host,
            port: params.port,
            encryption: params.encryption,
            auth_method: params.auth_method,
            username: params.username,
            credential: params.credential,
            smtp: params.smtp,
            pop3_settings: params.pop3_settings,
            color: params.color,
            avatar_path: params.avatar_path,
            category: params.category,
            sync_enabled: params.sync_enabled,
            on_demand: params.on_demand,
            polling_interval_minutes: params.polling_interval_minutes,
            unmetered_only: params.unmetered_only,
            vpn_only: params.vpn_only,
            schedule_exempt: params.schedule_exempt,
            is_primary: false,
            error_state: None,
            system_folders: params.system_folders,
            swipe_defaults: params.swipe_defaults,
            notifications_enabled: params.notifications_enabled,
            quota: None,
            security_settings: params.security_settings,
            fetch_settings: params.fetch_settings,
            keep_alive_settings: params.keep_alive_settings,
            pop3_needs_store_reevaluation: false,
            oauth_tenant: params.oauth_tenant,
            shared_mailbox: params.shared_mailbox,
        })
    }

    /// Create an account from store data. Does NOT validate the credential field,
    /// since credentials are now stored in the system keychain and loaded separately.
    pub(crate) fn new_from_store(
        id: Uuid,
        params: NewAccountParams,
    ) -> Result<Self, AccountValidationError> {
        if params.display_name.trim().is_empty() {
            return Err(AccountValidationError::EmptyDisplayName);
        }
        if params.host.trim().is_empty() {
            return Err(AccountValidationError::EmptyHost);
        }
        if params.username.trim().is_empty() {
            return Err(AccountValidationError::EmptyUsername);
        }
        // NOTE: credential is NOT validated here — it may be empty because
        // credentials are stored in the system keychain, not in the database.

        Ok(Self {
            id,
            display_name: params.display_name,
            protocol: params.protocol,
            host: params.host,
            port: params.port,
            encryption: params.encryption,
            auth_method: params.auth_method,
            username: params.username,
            credential: params.credential,
            smtp: params.smtp,
            pop3_settings: params.pop3_settings,
            color: params.color,
            avatar_path: params.avatar_path,
            category: params.category,
            sync_enabled: params.sync_enabled,
            on_demand: params.on_demand,
            polling_interval_minutes: params.polling_interval_minutes,
            unmetered_only: params.unmetered_only,
            vpn_only: params.vpn_only,
            schedule_exempt: params.schedule_exempt,
            is_primary: false,
            error_state: None,
            system_folders: params.system_folders,
            swipe_defaults: params.swipe_defaults,
            notifications_enabled: params.notifications_enabled,
            quota: None,
            security_settings: params.security_settings,
            fetch_settings: params.fetch_settings,
            keep_alive_settings: params.keep_alive_settings,
            pop3_needs_store_reevaluation: false,
            oauth_tenant: params.oauth_tenant,
            shared_mailbox: params.shared_mailbox,
        })
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn encryption(&self) -> EncryptionMode {
        self.encryption
    }

    pub fn auth_method(&self) -> AuthMethod {
        self.auth_method
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn credential(&self) -> &str {
        &self.credential
    }

    pub fn smtp(&self) -> Option<&SmtpConfig> {
        self.smtp.as_ref()
    }

    pub fn pop3_settings(&self) -> Option<&Pop3Settings> {
        self.pop3_settings.as_ref()
    }

    pub fn color(&self) -> Option<AccountColor> {
        self.color
    }

    pub fn avatar_path(&self) -> Option<&str> {
        self.avatar_path.as_deref()
    }

    /// Optional category label for organizing accounts (FR-17, US-17).
    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    pub fn sync_enabled(&self) -> bool {
        self.sync_enabled
    }

    /// Whether this account syncs only on explicit user request (FR-6, US-27, AC-12).
    pub fn on_demand(&self) -> bool {
        self.on_demand
    }

    /// Set the on-demand flag (FR-6, US-27, AC-12).
    pub fn set_on_demand(&mut self, on_demand: bool) {
        self.on_demand = on_demand;
    }

    /// Polling / keep-alive interval in minutes (FR-6, US-28).
    pub fn polling_interval_minutes(&self) -> Option<u32> {
        self.polling_interval_minutes
    }

    /// Set the polling / keep-alive interval in minutes (FR-6, US-28).
    pub fn set_polling_interval_minutes(&mut self, interval: Option<u32>) {
        self.polling_interval_minutes = interval;
    }

    /// Whether sync is suppressed on metered connections (FR-7, US-29).
    pub fn unmetered_only(&self) -> bool {
        self.unmetered_only
    }

    /// Set the unmetered-only flag (FR-7, US-29).
    pub fn set_unmetered_only(&mut self, val: bool) {
        self.unmetered_only = val;
    }

    /// Whether sync is suppressed when no VPN is active (FR-7, US-29, AC-13).
    pub fn vpn_only(&self) -> bool {
        self.vpn_only
    }

    /// Set the VPN-only flag (FR-7, US-29, AC-13).
    pub fn set_vpn_only(&mut self, val: bool) {
        self.vpn_only = val;
    }

    /// Whether this account is exempt from the global sync schedule (FR-7, US-30).
    pub fn schedule_exempt(&self) -> bool {
        self.schedule_exempt
    }

    /// Set the schedule exemption flag (FR-7, US-30).
    pub fn set_schedule_exempt(&mut self, val: bool) {
        self.schedule_exempt = val;
    }

    pub fn is_primary(&self) -> bool {
        self.is_primary
    }

    pub fn error_state(&self) -> Option<&str> {
        self.error_state.as_deref()
    }

    /// Set an error or warning message on this account.
    pub fn set_error_state(&mut self, error: Option<String>) {
        self.error_state = error;
    }

    /// Enable or disable synchronization (FR-6, AC-11).
    /// Disabling clears any active error/warning state (FR-32, AC-11).
    pub fn set_sync_enabled(&mut self, enabled: bool) {
        self.sync_enabled = enabled;
        if !enabled {
            self.error_state = None;
        }
    }

    /// Set or clear primary designation on this account.
    pub fn set_primary(&mut self, primary: bool) {
        self.is_primary = primary;
    }

    /// IMAP system folder designations (FR-35, FR-36, US-36).
    pub fn system_folders(&self) -> Option<&SystemFolders> {
        self.system_folders.as_ref()
    }

    /// Set or clear the system folder designations (FR-35, FR-36, US-36).
    pub fn set_system_folders(&mut self, folders: Option<SystemFolders>) {
        self.system_folders = folders;
    }

    /// Per-account swipe and move defaults (FR-37, FR-38, US-37).
    pub fn swipe_defaults(&self) -> Option<&SwipeDefaults> {
        self.swipe_defaults.as_ref()
    }

    /// Set or clear the swipe and move defaults (FR-37, FR-38, US-37).
    pub fn set_swipe_defaults(&mut self, defaults: Option<SwipeDefaults>) {
        self.swipe_defaults = defaults;
    }

    /// Whether notifications are enabled for this account (FR-39, AC-19).
    pub fn notifications_enabled(&self) -> bool {
        self.notifications_enabled
    }

    /// Enable or disable notifications for this account (FR-39, AC-19).
    /// Independent of sync enablement.
    pub fn set_notifications_enabled(&mut self, enabled: bool) {
        self.notifications_enabled = enabled;
    }

    /// Mailbox quota as reported by the server (FR-42, FR-43, AC-17).
    pub fn quota(&self) -> Option<QuotaInfo> {
        self.quota
    }

    /// Update the quota information from a sync result (FR-42, AC-17).
    /// Pass `None` when the server does not report quota.
    pub fn set_quota(&mut self, quota: Option<QuotaInfo>) {
        self.quota = quota;
    }

    /// Advanced connection security settings (FR-4, FR-53, US-8, US-9, US-10).
    pub fn security_settings(&self) -> Option<&SecuritySettings> {
        self.security_settings.as_ref()
    }

    /// Set or clear the advanced security settings (FR-4, FR-53, US-8, US-9, US-10).
    pub fn set_security_settings(&mut self, settings: Option<SecuritySettings>) {
        self.security_settings = settings;
    }

    /// Advanced fetch settings (FR-51, FR-53).
    pub fn fetch_settings(&self) -> Option<&FetchSettings> {
        self.fetch_settings.as_ref()
    }

    /// Set or clear the advanced fetch settings (FR-51, FR-53).
    pub fn set_fetch_settings(&mut self, settings: Option<FetchSettings>) {
        self.fetch_settings = settings;
    }

    /// Advanced keep-alive settings (FR-52, FR-53).
    pub fn keep_alive_settings(&self) -> Option<&KeepAliveSettings> {
        self.keep_alive_settings.as_ref()
    }

    /// Set or clear the advanced keep-alive settings (FR-52, FR-53).
    pub fn set_keep_alive_settings(&mut self, settings: Option<KeepAliveSettings>) {
        self.keep_alive_settings = settings;
    }

    /// Whether the POP3 "leave on server" setting changed and the local
    /// message store needs re-evaluation at next sync (FR-55).
    pub fn pop3_needs_store_reevaluation(&self) -> bool {
        self.pop3_needs_store_reevaluation
    }

    /// Clear the re-evaluation flag after the sync engine has processed it.
    pub fn clear_pop3_store_reevaluation(&mut self) {
        self.pop3_needs_store_reevaluation = false;
    }

    /// OAuth tenant identifier for multi-tenant providers (FR-10, US-4).
    pub fn oauth_tenant(&self) -> Option<&str> {
        self.oauth_tenant.as_deref()
    }

    /// Shared mailbox address for providers that support delegation (FR-40, N-8).
    pub fn shared_mailbox(&self) -> Option<&str> {
        self.shared_mailbox.as_deref()
    }

    /// Returns the effective username for IMAP/SMTP authentication.
    /// When a shared mailbox is configured, encodes the username as
    /// `shared@domain\user@domain` (Design Note N-8). Otherwise returns the
    /// plain username.
    pub fn effective_username(&self) -> String {
        encode_shared_mailbox_username(&self.username, self.shared_mailbox.as_deref())
    }

    /// Update only the credential and authentication method (FR-33, FR-34).
    /// Used by the re-authorization flow to refresh expired/revoked credentials
    /// without touching any other account properties.
    pub fn update_credentials(&mut self, credential: String, auth_method: AuthMethod) {
        self.credential = credential;
        self.auth_method = auth_method;
    }

    /// Switch the authentication type for both IMAP and SMTP (FR-30, N-5).
    ///
    /// Updates the account-level auth method and credential, and also updates the
    /// SMTP configuration's auth method and credential to match. All other account
    /// properties (folders, messages, settings) are preserved.
    pub fn switch_auth_type(&mut self, credential: String, auth_method: AuthMethod) {
        self.credential = credential.clone();
        self.auth_method = auth_method;
        if let Some(ref mut smtp) = self.smtp {
            smtp.auth_method = auth_method;
            smtp.credential = credential;
        }
    }

    /// Extract the configuration of this account into `NewAccountParams` suitable for
    /// creating a duplicate. The duplicate will NOT inherit:
    /// - The source's unique identifier (a new UUID is assigned on creation)
    /// - The primary designation
    /// - Any mutable state (error state, quota, messages, folders, sync state)
    pub fn to_new_account_params(&self) -> NewAccountParams {
        NewAccountParams {
            display_name: self.display_name.clone(),
            protocol: self.protocol,
            host: self.host.clone(),
            port: self.port,
            encryption: self.encryption,
            auth_method: self.auth_method,
            username: self.username.clone(),
            credential: self.credential.clone(),
            smtp: self.smtp.clone(),
            pop3_settings: self.pop3_settings.clone(),
            color: self.color,
            avatar_path: self.avatar_path.clone(),
            category: self.category.clone(),
            sync_enabled: self.sync_enabled,
            on_demand: self.on_demand,
            polling_interval_minutes: self.polling_interval_minutes,
            unmetered_only: self.unmetered_only,
            vpn_only: self.vpn_only,
            schedule_exempt: self.schedule_exempt,
            system_folders: self.system_folders.clone(),
            swipe_defaults: self.swipe_defaults.clone(),
            notifications_enabled: self.notifications_enabled,
            security_settings: self.security_settings.clone(),
            fetch_settings: self.fetch_settings.clone(),
            keep_alive_settings: self.keep_alive_settings.clone(),
            oauth_tenant: self.oauth_tenant.clone(),
            shared_mailbox: self.shared_mailbox.clone(),
        }
    }

    /// Update all mutable fields on this account, preserving the unique identifier.
    /// Validates the new values the same way `new()` does.
    pub fn update(&mut self, params: UpdateAccountParams) -> Result<(), AccountValidationError> {
        if params.display_name.trim().is_empty() {
            return Err(AccountValidationError::EmptyDisplayName);
        }
        if params.host.trim().is_empty() {
            return Err(AccountValidationError::EmptyHost);
        }
        if params.username.trim().is_empty() {
            return Err(AccountValidationError::EmptyUsername);
        }
        if params.credential.trim().is_empty() {
            return Err(AccountValidationError::EmptyCredential);
        }

        // Detect leave_on_server changes for POP3 re-evaluation (FR-55).
        let old_leave = self.pop3_settings.as_ref().map(|s| s.leave_on_server);
        let new_leave = params.pop3_settings.as_ref().map(|s| s.leave_on_server);
        if old_leave != new_leave && params.protocol == Protocol::Pop3 {
            self.pop3_needs_store_reevaluation = true;
        }

        self.display_name = params.display_name;
        self.protocol = params.protocol;
        self.host = params.host;
        self.port = params.port;
        self.encryption = params.encryption;
        self.auth_method = params.auth_method;
        self.username = params.username;
        self.credential = params.credential;
        self.smtp = params.smtp;
        self.pop3_settings = params.pop3_settings;
        self.color = params.color;
        self.avatar_path = params.avatar_path;
        self.category = params.category;
        self.set_sync_enabled(params.sync_enabled);
        self.on_demand = params.on_demand;
        self.polling_interval_minutes = params.polling_interval_minutes;
        self.unmetered_only = params.unmetered_only;
        self.vpn_only = params.vpn_only;
        self.schedule_exempt = params.schedule_exempt;
        self.system_folders = params.system_folders;
        self.swipe_defaults = params.swipe_defaults;
        self.notifications_enabled = params.notifications_enabled;
        self.security_settings = params.security_settings;
        self.fetch_settings = params.fetch_settings;
        self.keep_alive_settings = params.keep_alive_settings;
        self.oauth_tenant = params.oauth_tenant;
        self.shared_mailbox = params.shared_mailbox;
        Ok(())
    }
}

/// Encode a username for shared mailbox access (Design Note N-8).
/// When `shared_mailbox` is `Some` and non-empty, returns `shared@domain\user@domain`.
/// Otherwise returns the plain username unchanged.
pub fn encode_shared_mailbox_username(username: &str, shared_mailbox: Option<&str>) -> String {
    match shared_mailbox {
        Some(shared) if !shared.trim().is_empty() => {
            format!("{}\\{}", shared.trim(), username)
        }
        _ => username.to_string(),
    }
}

/// Generate a stable notification channel identifier for an account (FR-40).
/// The channel ID is deterministic and derived from the account's UUID.
pub fn notification_channel_id(account_id: Uuid) -> String {
    format!("account-{account_id}")
}

impl std::fmt::Display for EncryptionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::SslTls => write!(f, "SSL/TLS"),
            Self::StartTls => write!(f, "STARTTLS"),
        }
    }
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain => write!(f, "PLAIN"),
            Self::Login => write!(f, "LOGIN"),
            Self::OAuth2 => write!(f, "OAuth2"),
            Self::Certificate => write!(f, "Certificate"),
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Imap => write!(f, "IMAP"),
            Self::Pop3 => write!(f, "POP3"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_params() -> NewAccountParams {
        NewAccountParams {
            display_name: "Work Email".into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            smtp: None,
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
            oauth_tenant: None,
            shared_mailbox: None,
        }
    }

    fn valid_account() -> Account {
        Account::new(valid_params()).unwrap()
    }

    #[test]
    fn new_account_has_unique_id() {
        let a = valid_account();
        let b = valid_account();
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn new_account_stores_all_fields() {
        let a = valid_account();
        assert_eq!(a.display_name(), "Work Email");
        assert_eq!(a.protocol(), Protocol::Imap);
        assert_eq!(a.host(), "imap.example.com");
        assert_eq!(a.port(), 993);
        assert_eq!(a.encryption(), EncryptionMode::SslTls);
        assert_eq!(a.auth_method(), AuthMethod::Plain);
        assert_eq!(a.username(), "user@example.com");
    }

    #[test]
    fn empty_display_name_rejected() {
        let mut p = valid_params();
        p.display_name = "  ".into();
        assert!(matches!(
            Account::new(p),
            Err(AccountValidationError::EmptyDisplayName)
        ));
    }

    #[test]
    fn empty_host_rejected() {
        let mut p = valid_params();
        p.host = "".into();
        assert!(matches!(
            Account::new(p),
            Err(AccountValidationError::EmptyHost)
        ));
    }

    #[test]
    fn empty_username_rejected() {
        let mut p = valid_params();
        p.username = "".into();
        assert!(matches!(
            Account::new(p),
            Err(AccountValidationError::EmptyUsername)
        ));
    }

    #[test]
    fn empty_credential_rejected() {
        let mut p = valid_params();
        p.credential = " ".into();
        assert!(matches!(
            Account::new(p),
            Err(AccountValidationError::EmptyCredential)
        ));
    }

    #[test]
    fn account_serialization_roundtrip() {
        let a = valid_account();
        let json = serde_json::to_string(&a).unwrap();
        let b: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(a.id(), b.id());
        assert_eq!(a.display_name(), b.display_name());
        assert_eq!(a.host(), b.host());
    }

    fn valid_update_params() -> UpdateAccountParams {
        UpdateAccountParams {
            display_name: "Personal Email".into(),
            protocol: Protocol::Pop3,
            host: "pop.example.com".into(),
            port: 995,
            encryption: EncryptionMode::StartTls,
            auth_method: AuthMethod::Login,
            username: "new@example.com".into(),
            credential: "new-secret".into(),
            smtp: None,
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
            oauth_tenant: None,
            shared_mailbox: None,
        }
    }

    #[test]
    fn update_preserves_id() {
        let mut a = valid_account();
        let original_id = a.id();
        a.update(valid_update_params()).unwrap();
        assert_eq!(a.id(), original_id);
    }

    #[test]
    fn update_changes_all_fields() {
        let mut a = valid_account();
        a.update(valid_update_params()).unwrap();
        assert_eq!(a.display_name(), "Personal Email");
        assert_eq!(a.protocol(), Protocol::Pop3);
        assert_eq!(a.host(), "pop.example.com");
        assert_eq!(a.port(), 995);
        assert_eq!(a.encryption(), EncryptionMode::StartTls);
        assert_eq!(a.auth_method(), AuthMethod::Login);
        assert_eq!(a.username(), "new@example.com");
        assert_eq!(a.credential(), "new-secret");
    }

    #[test]
    fn update_rejects_empty_display_name() {
        let mut a = valid_account();
        let mut p = valid_update_params();
        p.display_name = "  ".into();
        assert!(matches!(
            a.update(p),
            Err(AccountValidationError::EmptyDisplayName)
        ));
        // Original fields unchanged after rejected update.
        assert_eq!(a.display_name(), "Work Email");
    }

    #[test]
    fn update_rejects_empty_host() {
        let mut a = valid_account();
        let mut p = valid_update_params();
        p.host = "".into();
        assert!(matches!(
            a.update(p),
            Err(AccountValidationError::EmptyHost)
        ));
    }

    #[test]
    fn update_rejects_empty_username() {
        let mut a = valid_account();
        let mut p = valid_update_params();
        p.username = "".into();
        assert!(matches!(
            a.update(p),
            Err(AccountValidationError::EmptyUsername)
        ));
    }

    #[test]
    fn update_rejects_empty_credential() {
        let mut a = valid_account();
        let mut p = valid_update_params();
        p.credential = " ".into();
        assert!(matches!(
            a.update(p),
            Err(AccountValidationError::EmptyCredential)
        ));
    }

    #[test]
    fn update_no_partial_mutation_on_validation_failure() {
        let mut a = valid_account();
        let original_host = a.host().to_string();
        let mut p = valid_update_params();
        p.credential = "".into(); // Will fail validation
        let _ = a.update(p);
        // No fields should have changed.
        assert_eq!(a.host(), original_host);
        assert_eq!(a.display_name(), "Work Email");
    }

    // -- POP3 folder structure tests (FR-10) --

    #[test]
    fn pop3_default_folders_are_fixed_local_set() {
        let folders = Protocol::Pop3.default_folders();
        let names: Vec<&str> = folders.iter().map(|f| f.name()).collect();
        assert_eq!(names, vec!["Inbox", "Drafts", "Sent", "Trash"]);
        assert!(folders.iter().all(|f| f.is_local_only()));
    }

    #[test]
    fn imap_default_folders_include_inbox() {
        let folders = Protocol::Imap.default_folders();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].name(), "Inbox");
        assert!(!folders[0].is_local_only());
    }

    // -- POP3 limitations tests (US-35) --

    #[test]
    fn pop3_has_limitations() {
        let lims = Protocol::Pop3.limitations();
        assert!(!lims.is_empty());
        assert!(lims.iter().any(|l| l.contains("server-side folders")));
        assert!(lims.iter().any(|l| l.contains("server-side search")));
        assert!(lims.iter().any(|l| l.contains("flag")));
        assert!(lims.iter().any(|l| l.contains("local")));
    }

    #[test]
    fn imap_has_no_limitations() {
        assert!(Protocol::Imap.limitations().is_empty());
    }

    #[test]
    fn folder_serialization_roundtrip() {
        let folders = Protocol::Pop3.default_folders();
        let json = serde_json::to_string(&folders).unwrap();
        let parsed: Vec<Folder> = serde_json::from_str(&json).unwrap();
        assert_eq!(folders, parsed);
    }

    #[test]
    fn pop3_account_creation_and_folder_access() {
        let mut p = valid_params();
        p.protocol = Protocol::Pop3;
        p.host = "pop.example.com".into();
        p.port = 995;
        let acct = Account::new(p).unwrap();
        assert_eq!(acct.protocol(), Protocol::Pop3);
        let folders = acct.protocol().default_folders();
        assert_eq!(folders.len(), 4);
    }

    // -- POP3-specific settings tests (US-31, US-32, US-33, US-34, FR-9) --

    #[test]
    fn pop3_settings_default_values() {
        let settings = Pop3Settings::default();
        assert!(settings.leave_on_server);
        assert!(!settings.delete_from_server_when_deleted_on_device);
        assert!(settings.keep_on_device_when_deleted_from_server);
        assert_eq!(settings.max_messages_to_download, None);
    }

    #[test]
    fn pop3_account_stores_pop3_settings() {
        let mut p = valid_params();
        p.protocol = Protocol::Pop3;
        p.host = "pop.example.com".into();
        p.port = 995;
        p.pop3_settings = Some(Pop3Settings::default());
        let acct = Account::new(p).unwrap();
        let settings = acct.pop3_settings().unwrap();
        assert!(settings.leave_on_server);
        assert!(!settings.delete_from_server_when_deleted_on_device);
        assert!(settings.keep_on_device_when_deleted_from_server);
        assert_eq!(settings.max_messages_to_download, None);
    }

    #[test]
    fn pop3_account_custom_settings() {
        let mut p = valid_params();
        p.protocol = Protocol::Pop3;
        p.host = "pop.example.com".into();
        p.port = 995;
        p.pop3_settings = Some(Pop3Settings {
            leave_on_server: false,
            delete_from_server_when_deleted_on_device: true,
            keep_on_device_when_deleted_from_server: false,
            max_messages_to_download: Some(500),
            apop_enabled: false,
        });
        let acct = Account::new(p).unwrap();
        let settings = acct.pop3_settings().unwrap();
        assert!(!settings.leave_on_server);
        assert!(settings.delete_from_server_when_deleted_on_device);
        assert!(!settings.keep_on_device_when_deleted_from_server);
        assert_eq!(settings.max_messages_to_download, Some(500));
    }

    #[test]
    fn imap_account_has_no_pop3_settings() {
        let acct = valid_account();
        assert!(acct.pop3_settings().is_none());
    }

    #[test]
    fn update_preserves_pop3_settings() {
        let mut p = valid_params();
        p.protocol = Protocol::Pop3;
        p.host = "pop.example.com".into();
        p.port = 995;
        p.pop3_settings = Some(Pop3Settings::default());
        let mut acct = Account::new(p).unwrap();
        let original_id = acct.id();

        let up = UpdateAccountParams {
            display_name: "Updated POP3".into(),
            protocol: Protocol::Pop3,
            host: "pop2.example.com".into(),
            port: 995,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            smtp: None,
            pop3_settings: Some(Pop3Settings {
                leave_on_server: false,
                delete_from_server_when_deleted_on_device: true,
                keep_on_device_when_deleted_from_server: false,
                max_messages_to_download: Some(100),
                apop_enabled: false,
            }),
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
            oauth_tenant: None,
            shared_mailbox: None,
        };
        acct.update(up).unwrap();
        assert_eq!(acct.id(), original_id);
        let settings = acct.pop3_settings().unwrap();
        assert!(!settings.leave_on_server);
        assert!(settings.delete_from_server_when_deleted_on_device);
        assert_eq!(settings.max_messages_to_download, Some(100));
    }

    #[test]
    fn pop3_settings_serialization_roundtrip() {
        let mut p = valid_params();
        p.protocol = Protocol::Pop3;
        p.host = "pop.example.com".into();
        p.port = 995;
        p.pop3_settings = Some(Pop3Settings {
            leave_on_server: false,
            delete_from_server_when_deleted_on_device: true,
            keep_on_device_when_deleted_from_server: false,
            max_messages_to_download: Some(250),
            apop_enabled: false,
        });
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(acct.pop3_settings(), restored.pop3_settings());
    }

    #[test]
    fn deserialize_account_without_pop3_settings_defaults_to_none() {
        // Simulates loading an account saved before pop3_settings existed.
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("pop3_settings");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.pop3_settings().is_none());
    }

    // -- POP3 leave_on_server re-evaluation tests (FR-55) --

    #[test]
    fn new_account_has_no_reevaluation_flag() {
        let mut p = valid_params();
        p.protocol = Protocol::Pop3;
        p.host = "pop.example.com".into();
        p.pop3_settings = Some(Pop3Settings::default());
        let acct = Account::new(p).unwrap();
        assert!(!acct.pop3_needs_store_reevaluation());
    }

    #[test]
    fn update_sets_reevaluation_when_leave_on_server_changes() {
        let mut p = valid_params();
        p.protocol = Protocol::Pop3;
        p.host = "pop.example.com".into();
        p.pop3_settings = Some(Pop3Settings::default()); // leave_on_server = true
        let mut acct = Account::new(p).unwrap();

        let up = UpdateAccountParams {
            display_name: "POP3".into(),
            protocol: Protocol::Pop3,
            host: "pop.example.com".into(),
            port: 995,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            smtp: None,
            pop3_settings: Some(Pop3Settings {
                leave_on_server: false, // changed from true to false
                ..Pop3Settings::default()
            }),
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
            oauth_tenant: None,
            shared_mailbox: None,
        };
        acct.update(up).unwrap();
        assert!(acct.pop3_needs_store_reevaluation());
    }

    #[test]
    fn update_no_reevaluation_when_leave_on_server_unchanged() {
        let mut p = valid_params();
        p.protocol = Protocol::Pop3;
        p.host = "pop.example.com".into();
        p.pop3_settings = Some(Pop3Settings::default()); // leave_on_server = true
        let mut acct = Account::new(p).unwrap();

        let up = UpdateAccountParams {
            display_name: "POP3 Updated".into(),
            protocol: Protocol::Pop3,
            host: "pop2.example.com".into(), // host changed, but leave_on_server same
            port: 995,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            smtp: None,
            pop3_settings: Some(Pop3Settings::default()), // leave_on_server still true
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
            oauth_tenant: None,
            shared_mailbox: None,
        };
        acct.update(up).unwrap();
        assert!(!acct.pop3_needs_store_reevaluation());
    }

    #[test]
    fn clear_reevaluation_flag() {
        let mut p = valid_params();
        p.protocol = Protocol::Pop3;
        p.host = "pop.example.com".into();
        p.pop3_settings = Some(Pop3Settings::default());
        let mut acct = Account::new(p).unwrap();

        // Force the flag on by updating leave_on_server.
        let up = UpdateAccountParams {
            display_name: "POP3".into(),
            protocol: Protocol::Pop3,
            host: "pop.example.com".into(),
            port: 995,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            smtp: None,
            pop3_settings: Some(Pop3Settings {
                leave_on_server: false,
                ..Pop3Settings::default()
            }),
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
            oauth_tenant: None,
            shared_mailbox: None,
        };
        acct.update(up).unwrap();
        assert!(acct.pop3_needs_store_reevaluation());

        acct.clear_pop3_store_reevaluation();
        assert!(!acct.pop3_needs_store_reevaluation());
    }

    // -- Account colour tests (FR-5, FR-12, FR-15) --

    #[test]
    fn account_color_defaults_to_none() {
        let acct = valid_account();
        assert!(acct.color().is_none());
    }

    #[test]
    fn account_color_can_be_set_on_creation() {
        let mut p = valid_params();
        p.color = Some(AccountColor::new(1.0, 0.0, 0.5));
        let acct = Account::new(p).unwrap();
        let c = acct.color().unwrap();
        assert!((c.red - 1.0).abs() < f32::EPSILON);
        assert!((c.green).abs() < f32::EPSILON);
        assert!((c.blue - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn account_color_can_be_changed_via_update() {
        let mut acct = valid_account();
        assert!(acct.color().is_none());
        let mut up = valid_update_params();
        up.color = Some(AccountColor::new(0.2, 0.4, 0.6));
        acct.update(up).unwrap();
        let c = acct.color().unwrap();
        assert!((c.red - 0.2).abs() < f32::EPSILON);
        assert!((c.green - 0.4).abs() < f32::EPSILON);
        assert!((c.blue - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn account_color_can_be_cleared_via_update() {
        let mut p = valid_params();
        p.color = Some(AccountColor::new(1.0, 0.0, 0.0));
        let mut acct = Account::new(p).unwrap();
        assert!(acct.color().is_some());
        let mut up = valid_update_params();
        up.color = None;
        acct.update(up).unwrap();
        assert!(acct.color().is_none());
    }

    #[test]
    fn account_color_serialization_roundtrip() {
        let mut p = valid_params();
        p.color = Some(AccountColor::new(0.1, 0.2, 0.3));
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(acct.color(), restored.color());
    }

    #[test]
    fn deserialize_account_without_color_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("color");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.color().is_none());
    }

    #[test]
    fn account_color_to_hex() {
        let c = AccountColor::new(1.0, 0.533, 0.0);
        let hex = c.to_hex();
        assert_eq!(hex, "#ff8800");
    }

    #[test]
    fn account_color_to_hex_clamps() {
        let c = AccountColor::new(1.5, -0.1, 0.5);
        let hex = c.to_hex();
        assert_eq!(hex, "#ff0080");
    }

    // -- Colour precedence tests (FR-15) --

    #[test]
    fn resolve_color_returns_none_when_all_unset() {
        assert!(resolve_color(None, None, None).is_none());
    }

    #[test]
    fn resolve_color_returns_account_when_only_account_set() {
        let ac = AccountColor::new(1.0, 0.0, 0.0);
        assert_eq!(resolve_color(Some(ac), None, None), Some(ac));
    }

    #[test]
    fn resolve_color_folder_overrides_account() {
        let ac = AccountColor::new(1.0, 0.0, 0.0);
        let fc = AccountColor::new(0.0, 1.0, 0.0);
        assert_eq!(resolve_color(Some(ac), Some(fc), None), Some(fc));
    }

    #[test]
    fn resolve_color_identity_overrides_all() {
        let ac = AccountColor::new(1.0, 0.0, 0.0);
        let fc = AccountColor::new(0.0, 1.0, 0.0);
        let ic = AccountColor::new(0.0, 0.0, 1.0);
        assert_eq!(resolve_color(Some(ac), Some(fc), Some(ic)), Some(ic));
    }

    #[test]
    fn resolve_color_identity_overrides_account_skipping_folder() {
        let ac = AccountColor::new(1.0, 0.0, 0.0);
        let ic = AccountColor::new(0.0, 0.0, 1.0);
        assert_eq!(resolve_color(Some(ac), None, Some(ic)), Some(ic));
    }

    // -- Account avatar tests (FR-5, FR-13, US-15, US-16) --

    #[test]
    fn account_avatar_defaults_to_none() {
        let acct = valid_account();
        assert!(acct.avatar_path().is_none());
    }

    #[test]
    fn account_avatar_can_be_set_on_creation() {
        let mut p = valid_params();
        p.avatar_path = Some("/home/user/photo.png".into());
        let acct = Account::new(p).unwrap();
        assert_eq!(acct.avatar_path(), Some("/home/user/photo.png"));
    }

    #[test]
    fn account_avatar_can_be_changed_via_update() {
        let mut acct = valid_account();
        assert!(acct.avatar_path().is_none());
        let mut up = valid_update_params();
        up.avatar_path = Some("/tmp/avatar.jpg".into());
        acct.update(up).unwrap();
        assert_eq!(acct.avatar_path(), Some("/tmp/avatar.jpg"));
    }

    #[test]
    fn account_avatar_can_be_cleared_via_update() {
        let mut p = valid_params();
        p.avatar_path = Some("/tmp/avatar.png".into());
        let mut acct = Account::new(p).unwrap();
        assert!(acct.avatar_path().is_some());
        let mut up = valid_update_params();
        up.avatar_path = None;
        acct.update(up).unwrap();
        assert!(acct.avatar_path().is_none());
    }

    #[test]
    fn account_avatar_serialization_roundtrip() {
        let mut p = valid_params();
        p.avatar_path = Some("/data/avatars/work.png".into());
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(acct.avatar_path(), restored.avatar_path());
    }

    #[test]
    fn deserialize_account_without_avatar_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("avatar_path");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.avatar_path().is_none());
    }

    // -- Sync toggle tests (FR-6, AC-11, AC-19) --

    #[test]
    fn set_sync_enabled_disables_sync() {
        let mut acct = valid_account();
        assert!(acct.sync_enabled());
        acct.set_sync_enabled(false);
        assert!(!acct.sync_enabled());
    }

    #[test]
    fn set_sync_enabled_enables_sync() {
        let mut p = valid_params();
        p.sync_enabled = false;
        let mut acct = Account::new(p).unwrap();
        assert!(!acct.sync_enabled());
        acct.set_sync_enabled(true);
        assert!(acct.sync_enabled());
    }

    #[test]
    fn disabling_sync_clears_error_state() {
        let mut acct = valid_account();
        acct.set_error_state(Some("connection timeout".into()));
        assert!(acct.error_state().is_some());
        acct.set_sync_enabled(false);
        assert!(acct.error_state().is_none());
    }

    #[test]
    fn enabling_sync_preserves_error_state() {
        let mut p = valid_params();
        p.sync_enabled = false;
        let mut acct = Account::new(p).unwrap();
        acct.set_error_state(Some("stale error".into()));
        acct.set_sync_enabled(true);
        assert_eq!(acct.error_state(), Some("stale error"));
    }

    #[test]
    fn update_disabling_sync_clears_error_state() {
        let mut acct = valid_account();
        acct.set_error_state(Some("auth failed".into()));
        let mut up = valid_update_params();
        up.sync_enabled = false;
        acct.update(up).unwrap();
        assert!(!acct.sync_enabled());
        assert!(acct.error_state().is_none());
    }

    #[test]
    fn error_state_defaults_to_none() {
        let acct = valid_account();
        assert!(acct.error_state().is_none());
    }

    #[test]
    fn error_state_serialization_roundtrip() {
        let mut acct = valid_account();
        acct.set_error_state(Some("sync failure".into()));
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.error_state(), Some("sync failure"));
    }

    #[test]
    fn deserialize_account_without_error_state_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("error_state");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.error_state().is_none());
    }

    // -- On-demand sync tests (FR-6, US-27, AC-12) --

    #[test]
    fn on_demand_defaults_to_false() {
        let acct = valid_account();
        assert!(!acct.on_demand());
    }

    #[test]
    fn on_demand_can_be_set_on_creation() {
        let mut p = valid_params();
        p.on_demand = true;
        let acct = Account::new(p).unwrap();
        assert!(acct.on_demand());
    }

    #[test]
    fn set_on_demand_toggles_flag() {
        let mut acct = valid_account();
        acct.set_on_demand(true);
        assert!(acct.on_demand());
        acct.set_on_demand(false);
        assert!(!acct.on_demand());
    }

    #[test]
    fn on_demand_independent_of_sync_enabled() {
        let mut acct = valid_account();
        acct.set_on_demand(true);
        acct.set_sync_enabled(false);
        assert!(acct.on_demand());
        assert!(!acct.sync_enabled());
    }

    #[test]
    fn on_demand_changed_via_update() {
        let mut acct = valid_account();
        assert!(!acct.on_demand());
        let mut up = valid_update_params();
        up.on_demand = true;
        acct.update(up).unwrap();
        assert!(acct.on_demand());
    }

    #[test]
    fn on_demand_serialization_roundtrip() {
        let mut p = valid_params();
        p.on_demand = true;
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert!(restored.on_demand());
    }

    #[test]
    fn deserialize_account_without_on_demand_defaults_to_false() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("on_demand");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(!restored.on_demand());
    }

    // -- Polling interval tests (FR-6, US-28) --

    #[test]
    fn polling_interval_defaults_to_none() {
        let acct = valid_account();
        assert!(acct.polling_interval_minutes().is_none());
    }

    #[test]
    fn polling_interval_can_be_set_on_creation() {
        let mut p = valid_params();
        p.polling_interval_minutes = Some(15);
        let acct = Account::new(p).unwrap();
        assert_eq!(acct.polling_interval_minutes(), Some(15));
    }

    #[test]
    fn set_polling_interval() {
        let mut acct = valid_account();
        acct.set_polling_interval_minutes(Some(30));
        assert_eq!(acct.polling_interval_minutes(), Some(30));
        acct.set_polling_interval_minutes(None);
        assert!(acct.polling_interval_minutes().is_none());
    }

    #[test]
    fn polling_interval_independent_of_sync_enabled_and_on_demand() {
        let mut acct = valid_account();
        acct.set_polling_interval_minutes(Some(10));
        acct.set_sync_enabled(false);
        acct.set_on_demand(true);
        assert_eq!(acct.polling_interval_minutes(), Some(10));
        assert!(!acct.sync_enabled());
        assert!(acct.on_demand());
    }

    #[test]
    fn polling_interval_changed_via_update() {
        let mut acct = valid_account();
        let mut up = valid_update_params();
        up.polling_interval_minutes = Some(5);
        acct.update(up).unwrap();
        assert_eq!(acct.polling_interval_minutes(), Some(5));
    }

    #[test]
    fn polling_interval_serialization_roundtrip() {
        let mut p = valid_params();
        p.polling_interval_minutes = Some(60);
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.polling_interval_minutes(), Some(60));
    }

    #[test]
    fn deserialize_account_without_polling_interval_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut()
            .unwrap()
            .remove("polling_interval_minutes");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.polling_interval_minutes().is_none());
    }

    // -- Network condition fields tests (FR-7, US-29, US-30) --

    #[test]
    fn unmetered_only_defaults_to_false() {
        let acct = valid_account();
        assert!(!acct.unmetered_only());
    }

    #[test]
    fn unmetered_only_can_be_set_on_creation() {
        let mut p = valid_params();
        p.unmetered_only = true;
        let acct = Account::new(p).unwrap();
        assert!(acct.unmetered_only());
    }

    #[test]
    fn set_unmetered_only_toggles() {
        let mut acct = valid_account();
        acct.set_unmetered_only(true);
        assert!(acct.unmetered_only());
        acct.set_unmetered_only(false);
        assert!(!acct.unmetered_only());
    }

    #[test]
    fn vpn_only_defaults_to_false() {
        let acct = valid_account();
        assert!(!acct.vpn_only());
    }

    #[test]
    fn vpn_only_can_be_set_on_creation() {
        let mut p = valid_params();
        p.vpn_only = true;
        let acct = Account::new(p).unwrap();
        assert!(acct.vpn_only());
    }

    #[test]
    fn set_vpn_only_toggles() {
        let mut acct = valid_account();
        acct.set_vpn_only(true);
        assert!(acct.vpn_only());
        acct.set_vpn_only(false);
        assert!(!acct.vpn_only());
    }

    #[test]
    fn schedule_exempt_defaults_to_false() {
        let acct = valid_account();
        assert!(!acct.schedule_exempt());
    }

    #[test]
    fn schedule_exempt_can_be_set_on_creation() {
        let mut p = valid_params();
        p.schedule_exempt = true;
        let acct = Account::new(p).unwrap();
        assert!(acct.schedule_exempt());
    }

    #[test]
    fn set_schedule_exempt_toggles() {
        let mut acct = valid_account();
        acct.set_schedule_exempt(true);
        assert!(acct.schedule_exempt());
        acct.set_schedule_exempt(false);
        assert!(!acct.schedule_exempt());
    }

    #[test]
    fn network_conditions_changed_via_update() {
        let mut acct = valid_account();
        let mut up = valid_update_params();
        up.unmetered_only = true;
        up.vpn_only = true;
        up.schedule_exempt = true;
        acct.update(up).unwrap();
        assert!(acct.unmetered_only());
        assert!(acct.vpn_only());
        assert!(acct.schedule_exempt());
    }

    #[test]
    fn network_conditions_serialization_roundtrip() {
        let mut p = valid_params();
        p.unmetered_only = true;
        p.vpn_only = true;
        p.schedule_exempt = true;
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert!(restored.unmetered_only());
        assert!(restored.vpn_only());
        assert!(restored.schedule_exempt());
    }

    #[test]
    fn deserialize_account_without_network_conditions_defaults_to_false() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("unmetered_only");
        json.as_object_mut().unwrap().remove("vpn_only");
        json.as_object_mut().unwrap().remove("schedule_exempt");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(!restored.unmetered_only());
        assert!(!restored.vpn_only());
        assert!(!restored.schedule_exempt());
    }

    #[test]
    fn network_conditions_independent_of_sync_enabled() {
        let mut acct = valid_account();
        acct.set_unmetered_only(true);
        acct.set_vpn_only(true);
        acct.set_schedule_exempt(true);
        acct.set_sync_enabled(false);
        assert!(acct.unmetered_only());
        assert!(acct.vpn_only());
        assert!(acct.schedule_exempt());
        assert!(!acct.sync_enabled());
    }

    // -- Category label tests (FR-17, US-17, FR-22, FR-23, N-4) --

    #[test]
    fn category_defaults_to_none() {
        let acct = valid_account();
        assert!(acct.category().is_none());
    }

    #[test]
    fn category_can_be_set_on_creation() {
        let mut p = valid_params();
        p.category = Some("Work".into());
        let acct = Account::new(p).unwrap();
        assert_eq!(acct.category(), Some("Work"));
    }

    #[test]
    fn category_can_be_changed_via_update() {
        let mut acct = valid_account();
        assert!(acct.category().is_none());
        let mut up = valid_update_params();
        up.category = Some("Personal".into());
        acct.update(up).unwrap();
        assert_eq!(acct.category(), Some("Personal"));
    }

    #[test]
    fn category_can_be_cleared_via_update() {
        let mut p = valid_params();
        p.category = Some("Work".into());
        let mut acct = Account::new(p).unwrap();
        assert!(acct.category().is_some());
        let mut up = valid_update_params();
        up.category = None;
        acct.update(up).unwrap();
        assert!(acct.category().is_none());
    }

    #[test]
    fn category_serialization_roundtrip() {
        let mut p = valid_params();
        p.category = Some("Finance".into());
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(acct.category(), restored.category());
    }

    #[test]
    fn deserialize_account_without_category_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("category");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.category().is_none());
    }

    #[test]
    fn category_is_case_sensitive() {
        let mut p1 = valid_params();
        p1.category = Some("Work".into());
        let a1 = Account::new(p1).unwrap();

        let mut p2 = valid_params();
        p2.category = Some("work".into());
        let a2 = Account::new(p2).unwrap();

        assert_ne!(a1.category(), a2.category());
    }

    // -- collect_categories tests (FR-22, FR-23) --

    #[test]
    fn collect_categories_empty_when_no_accounts() {
        assert!(collect_categories(&[]).is_empty());
    }

    #[test]
    fn collect_categories_empty_when_no_categories_set() {
        let accounts = vec![valid_account(), valid_account()];
        assert!(collect_categories(&accounts).is_empty());
    }

    #[test]
    fn collect_categories_returns_distinct_sorted() {
        let mut p1 = valid_params();
        p1.category = Some("Work".into());
        let a1 = Account::new(p1).unwrap();

        let mut p2 = valid_params();
        p2.category = Some("Personal".into());
        let a2 = Account::new(p2).unwrap();

        let mut p3 = valid_params();
        p3.category = Some("Work".into());
        let a3 = Account::new(p3).unwrap();

        let cats = collect_categories(&[a1, a2, a3]);
        assert_eq!(cats, vec!["Personal", "Work"]);
    }

    #[test]
    fn collect_categories_is_case_sensitive() {
        let mut p1 = valid_params();
        p1.category = Some("Work".into());
        let a1 = Account::new(p1).unwrap();

        let mut p2 = valid_params();
        p2.category = Some("work".into());
        let a2 = Account::new(p2).unwrap();

        let cats = collect_categories(&[a1, a2]);
        assert_eq!(cats, vec!["Work", "work"]);
    }

    #[test]
    fn collect_categories_ignores_empty_and_whitespace() {
        let mut p1 = valid_params();
        p1.category = Some("".into());
        let a1 = Account::new(p1).unwrap();

        let mut p2 = valid_params();
        p2.category = Some("   ".into());
        let a2 = Account::new(p2).unwrap();

        let mut p3 = valid_params();
        p3.category = Some("Work".into());
        let a3 = Account::new(p3).unwrap();

        let cats = collect_categories(&[a1, a2, a3]);
        assert_eq!(cats, vec!["Work"]);
    }

    #[test]
    fn collect_categories_implicitly_deleted_when_no_account_has_it() {
        let mut p1 = valid_params();
        p1.category = Some("Work".into());
        let mut a1 = Account::new(p1).unwrap();

        let mut p2 = valid_params();
        p2.category = Some("Personal".into());
        let a2 = Account::new(p2).unwrap();

        // Both categories exist.
        let cats = collect_categories(&[a1.clone(), a2.clone()]);
        assert_eq!(cats.len(), 2);

        // Remove "Work" from the only account that had it.
        let mut up = valid_update_params();
        up.category = None;
        a1.update(up).unwrap();

        let cats = collect_categories(&[a1, a2]);
        assert_eq!(cats, vec!["Personal"]);
    }

    // -- System folder designation tests (FR-35, FR-36, US-36) --

    #[test]
    fn system_folders_defaults_to_none() {
        let acct = valid_account();
        assert!(acct.system_folders().is_none());
    }

    #[test]
    fn system_folders_can_be_set_on_creation() {
        let mut p = valid_params();
        p.system_folders = Some(SystemFolders {
            drafts: Some("Drafts".into()),
            sent: Some("Sent".into()),
            archive: Some("Archive".into()),
            trash: Some("Trash".into()),
            junk: Some("Spam".into()),
        });
        let acct = Account::new(p).unwrap();
        let sf = acct.system_folders().unwrap();
        assert_eq!(sf.drafts.as_deref(), Some("Drafts"));
        assert_eq!(sf.sent.as_deref(), Some("Sent"));
        assert_eq!(sf.archive.as_deref(), Some("Archive"));
        assert_eq!(sf.trash.as_deref(), Some("Trash"));
        assert_eq!(sf.junk.as_deref(), Some("Spam"));
    }

    #[test]
    fn system_folders_can_be_changed_via_update() {
        let mut acct = valid_account();
        assert!(acct.system_folders().is_none());
        let mut up = valid_update_params();
        up.system_folders = Some(SystemFolders {
            drafts: Some("MyDrafts".into()),
            sent: None,
            archive: None,
            trash: Some("Deleted Items".into()),
            junk: None,
        });
        acct.update(up).unwrap();
        let sf = acct.system_folders().unwrap();
        assert_eq!(sf.drafts.as_deref(), Some("MyDrafts"));
        assert_eq!(sf.trash.as_deref(), Some("Deleted Items"));
        assert!(sf.sent.is_none());
    }

    #[test]
    fn system_folders_can_be_cleared_via_update() {
        let mut p = valid_params();
        p.system_folders = Some(SystemFolders {
            drafts: Some("Drafts".into()),
            ..Default::default()
        });
        let mut acct = Account::new(p).unwrap();
        assert!(acct.system_folders().is_some());
        let mut up = valid_update_params();
        up.system_folders = None;
        acct.update(up).unwrap();
        assert!(acct.system_folders().is_none());
    }

    #[test]
    fn system_folders_set_and_get_by_role() {
        let mut sf = SystemFolders::default();
        assert!(sf.is_empty());
        sf.set(FolderRole::Drafts, Some("Drafts".into()));
        sf.set(FolderRole::Junk, Some("Bulk Mail".into()));
        assert_eq!(sf.get(FolderRole::Drafts), Some("Drafts"));
        assert_eq!(sf.get(FolderRole::Junk), Some("Bulk Mail"));
        assert!(sf.get(FolderRole::Sent).is_none());
        assert!(!sf.is_empty());
    }

    #[test]
    fn system_folders_serialization_roundtrip() {
        let mut p = valid_params();
        p.system_folders = Some(SystemFolders {
            drafts: Some("Drafts".into()),
            sent: Some("Sent Items".into()),
            archive: None,
            trash: Some("Deleted Items".into()),
            junk: Some("Junk E-mail".into()),
        });
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(acct.system_folders(), restored.system_folders());
    }

    #[test]
    fn deserialize_account_without_system_folders_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("system_folders");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.system_folders().is_none());
    }

    #[test]
    fn system_folders_override_auto_detected() {
        // Simulate auto-detection then user override (FR-36).
        let detected = detect_system_folders(&[
            ("Drafts".into(), "\\Drafts".into()),
            ("Sent".into(), "\\Sent".into()),
            ("Trash".into(), "\\Trash".into()),
        ]);
        assert_eq!(detected.drafts.as_deref(), Some("Drafts"));
        assert_eq!(detected.sent.as_deref(), Some("Sent"));
        assert_eq!(detected.trash.as_deref(), Some("Trash"));

        // User overrides Sent folder.
        let mut overridden = detected.clone();
        overridden.sent = Some("Sent Messages".into());
        assert_eq!(overridden.sent.as_deref(), Some("Sent Messages"));
        // Other auto-detected values remain.
        assert_eq!(overridden.drafts.as_deref(), Some("Drafts"));
    }

    #[test]
    fn detect_system_folders_from_special_use() {
        let attrs = vec![
            ("Drafts".into(), "\\Drafts".into()),
            ("Sent".into(), "\\Sent".into()),
            ("All Mail".into(), "\\Archive".into()),
            ("Bin".into(), "\\Trash".into()),
            ("Spam".into(), "\\Junk".into()),
            ("INBOX".into(), "\\Inbox".into()), // Not a system folder role
        ];
        let sf = detect_system_folders(&attrs);
        assert_eq!(sf.drafts.as_deref(), Some("Drafts"));
        assert_eq!(sf.sent.as_deref(), Some("Sent"));
        assert_eq!(sf.archive.as_deref(), Some("All Mail"));
        assert_eq!(sf.trash.as_deref(), Some("Bin"));
        assert_eq!(sf.junk.as_deref(), Some("Spam"));
    }

    #[test]
    fn detect_system_folders_partial() {
        let attrs = vec![("Sent".into(), "\\Sent".into())];
        let sf = detect_system_folders(&attrs);
        assert!(sf.drafts.is_none());
        assert_eq!(sf.sent.as_deref(), Some("Sent"));
        assert!(sf.archive.is_none());
        assert!(sf.trash.is_none());
        assert!(sf.junk.is_none());
    }

    #[test]
    fn detect_system_folders_empty_input() {
        let sf = detect_system_folders(&[]);
        assert!(sf.is_empty());
    }

    #[test]
    fn folder_role_all_returns_five_roles() {
        assert_eq!(FolderRole::all().len(), 5);
    }

    #[test]
    fn folder_role_display() {
        assert_eq!(FolderRole::Drafts.to_string(), "Drafts");
        assert_eq!(FolderRole::Sent.to_string(), "Sent");
        assert_eq!(FolderRole::Archive.to_string(), "Archive");
        assert_eq!(FolderRole::Trash.to_string(), "Trash");
        assert_eq!(FolderRole::Junk.to_string(), "Junk");
    }

    #[test]
    fn system_folders_set_via_setter_method() {
        let mut acct = valid_account();
        acct.set_system_folders(Some(SystemFolders {
            drafts: Some("Drafts".into()),
            ..Default::default()
        }));
        assert_eq!(
            acct.system_folders().unwrap().drafts.as_deref(),
            Some("Drafts")
        );
        acct.set_system_folders(None);
        assert!(acct.system_folders().is_none());
    }

    #[test]
    fn pop3_account_system_folders_not_applicable() {
        // POP3 accounts have fixed local folders; system_folders should be None.
        let mut p = valid_params();
        p.protocol = Protocol::Pop3;
        p.host = "pop.example.com".into();
        p.port = 995;
        p.pop3_settings = Some(Pop3Settings::default());
        p.system_folders = None;
        let acct = Account::new(p).unwrap();
        assert!(acct.system_folders().is_none());
    }

    // -- Swipe and move defaults tests (FR-37, FR-38, US-37) --

    #[test]
    fn swipe_defaults_none_by_default() {
        let acct = valid_account();
        assert!(acct.swipe_defaults().is_none());
    }

    #[test]
    fn swipe_defaults_can_be_set_on_creation() {
        let mut p = valid_params();
        p.swipe_defaults = Some(SwipeDefaults {
            swipe_left: SwipeAction::Delete,
            swipe_right: SwipeAction::Archive,
            default_move_to: Some("Archive".into()),
        });
        let acct = Account::new(p).unwrap();
        let sd = acct.swipe_defaults().unwrap();
        assert_eq!(sd.swipe_left, SwipeAction::Delete);
        assert_eq!(sd.swipe_right, SwipeAction::Archive);
        assert_eq!(sd.default_move_to.as_deref(), Some("Archive"));
    }

    #[test]
    fn swipe_defaults_can_be_changed_via_update() {
        let mut acct = valid_account();
        assert!(acct.swipe_defaults().is_none());
        let mut up = valid_update_params();
        up.swipe_defaults = Some(SwipeDefaults {
            swipe_left: SwipeAction::MarkRead,
            swipe_right: SwipeAction::MoveToFolder("Important".into()),
            default_move_to: Some("Reviewed".into()),
        });
        acct.update(up).unwrap();
        let sd = acct.swipe_defaults().unwrap();
        assert_eq!(sd.swipe_left, SwipeAction::MarkRead);
        assert_eq!(
            sd.swipe_right,
            SwipeAction::MoveToFolder("Important".into())
        );
        assert_eq!(sd.default_move_to.as_deref(), Some("Reviewed"));
    }

    #[test]
    fn swipe_defaults_can_be_cleared_via_update() {
        let mut p = valid_params();
        p.swipe_defaults = Some(SwipeDefaults {
            swipe_left: SwipeAction::Delete,
            swipe_right: SwipeAction::Archive,
            default_move_to: None,
        });
        let mut acct = Account::new(p).unwrap();
        assert!(acct.swipe_defaults().is_some());
        let mut up = valid_update_params();
        up.swipe_defaults = None;
        acct.update(up).unwrap();
        assert!(acct.swipe_defaults().is_none());
    }

    #[test]
    fn swipe_defaults_set_via_setter() {
        let mut acct = valid_account();
        acct.set_swipe_defaults(Some(SwipeDefaults {
            swipe_left: SwipeAction::MarkUnread,
            swipe_right: SwipeAction::None,
            default_move_to: Some("Inbox".into()),
        }));
        let sd = acct.swipe_defaults().unwrap();
        assert_eq!(sd.swipe_left, SwipeAction::MarkUnread);
        assert_eq!(sd.swipe_right, SwipeAction::None);
        assert_eq!(sd.default_move_to.as_deref(), Some("Inbox"));
        acct.set_swipe_defaults(None);
        assert!(acct.swipe_defaults().is_none());
    }

    #[test]
    fn swipe_defaults_serialization_roundtrip() {
        let mut p = valid_params();
        p.swipe_defaults = Some(SwipeDefaults {
            swipe_left: SwipeAction::Delete,
            swipe_right: SwipeAction::MoveToFolder("Spam".into()),
            default_move_to: Some("Archive".into()),
        });
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(acct.swipe_defaults(), restored.swipe_defaults());
    }

    #[test]
    fn deserialize_account_without_swipe_defaults_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("swipe_defaults");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.swipe_defaults().is_none());
    }

    #[test]
    fn swipe_action_default_is_none() {
        assert_eq!(SwipeAction::default(), SwipeAction::None);
    }

    #[test]
    fn swipe_action_display() {
        assert_eq!(SwipeAction::None.to_string(), "None");
        assert_eq!(SwipeAction::Archive.to_string(), "Archive");
        assert_eq!(SwipeAction::Delete.to_string(), "Delete");
        assert_eq!(SwipeAction::MarkRead.to_string(), "Mark as read");
        assert_eq!(SwipeAction::MarkUnread.to_string(), "Mark as unread");
        assert_eq!(
            SwipeAction::MoveToFolder("Spam".into()).to_string(),
            "Move to Spam"
        );
    }

    #[test]
    fn swipe_defaults_default_values() {
        let sd = SwipeDefaults::default();
        assert_eq!(sd.swipe_left, SwipeAction::None);
        assert_eq!(sd.swipe_right, SwipeAction::None);
        assert!(sd.default_move_to.is_none());
    }

    // -- Per-account notification tests (FR-39, FR-40, FR-41, AC-19) --

    #[test]
    fn notifications_enabled_defaults_to_true() {
        let acct = valid_account();
        assert!(acct.notifications_enabled());
    }

    #[test]
    fn notifications_enabled_can_be_set_on_creation() {
        let mut p = valid_params();
        p.notifications_enabled = false;
        let acct = Account::new(p).unwrap();
        assert!(!acct.notifications_enabled());
    }

    #[test]
    fn set_notifications_enabled_toggles() {
        let mut acct = valid_account();
        assert!(acct.notifications_enabled());
        acct.set_notifications_enabled(false);
        assert!(!acct.notifications_enabled());
        acct.set_notifications_enabled(true);
        assert!(acct.notifications_enabled());
    }

    #[test]
    fn notifications_independent_of_sync_enabled() {
        let mut acct = valid_account();
        acct.set_notifications_enabled(true);
        acct.set_sync_enabled(false);
        // Disabling sync must NOT change notifications (AC-19).
        assert!(acct.notifications_enabled());
        assert!(!acct.sync_enabled());

        acct.set_sync_enabled(true);
        // Re-enabling sync must NOT change notifications either.
        assert!(acct.notifications_enabled());
    }

    #[test]
    fn notifications_survive_sync_toggle_off_and_on() {
        let mut acct = valid_account();
        acct.set_notifications_enabled(false);
        acct.set_sync_enabled(false);
        acct.set_sync_enabled(true);
        // Notification setting survives sync toggling (AC-19).
        assert!(!acct.notifications_enabled());
    }

    #[test]
    fn notifications_changed_via_update() {
        let mut acct = valid_account();
        assert!(acct.notifications_enabled());
        let mut up = valid_update_params();
        up.notifications_enabled = false;
        acct.update(up).unwrap();
        assert!(!acct.notifications_enabled());
    }

    #[test]
    fn notifications_serialization_roundtrip() {
        let mut p = valid_params();
        p.notifications_enabled = false;
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert!(!restored.notifications_enabled());
    }

    #[test]
    fn deserialize_account_without_notifications_defaults_to_true() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut()
            .unwrap()
            .remove("notifications_enabled");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.notifications_enabled());
    }

    #[test]
    fn notification_channel_id_is_deterministic() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let ch1 = notification_channel_id(id);
        let ch2 = notification_channel_id(id);
        assert_eq!(ch1, ch2);
        assert_eq!(ch1, "account-550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn notification_channel_id_differs_per_account() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        assert_ne!(notification_channel_id(id1), notification_channel_id(id2));
    }

    // -- Quota display tests (FR-42, FR-43, AC-17) --

    #[test]
    fn quota_defaults_to_none() {
        let acct = valid_account();
        assert!(acct.quota().is_none());
    }

    #[test]
    fn quota_new_returns_none_for_zero_limit() {
        assert!(QuotaInfo::new(100, 0).is_none());
    }

    #[test]
    fn quota_new_returns_some_for_valid_values() {
        let q = QuotaInfo::new(500, 1000).unwrap();
        assert_eq!(q.used_bytes, 500);
        assert_eq!(q.limit_bytes, 1000);
    }

    #[test]
    fn quota_usage_percent() {
        let q = QuotaInfo::new(750, 1000).unwrap();
        assert!((q.usage_percent() - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn quota_usage_percent_full() {
        let q = QuotaInfo::new(1000, 1000).unwrap();
        assert!((q.usage_percent() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn quota_usage_percent_over() {
        let q = QuotaInfo::new(1100, 1000).unwrap();
        assert!(q.usage_percent() > 100.0);
    }

    #[test]
    fn quota_is_high_usage_below_threshold() {
        let q = QuotaInfo::new(940, 1000).unwrap();
        assert!(!q.is_high_usage());
    }

    #[test]
    fn quota_is_high_usage_at_threshold() {
        let q = QuotaInfo::new(950, 1000).unwrap();
        assert!(q.is_high_usage());
    }

    #[test]
    fn quota_is_high_usage_above_threshold() {
        let q = QuotaInfo::new(990, 1000).unwrap();
        assert!(q.is_high_usage());
    }

    #[test]
    fn quota_set_and_get() {
        let mut acct = valid_account();
        let q = QuotaInfo::new(500_000_000, 1_000_000_000).unwrap();
        acct.set_quota(Some(q));
        assert_eq!(acct.quota(), Some(q));
    }

    #[test]
    fn quota_can_be_cleared() {
        let mut acct = valid_account();
        let q = QuotaInfo::new(100, 200).unwrap();
        acct.set_quota(Some(q));
        assert!(acct.quota().is_some());
        acct.set_quota(None);
        assert!(acct.quota().is_none());
    }

    #[test]
    fn quota_serialization_roundtrip() {
        let mut acct = valid_account();
        let q = QuotaInfo::new(750_000_000, 1_000_000_000).unwrap();
        acct.set_quota(Some(q));
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.quota(), Some(q));
    }

    #[test]
    fn deserialize_account_without_quota_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("quota");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.quota().is_none());
    }

    #[test]
    fn quota_preserved_across_update() {
        let mut acct = valid_account();
        let q = QuotaInfo::new(800, 1000).unwrap();
        acct.set_quota(Some(q));
        acct.update(valid_update_params()).unwrap();
        // Quota is server-reported; update() does not touch it.
        assert_eq!(acct.quota(), Some(q));
    }

    #[test]
    fn quota_format_bytes_bytes() {
        assert_eq!(QuotaInfo::format_bytes(512), "512 B");
    }

    #[test]
    fn quota_format_bytes_kb() {
        assert_eq!(QuotaInfo::format_bytes(2048), "2 KB");
    }

    #[test]
    fn quota_format_bytes_mb() {
        assert_eq!(QuotaInfo::format_bytes(5_242_880), "5.0 MB");
    }

    #[test]
    fn quota_format_bytes_gb() {
        assert_eq!(QuotaInfo::format_bytes(1_073_741_824), "1.00 GB");
    }

    // -- Advanced connection security settings tests (FR-4, FR-53, US-8, US-9, US-10) --

    #[test]
    fn security_settings_defaults_to_none() {
        let acct = valid_account();
        assert!(acct.security_settings().is_none());
    }

    #[test]
    fn security_settings_can_be_set_on_creation() {
        let mut p = valid_params();
        p.security_settings = Some(SecuritySettings {
            dnssec: true,
            dane: true,
            insecure: false,
            certificate_fingerprint: Some("aa:bb:cc".into()),
            client_certificate: Some("/path/to/cert".into()),
            auth_realm: Some("realm.example.com".into()),
            allow_insecure_auth: false,
            max_tls_version: None,
            disable_ip_connections: false,
        });
        let acct = Account::new(p).unwrap();
        let sec = acct.security_settings().unwrap();
        assert!(sec.dnssec);
        assert!(sec.dane);
        assert!(!sec.insecure);
        assert_eq!(sec.certificate_fingerprint.as_deref(), Some("aa:bb:cc"));
        assert_eq!(sec.client_certificate.as_deref(), Some("/path/to/cert"));
        assert_eq!(sec.auth_realm.as_deref(), Some("realm.example.com"));
    }

    #[test]
    fn security_settings_can_be_changed_via_update() {
        let mut acct = valid_account();
        assert!(acct.security_settings().is_none());
        let mut up = valid_update_params();
        up.security_settings = Some(SecuritySettings {
            dnssec: false,
            dane: false,
            insecure: true,
            certificate_fingerprint: None,
            client_certificate: None,
            auth_realm: None,
            allow_insecure_auth: false,
            max_tls_version: None,
            disable_ip_connections: false,
        });
        acct.update(up).unwrap();
        let sec = acct.security_settings().unwrap();
        assert!(sec.insecure);
        assert!(!sec.dnssec);
    }

    #[test]
    fn security_settings_can_be_cleared_via_update() {
        let mut p = valid_params();
        p.security_settings = Some(SecuritySettings {
            dnssec: true,
            ..Default::default()
        });
        let mut acct = Account::new(p).unwrap();
        assert!(acct.security_settings().is_some());
        let mut up = valid_update_params();
        up.security_settings = None;
        acct.update(up).unwrap();
        assert!(acct.security_settings().is_none());
    }

    #[test]
    fn security_settings_set_via_setter() {
        let mut acct = valid_account();
        acct.set_security_settings(Some(SecuritySettings {
            dane: true,
            ..Default::default()
        }));
        assert!(acct.security_settings().unwrap().dane);
        acct.set_security_settings(None);
        assert!(acct.security_settings().is_none());
    }

    #[test]
    fn security_settings_serialization_roundtrip() {
        let mut p = valid_params();
        p.security_settings = Some(SecuritySettings {
            dnssec: true,
            dane: true,
            insecure: false,
            certificate_fingerprint: Some("de:ad:be:ef".into()),
            client_certificate: Some("/etc/ssl/client.pem".into()),
            auth_realm: Some("mail.example.com".into()),
            allow_insecure_auth: false,
            max_tls_version: None,
            disable_ip_connections: false,
        });
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(acct.security_settings(), restored.security_settings());
    }

    #[test]
    fn deserialize_account_without_security_settings_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("security_settings");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.security_settings().is_none());
    }

    #[test]
    fn security_settings_per_account_isolation() {
        let mut p1 = valid_params();
        p1.security_settings = Some(SecuritySettings {
            insecure: true,
            ..Default::default()
        });
        let a1 = Account::new(p1).unwrap();

        let a2 = valid_account();

        // Insecure flag on a1 does not affect a2.
        assert!(a1.security_settings().unwrap().insecure);
        assert!(a2.security_settings().is_none());
    }

    #[test]
    fn security_settings_default_struct_all_disabled() {
        let s = SecuritySettings::default();
        assert!(!s.dnssec);
        assert!(!s.dane);
        assert!(!s.insecure);
        assert!(s.certificate_fingerprint.is_none());
        assert!(s.client_certificate.is_none());
        assert!(s.auth_realm.is_none());
    }

    // -- FetchSettings tests (FR-51) --

    #[test]
    fn fetch_settings_default_none() {
        let acct = valid_account();
        assert!(acct.fetch_settings().is_none());
    }

    #[test]
    fn fetch_settings_default_struct_all_off() {
        let s = FetchSettings::default();
        assert!(!s.partial_fetch);
        assert!(!s.raw_fetch);
        assert!(!s.ignore_size_limits);
        assert_eq!(s.date_header_preference, DateHeaderPreference::ServerTime);
        assert!(!s.utf8_support);
    }

    #[test]
    fn fetch_settings_creation() {
        let mut p = valid_params();
        p.fetch_settings = Some(FetchSettings {
            partial_fetch: true,
            raw_fetch: false,
            ignore_size_limits: true,
            date_header_preference: DateHeaderPreference::ReceivedHeader,
            utf8_support: true,
            ..Default::default()
        });
        let acct = Account::new(p).unwrap();
        let fs = acct.fetch_settings().unwrap();
        assert!(fs.partial_fetch);
        assert!(!fs.raw_fetch);
        assert!(fs.ignore_size_limits);
        assert_eq!(
            fs.date_header_preference,
            DateHeaderPreference::ReceivedHeader
        );
        assert!(fs.utf8_support);
    }

    #[test]
    fn fetch_settings_setter() {
        let mut acct = valid_account();
        acct.set_fetch_settings(Some(FetchSettings {
            raw_fetch: true,
            ..Default::default()
        }));
        assert!(acct.fetch_settings().unwrap().raw_fetch);
        acct.set_fetch_settings(None);
        assert!(acct.fetch_settings().is_none());
    }

    #[test]
    fn fetch_settings_update() {
        let mut acct = valid_account();
        let mut up = valid_update_params();
        up.fetch_settings = Some(FetchSettings {
            partial_fetch: true,
            date_header_preference: DateHeaderPreference::DateHeader,
            ..Default::default()
        });
        acct.update(up).unwrap();
        let fs = acct.fetch_settings().unwrap();
        assert!(fs.partial_fetch);
        assert_eq!(fs.date_header_preference, DateHeaderPreference::DateHeader);
    }

    #[test]
    fn fetch_settings_can_be_cleared_via_update() {
        let mut p = valid_params();
        p.fetch_settings = Some(FetchSettings {
            partial_fetch: true,
            ..Default::default()
        });
        let mut acct = Account::new(p).unwrap();
        assert!(acct.fetch_settings().is_some());
        let mut up = valid_update_params();
        up.fetch_settings = None;
        acct.update(up).unwrap();
        assert!(acct.fetch_settings().is_none());
    }

    #[test]
    fn fetch_settings_serialization_roundtrip() {
        let mut p = valid_params();
        p.fetch_settings = Some(FetchSettings {
            partial_fetch: true,
            raw_fetch: true,
            ignore_size_limits: true,
            date_header_preference: DateHeaderPreference::ReceivedHeader,
            utf8_support: true,
            ..Default::default()
        });
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(acct.fetch_settings(), restored.fetch_settings());
    }

    #[test]
    fn deserialize_account_without_fetch_settings_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("fetch_settings");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.fetch_settings().is_none());
    }

    #[test]
    fn fetch_settings_per_account_isolation() {
        let mut p1 = valid_params();
        p1.fetch_settings = Some(FetchSettings {
            partial_fetch: true,
            ..Default::default()
        });
        let a1 = Account::new(p1).unwrap();
        let a2 = valid_account();
        assert!(a1.fetch_settings().unwrap().partial_fetch);
        assert!(a2.fetch_settings().is_none());
    }

    // -- KeepAliveSettings tests (FR-52) --

    #[test]
    fn keep_alive_settings_default_none() {
        let acct = valid_account();
        assert!(acct.keep_alive_settings().is_none());
    }

    #[test]
    fn keep_alive_settings_default_struct_all_off() {
        let s = KeepAliveSettings::default();
        assert!(!s.use_noop_instead_of_idle);
    }

    #[test]
    fn keep_alive_settings_creation() {
        let mut p = valid_params();
        p.keep_alive_settings = Some(KeepAliveSettings {
            use_noop_instead_of_idle: true,
        });
        let acct = Account::new(p).unwrap();
        assert!(acct.keep_alive_settings().unwrap().use_noop_instead_of_idle);
    }

    #[test]
    fn keep_alive_settings_setter() {
        let mut acct = valid_account();
        acct.set_keep_alive_settings(Some(KeepAliveSettings {
            use_noop_instead_of_idle: true,
        }));
        assert!(acct.keep_alive_settings().unwrap().use_noop_instead_of_idle);
        acct.set_keep_alive_settings(None);
        assert!(acct.keep_alive_settings().is_none());
    }

    #[test]
    fn keep_alive_settings_update() {
        let mut acct = valid_account();
        let mut up = valid_update_params();
        up.keep_alive_settings = Some(KeepAliveSettings {
            use_noop_instead_of_idle: true,
        });
        acct.update(up).unwrap();
        assert!(acct.keep_alive_settings().unwrap().use_noop_instead_of_idle);
    }

    #[test]
    fn keep_alive_settings_can_be_cleared_via_update() {
        let mut p = valid_params();
        p.keep_alive_settings = Some(KeepAliveSettings {
            use_noop_instead_of_idle: true,
        });
        let mut acct = Account::new(p).unwrap();
        assert!(acct.keep_alive_settings().is_some());
        let mut up = valid_update_params();
        up.keep_alive_settings = None;
        acct.update(up).unwrap();
        assert!(acct.keep_alive_settings().is_none());
    }

    #[test]
    fn keep_alive_settings_serialization_roundtrip() {
        let mut p = valid_params();
        p.keep_alive_settings = Some(KeepAliveSettings {
            use_noop_instead_of_idle: true,
        });
        let acct = Account::new(p).unwrap();
        let json = serde_json::to_string(&acct).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(acct.keep_alive_settings(), restored.keep_alive_settings());
    }

    #[test]
    fn deserialize_account_without_keep_alive_settings_defaults_to_none() {
        let acct = valid_account();
        let mut json: serde_json::Value = serde_json::to_value(&acct).unwrap();
        json.as_object_mut().unwrap().remove("keep_alive_settings");
        let restored: Account = serde_json::from_value(json).unwrap();
        assert!(restored.keep_alive_settings().is_none());
    }

    #[test]
    fn keep_alive_settings_per_account_isolation() {
        let mut p1 = valid_params();
        p1.keep_alive_settings = Some(KeepAliveSettings {
            use_noop_instead_of_idle: true,
        });
        let a1 = Account::new(p1).unwrap();
        let a2 = valid_account();
        assert!(a1.keep_alive_settings().unwrap().use_noop_instead_of_idle);
        assert!(a2.keep_alive_settings().is_none());
    }

    #[test]
    fn date_header_preference_display() {
        assert_eq!(DateHeaderPreference::ServerTime.to_string(), "Server time");
        assert_eq!(DateHeaderPreference::DateHeader.to_string(), "Date header");
        assert_eq!(
            DateHeaderPreference::ReceivedHeader.to_string(),
            "Received header"
        );
    }

    #[test]
    fn duplicate_preserves_fetch_and_keep_alive_settings() {
        let mut p = valid_params();
        p.fetch_settings = Some(FetchSettings {
            partial_fetch: true,
            utf8_support: true,
            ..Default::default()
        });
        p.keep_alive_settings = Some(KeepAliveSettings {
            use_noop_instead_of_idle: true,
        });
        let acct = Account::new(p).unwrap();
        let dup_params = acct.to_new_account_params();
        assert_eq!(dup_params.fetch_settings, acct.fetch_settings().cloned());
        assert_eq!(
            dup_params.keep_alive_settings,
            acct.keep_alive_settings().cloned()
        );
    }

    // -- Shared mailbox tests (FR-40, N-8) --

    #[test]
    fn encode_shared_mailbox_username_with_shared() {
        let result = encode_shared_mailbox_username("user@contoso.com", Some("shared@contoso.com"));
        assert_eq!(result, "shared@contoso.com\\user@contoso.com");
    }

    #[test]
    fn encode_shared_mailbox_username_without_shared() {
        let result = encode_shared_mailbox_username("user@contoso.com", None);
        assert_eq!(result, "user@contoso.com");
    }

    #[test]
    fn encode_shared_mailbox_username_empty_shared() {
        let result = encode_shared_mailbox_username("user@contoso.com", Some(""));
        assert_eq!(result, "user@contoso.com");
    }

    #[test]
    fn encode_shared_mailbox_username_whitespace_shared() {
        let result = encode_shared_mailbox_username("user@contoso.com", Some("   "));
        assert_eq!(result, "user@contoso.com");
    }

    #[test]
    fn encode_shared_mailbox_username_trims_shared() {
        let result =
            encode_shared_mailbox_username("user@contoso.com", Some("  shared@contoso.com  "));
        assert_eq!(result, "shared@contoso.com\\user@contoso.com");
    }

    #[test]
    fn effective_username_without_shared_mailbox() {
        let acct = valid_account();
        assert_eq!(acct.effective_username(), "user@example.com");
    }

    #[test]
    fn effective_username_with_shared_mailbox() {
        let mut p = valid_params();
        p.shared_mailbox = Some("shared@example.com".into());
        let acct = Account::new(p).unwrap();
        assert_eq!(
            acct.effective_username(),
            "shared@example.com\\user@example.com"
        );
    }

    #[test]
    fn shared_mailbox_preserved_in_to_new_account_params() {
        let mut p = valid_params();
        p.shared_mailbox = Some("shared@example.com".into());
        let acct = Account::new(p).unwrap();
        let dup = acct.to_new_account_params();
        assert_eq!(dup.shared_mailbox, Some("shared@example.com".into()));
    }

    #[test]
    fn shared_mailbox_updated_via_update() {
        let mut acct = valid_account();
        assert!(acct.shared_mailbox().is_none());
        let mut up = valid_update_params();
        up.shared_mailbox = Some("shared@example.com".into());
        acct.update(up).unwrap();
        assert_eq!(acct.shared_mailbox(), Some("shared@example.com"));
    }

    // -- switch_auth_type tests (FR-30, US-17) --

    fn oauth_account_with_smtp() -> Account {
        let mut p = valid_params();
        p.auth_method = AuthMethod::OAuth2;
        p.credential = "access-token".into();
        p.smtp = Some(SmtpConfig {
            host: "smtp.example.com".into(),
            port: 465,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::OAuth2,
            username: "user@example.com".into(),
            credential: "access-token".into(),
        });
        Account::new(p).unwrap()
    }

    fn password_account_with_smtp() -> Account {
        let mut p = valid_params();
        p.auth_method = AuthMethod::Plain;
        p.credential = "password123".into();
        p.smtp = Some(SmtpConfig {
            host: "smtp.example.com".into(),
            port: 465,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "password123".into(),
        });
        Account::new(p).unwrap()
    }

    #[test]
    fn switch_auth_type_oauth_to_password() {
        let mut acct = oauth_account_with_smtp();
        assert_eq!(acct.auth_method(), AuthMethod::OAuth2);

        acct.switch_auth_type("new-password".into(), AuthMethod::Plain);

        assert_eq!(acct.auth_method(), AuthMethod::Plain);
        assert_eq!(acct.credential(), "new-password");
        let smtp = acct.smtp().unwrap();
        assert_eq!(smtp.auth_method, AuthMethod::Plain);
        assert_eq!(smtp.credential, "new-password");
    }

    #[test]
    fn switch_auth_type_password_to_oauth() {
        let mut acct = password_account_with_smtp();
        assert_eq!(acct.auth_method(), AuthMethod::Plain);

        acct.switch_auth_type("oauth-token".into(), AuthMethod::OAuth2);

        assert_eq!(acct.auth_method(), AuthMethod::OAuth2);
        assert_eq!(acct.credential(), "oauth-token");
        let smtp = acct.smtp().unwrap();
        assert_eq!(smtp.auth_method, AuthMethod::OAuth2);
        assert_eq!(smtp.credential, "oauth-token");
    }

    #[test]
    fn switch_auth_type_preserves_all_other_properties() {
        let mut acct = oauth_account_with_smtp();
        let original_id = acct.id();
        let original_host = acct.host().to_string();
        let original_username = acct.username().to_string();
        let original_display_name = acct.display_name().to_string();
        let original_smtp_host = acct.smtp().unwrap().host.clone();

        acct.switch_auth_type("new-password".into(), AuthMethod::Plain);

        assert_eq!(acct.id(), original_id);
        assert_eq!(acct.host(), original_host);
        assert_eq!(acct.username(), original_username);
        assert_eq!(acct.display_name(), original_display_name);
        assert_eq!(acct.smtp().unwrap().host, original_smtp_host);
        assert_eq!(acct.smtp().unwrap().username, original_username);
    }

    #[test]
    fn switch_auth_type_without_smtp_config() {
        let mut acct = valid_account(); // no SMTP config
        assert!(acct.smtp().is_none());

        acct.switch_auth_type("new-cred".into(), AuthMethod::OAuth2);

        assert_eq!(acct.auth_method(), AuthMethod::OAuth2);
        assert_eq!(acct.credential(), "new-cred");
        assert!(acct.smtp().is_none()); // still none, no panic
    }

    #[test]
    fn certificate_auth_method_display() {
        assert_eq!(format!("{}", AuthMethod::Certificate), "Certificate");
    }

    #[test]
    fn certificate_auth_method_serde_roundtrip() {
        let method = AuthMethod::Certificate;
        let json = serde_json::to_string(&method).unwrap();
        let deserialized: AuthMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AuthMethod::Certificate);
    }

    #[test]
    fn create_account_with_certificate_auth() {
        let mut p = valid_params();
        p.auth_method = AuthMethod::Certificate;
        p.security_settings = Some(SecuritySettings {
            client_certificate: Some("/etc/pki/client.p12".into()),
            ..Default::default()
        });
        let acct = Account::new(p).unwrap();
        assert_eq!(acct.auth_method(), AuthMethod::Certificate);
        assert_eq!(
            acct.security_settings()
                .and_then(|s| s.client_certificate.clone()),
            Some("/etc/pki/client.p12".into())
        );
    }

    #[test]
    fn clear_client_certificate_removes_reference() {
        let mut sec = SecuritySettings {
            client_certificate: Some("/etc/pki/client.p12".into()),
            ..Default::default()
        };
        assert!(sec.clear_client_certificate());
        assert!(sec.client_certificate.is_none());
        // Second clear returns false (was already None).
        assert!(!sec.clear_client_certificate());
    }

    #[test]
    fn clear_client_certificate_makes_settings_empty() {
        let mut sec = SecuritySettings {
            client_certificate: Some("/path/to/cert.p12".into()),
            ..Default::default()
        };
        assert!(!sec.is_empty());
        sec.clear_client_certificate();
        assert!(sec.is_empty());
    }

    #[test]
    fn clear_certificate_and_revert_to_password_auth() {
        // Create account with certificate auth.
        let mut p = valid_params();
        p.auth_method = AuthMethod::Certificate;
        p.security_settings = Some(SecuritySettings {
            client_certificate: Some("/etc/pki/client.p12".into()),
            ..Default::default()
        });
        let mut acct = Account::new(p).unwrap();
        assert_eq!(acct.auth_method(), AuthMethod::Certificate);

        // User clears certificate and switches to password auth.
        let mut up = valid_update_params();
        up.auth_method = AuthMethod::Plain;
        up.security_settings = None; // certificate cleared, no other security settings
        acct.update(up).unwrap();

        assert_eq!(acct.auth_method(), AuthMethod::Plain);
        assert!(acct.security_settings().is_none());
    }

    #[test]
    fn clear_certificate_and_revert_to_oauth() {
        // Create account with certificate auth.
        let mut p = valid_params();
        p.auth_method = AuthMethod::Certificate;
        p.security_settings = Some(SecuritySettings {
            client_certificate: Some("/etc/pki/client.p12".into()),
            ..Default::default()
        });
        let mut acct = Account::new(p).unwrap();

        // User clears certificate and switches to OAuth2.
        let mut up = valid_update_params();
        up.auth_method = AuthMethod::OAuth2;
        up.security_settings = None;
        acct.update(up).unwrap();

        assert_eq!(acct.auth_method(), AuthMethod::OAuth2);
        assert!(acct
            .security_settings()
            .and_then(|s| s.client_certificate.clone())
            .is_none());
    }

    #[test]
    fn security_settings_is_empty_when_default() {
        let sec = SecuritySettings::default();
        assert!(sec.is_empty());
    }

    #[test]
    fn security_settings_not_empty_with_other_fields() {
        let mut sec = SecuritySettings {
            client_certificate: Some("/path.p12".into()),
            dane: true,
            ..Default::default()
        };
        sec.clear_client_certificate();
        // Still not empty because dane is set.
        assert!(!sec.is_empty());
    }
}
