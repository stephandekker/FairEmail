pub mod account;
pub mod account_creation;
pub mod account_order;
pub mod account_review;
pub mod auth_conversion;
pub mod auth_error;
pub mod auth_mechanism;
pub mod auto_config;
pub(crate) mod browser_selection;
pub mod certificate;
pub mod connection_log;
pub mod connection_state;
pub mod connection_test;
pub mod content_store;
pub mod credential_store;
pub mod delete_account;
pub(crate) mod detection_failure;
pub(crate) mod detection_progress;
pub mod dns_discovery;
pub mod duplicate_account;
pub mod ehlo;
pub mod export_accounts;
pub mod field_validation;
pub mod folder_setup;
pub(crate) mod idle_manager;
pub mod imap_check;
pub mod import_accounts;
pub mod inbound_test;
pub mod inbound_test_diagnostics;
#[allow(dead_code)]
pub(crate) mod ispdb_discovery;
pub mod message;
pub mod navigation;
#[allow(dead_code)]
pub(crate) mod oauth_browser_flow;
pub(crate) mod oauth_flow;
pub(crate) mod oauth_identity;
pub mod oauth_provider_registry;
pub(crate) mod oauth_signin;
pub(crate) mod oauth_wizard;
pub mod password_propagation;
pub mod pending_operation;
pub mod port_autofill;
#[allow(dead_code)]
pub(crate) mod port_scanning;
pub mod primary;
pub(crate) mod privacy;
pub mod proprietary_provider;
pub mod provider;
pub(crate) mod provider_data;
pub mod provider_dropdown;
pub mod reauth;
#[allow(dead_code)]
pub(crate) mod revocation;
pub mod save_auto_test;
pub mod smtp_check;
pub mod smtp_identity;
pub mod smtp_test_diagnostics;
pub mod sync_conditions;
pub mod sync_event;
pub mod sync_state;
#[allow(dead_code)]
pub(crate) mod token_refresh;
pub mod user_provider_file;
#[allow(dead_code)]
pub(crate) mod vendor_discovery;
pub mod wizard_validation;
pub(crate) mod xoauth2;

