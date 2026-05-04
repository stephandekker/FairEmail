use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}

impl Default for Pop3Settings {
    fn default() -> Self {
        Self {
            leave_on_server: true,
            delete_from_server_when_deleted_on_device: false,
            keep_on_device_when_deleted_from_server: true,
            max_messages_to_download: None,
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
        Ok(())
    }
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
            }),
            color: None,
            avatar_path: None,
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
}