pub use account::{
    collect_categories, detect_system_folders, encode_shared_mailbox_username,
    notification_channel_id, resolve_color, Account, AccountColor, AccountValidationError,
    AuthMethod, DateHeaderPreference, EncryptionMode, FetchSettings, Folder, FolderRole,
    KeepAliveSettings, NewAccountParams, Pop3Settings, Protocol, QuotaInfo, SecuritySettings,
    SmtpConfig, SwipeAction, SwipeDefaults, SystemFolders, UpdateAccountParams,
    QUOTA_HIGH_THRESHOLD_PERCENT,
};
pub use account_creation::{
    create_account_and_identity, AccountCreationParams, AccountCreationResult, SendingIdentity,
};
pub use account_order::{apply_custom_order, default_order, move_account, order_from_accounts};
pub use account_review::{build_review_data, AccountReviewData, ReviewFolderEntry};
pub use auth_conversion::{can_convert_to_password, find_oauth_config_for_conversion};
pub use auth_error::{
    app_password_hint_text, build_provider_hint, is_outlook_domain, outlook_documentation_url,
    present_connectivity_error, present_imap_error, present_smtp_error, AuthErrorKind,
    AuthErrorPresentation, ProviderHint, GENERAL_SUPPORT_URL,
};
pub use auth_mechanism::{
    log_negotiation, negotiate_password_mechanism, parse_imap_capabilities, parse_pop3_capa,
    parse_smtp_ehlo, supported_mechanisms, AuthMechanism, AuthProtocol,
};
pub use browser_selection::{
    detect_installed_browsers, select_browser, BrowserInfo, BrowserSelectionResult,
};
pub use certificate::{CertificateDecision, CertificateInfo};
pub use connection_log::{ConnectionLogEventType, ConnectionLogRecord};
pub use connection_state::{
    format_log_timestamp, ConnectionLogEntry, ConnectionState, ConnectionStateManager,
};
pub use connection_test::{
    ConnectionTestError, ConnectionTestRequest, ConnectionTestResult, ServerTestOutcome,
};
pub use content_store::{ContentStore, ContentStoreError};
pub use credential_store::{
    identity_credential_uuid, CredentialError, CredentialRole, CredentialStore, SecretValue,
};
pub use delete_account::{clear_primary_if_deleted, remove_from_order};
pub use dns_discovery::{discover_by_dns, DnsDiscoveryResult, DnsError, DnsResolver, SrvRecord};
pub use duplicate_account::duplicate as duplicate_account;
pub use ehlo::resolve_ehlo_hostname;
pub use export_accounts::{
    export_accounts, EncryptedEnvelope, ExportCategory, ExportEnvelope, ExportError, ExportOptions,
    ExportedAccount,
};
pub use folder_setup::{
    build_default_sync_configs, complete_system_folders, default_folder_name,
    find_missing_system_folders, FolderSetupError, FolderSetupResult, FolderSyncConfig, PushMode,
    SyncMode,
};
pub use imap_check::{
    build_imap_success, detect_folder_role, resolve_username_candidates, ImapCheckError,
    ImapCheckResult, ImapCheckSuccess, ImapConnectionParams, ImapFolder, UsernameCandidate,
};
pub use import_accounts::{
    import_accounts, parse_import_data, AccountImportOutcome, DuplicateStrategy, ImportError,
    ImportOptions, ImportResult,
};
pub use inbound_test::{
    InboundTestError, InboundTestParams, InboundTestResult, InboundTestSuccess,
    INBOUND_TEST_TIMEOUT,
};
pub use inbound_test_diagnostics::{diagnose_error, ConnectionDiagnostic};
pub use ispdb_discovery::AutoconfigError;
pub use message::{
    derive_body_text, flags_from_imap, parse_raw_message, Message, NewMessage, FLAG_ANSWERED,
    FLAG_DELETED, FLAG_DRAFT, FLAG_FLAGGED, FLAG_SEEN,
};
pub use navigation::{group_by_category, sort_accounts_flat, CategoryGroup};
#[allow(unused_imports)]
pub(crate) use oauth_browser_flow::{
    privacy_policy_url, run_browser_authorization_flow, BrowserAuthParams, BrowserAuthResult,
};
pub use oauth_flow::{
    compute_expiry_epoch, decode_jump_state_port, generate_state, parse_callback_query,
    parse_token_response_json, validate_token_response, CallbackParams, OAuthFlowError,
    OAuthSession, PkceChallenge, RedirectMethod, TokenExchangeParams, TokenResponse,
    ValidatedTokenResponse, DEFAULT_JUMP_PAGE_URL, SESSION_TIMEOUT,
};
pub use oauth_identity::{
    extract_identity_from_id_token, needs_userinfo_fetch, parse_userinfo_json, resolve_identity,
    userinfo_url, validate_userinfo_response, IdentityExtractionResult, IdentityInfo,
};
pub use oauth_provider_registry::{OAuthProviderEntry, OAuthProviderRegistry};
pub use oauth_signin::{
    determine_auth_options, has_oauth_credentials, is_oauth_provider,
    oauth_credentials_file_exists, oauth_unavailable_message, resolve_auth_from_choice, AuthChoice,
    AuthOptions, OAuthTokenResult, OAuthUnavailableReason, OAUTH_CREDENTIALS_FILENAME,
};
pub use oauth_wizard::{
    build_oauth_connection_error, create_oauth_account, derive_oauth_username, OAuthConnectionError,
};
pub use password_propagation::{password_has_changed, propagate_password_to_identities};
pub use pending_operation::{
    OperationKind, OperationState, PendingOperation, SendPayload, StoreFlagsPayload,
};
pub use port_autofill::{default_port, should_autofill, smtp_default_port};
pub use primary::{
    auto_designate_on_add, revoke_if_sync_disabled, set_primary, PrimaryDesignationError,
};
pub use provider::{
    merge_network_with_bundled, LocalizedDoc, MatchScore, MaxTlsVersion, OAuthConfig, Provider,
    ProviderCandidate, ProviderDatabase, ProviderEncryption, ServerConfig, UsernameType,
};
pub use provider_dropdown::{
    build_dropdown_entries, index_for_provider_id, prefill_for_provider, provider_guidance,
    ProviderDropdownEntry, ProviderPrefill,
};
pub use reauth::{
    find_matching_account, find_oauth_config_for_reauth, reauthorize_account, ReauthError,
    ReauthParams,
};
#[allow(unused_imports)]
pub(crate) use revocation::{
    build_revocation_notification, build_revocation_notification_with_hint,
    revocation_event_from_outcome, RevocationEvent, RevocationNotification,
};
pub use smtp_check::{
    combine_connectivity_results, ConnectivityCheckError, ConnectivityCheckResult, SmtpCheckError,
    SmtpCheckResult, SmtpCheckSuccess, SmtpConnectionParams,
};
pub use smtp_identity::{
    validate_smtp_identity, SmtpIdentityFieldError, SmtpIdentityParams,
    SmtpIdentityValidationResult,
};
pub use smtp_test_diagnostics::diagnose_smtp_error;
pub use sync_conditions::{
    evaluate as evaluate_sync_conditions, EnvironmentStatus, SyncEligibility, SyncPauseReason,
};
pub use sync_event::SyncEvent;
pub use sync_state::SyncState;
pub use user_provider_file::{
    build_merged_database, merge_user_providers, parse_and_validate_provider_file,
    parse_user_provider_file, validate_provider_configs, ProviderValidationError,
    UserProviderFileError, APP_CONFIG_DIR, USER_PROVIDER_FILENAME,
};
