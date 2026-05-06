use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use glib::clone;

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::account_review::{build_review_data, AccountReviewData};
use crate::core::detection_failure::build_detection_failure_fallback;
use crate::core::detection_progress::{detection_sequence, DetectionStep};
use crate::core::oauth_flow::OAuthSession;
use crate::core::oauth_signin::{
    determine_auth_options, oauth_unavailable_message, OAuthUnavailableReason,
};
use crate::core::oauth_wizard::{build_oauth_connection_error, create_oauth_account};
use crate::core::privacy;
use crate::core::proprietary_provider::check_proprietary_provider;
use crate::core::provider::ProviderDatabase;
use crate::core::user_provider_file::build_merged_database;
use crate::core::wizard_validation::{
    validate_wizard_fields, WizardFieldError, WizardValidationResult,
};
use crate::services::network::is_network_available;
use crate::services::oauth_service;

/// Result passed back from the wizard: the validated name, email, and password,
/// or `None` if the user cancelled / closed the dialog.
pub(crate) type WizardResult = Option<WizardAction>;

/// The action chosen in the wizard: auto-detect, manual setup, or re-authorize (FR-35, FR-32).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum WizardAction {
    /// User clicked "Check" — proceed with auto-detection.
    Check(WizardData),
    /// User clicked "Manual setup" — open manual configuration (US-26).
    ManualSetup(WizardData),
    /// User clicked "Authorize existing account again" — re-authorize flow (FR-32).
    Reauthorize(WizardData),
    /// OAuth flow completed and account was created (FR-21, FR-23).
    OAuthComplete(WizardData),
}

/// Validated wizard data ready for the next step (provider detection / account creation).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct WizardData {
    pub display_name: String,
    pub email: String,
    pub password: String,
}

/// Messages sent from the OAuth background thread to the UI thread.
enum OAuthMessage {
    Progress(String),
    Error(String),
    TokensReceived {
        access_token: String,
        refresh_token: String,
        expires_in: Option<u64>,
    },
}

/// Result of connection testing after OAuth token acquisition.
struct ConnectionTestMessage {
    imap_result: crate::core::imap_check::ImapCheckResult,
    smtp_result: crate::core::smtp_check::SmtpCheckResult,
}

/// Intermediate state held between OAuth token acquisition and account creation.
/// Stored in an `Rc<RefCell<...>>` so it can be shared between async steps and
/// the review confirm button handler.
#[allow(dead_code)]
struct OAuthSetupState {
    provider: crate::core::provider::Provider,
    email: String,
    display_name: String,
    access_token: String,
    refresh_token: String,
    expires_in: Option<u64>,
    imap_result: crate::core::imap_check::ImapCheckSuccess,
    smtp_result: crate::core::smtp_check::SmtpCheckSuccess,
    oauth_tenant: Option<String>,
    shared_mailbox: Option<String>,
}

/// Build and present the quick-setup wizard dialog (FR-1, FR-2, FR-3).
///
/// `on_done` is called when the user finishes or closes the wizard.
pub(crate) fn show(parent: &adw::ApplicationWindow, on_done: impl Fn(WizardResult) + 'static) {
    let dialog = adw::Dialog::builder()
        .title(gettextrs::gettext("Set up account"))
        .content_width(420)
        .content_height(460)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let toast_overlay = adw::ToastOverlay::new();

    let scrolled = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();

    let clamp = adw::Clamp::builder()
        .maximum_size(460)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);

    // -- Display name (FR-3) --
    let name_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Your name"))
        .build();
    let name_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Display name"))
        .build();
    name_row.set_tooltip_text(Some(&gettextrs::gettext(
        "The name shown to recipients of your emails",
    )));
    name_group.add(&name_row);
    // Field-specific error label (hidden by default).
    let name_error = gtk::Label::builder()
        .css_classes(["error", "caption"])
        .halign(gtk::Align::Start)
        .margin_start(12)
        .visible(false)
        .build();
    name_error.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Display name error",
    ))]);
    vbox.append(&name_group);
    vbox.append(&name_error);

    // -- Email address (FR-3) --
    let email_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Email"))
        .build();
    let email_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Email address"))
        .input_purpose(gtk::InputPurpose::Email)
        .build();
    email_group.add(&email_row);
    let email_error = gtk::Label::builder()
        .css_classes(["error", "caption"])
        .halign(gtk::Align::Start)
        .margin_start(12)
        .visible(false)
        .build();
    email_error.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Email error",
    ))]);
    vbox.append(&email_group);
    vbox.append(&email_error);

    // -- Password (FR-3, FR-4) --
    let password_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Password"))
        .build();
    let password_row = adw::PasswordEntryRow::builder()
        .title(gettextrs::gettext("Password"))
        .build();
    password_group.add(&password_row);
    let password_error = gtk::Label::builder()
        .css_classes(["error", "caption"])
        .halign(gtk::Align::Start)
        .margin_start(12)
        .visible(false)
        .build();
    password_error.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Password error",
    ))]);
    // Non-blocking warning label (FR-6).
    let password_warning = gtk::Label::builder()
        .css_classes(["warning", "caption"])
        .halign(gtk::Align::Start)
        .margin_start(12)
        .visible(false)
        .wrap(true)
        .build();
    password_warning.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Password warning",
    ))]);
    vbox.append(&password_group);
    vbox.append(&password_error);
    vbox.append(&password_warning);

    // Live password warning update as the user types (FR-6).
    password_row.connect_changed(clone!(
        #[weak]
        password_warning,
        move |row| {
            let pw = row.text().to_string();
            let warnings =
                crate::core::wizard_validation::validate_wizard_fields("x", "x@x.x", &pw)
                    .password_warnings;
            if warnings.is_empty() || pw.is_empty() {
                password_warning.set_visible(false);
            } else {
                let msgs: Vec<&str> = warnings.iter().map(|w| w.message()).collect();
                password_warning.set_label(&msgs.join("; "));
                password_warning.set_visible(true);
            }
        }
    ));

    // -- OAuth recommendation section (FR-21, AC-1, AC-2) --
    // Hidden by default; shown when provider detection finds OAuth support.
    let oauth_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .visible(false)
        .build();
    oauth_box.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "OAuth sign-in options",
    ))]);

    let oauth_provider_label = gtk::Label::builder()
        .css_classes(["title-4"])
        .halign(gtk::Align::Start)
        .margin_start(12)
        .wrap(true)
        .build();
    oauth_box.append(&oauth_provider_label);

    let oauth_signin_btn = gtk::Button::builder()
        .css_classes(["suggested-action", "pill"])
        .halign(gtk::Align::Center)
        .build();
    oauth_signin_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Sign in with your email provider using OAuth",
    ))]);
    oauth_box.append(&oauth_signin_btn);

    let oauth_caption = gtk::Label::builder()
        .label(gettextrs::gettext("Recommended — no password needed"))
        .css_classes(["dim-label", "caption"])
        .halign(gtk::Align::Center)
        .build();
    oauth_box.append(&oauth_caption);

    // Tenant input field for multi-tenant providers like Microsoft (FR-10, US-4).
    // Hidden by default; shown when the detected provider requires a tenant.
    let tenant_group = adw::PreferencesGroup::builder()
        .margin_start(12)
        .margin_end(12)
        .margin_top(4)
        .visible(false)
        .build();
    let tenant_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Tenant identifier (optional)"))
        .build();
    tenant_group.add(&tenant_row);
    let tenant_hint = gtk::Label::builder()
        .label(gettextrs::gettext(
            "Enter your organization's directory ID or domain, or leave blank for personal accounts",
        ))
        .css_classes(["dim-label", "caption"])
        .halign(gtk::Align::Start)
        .margin_start(12)
        .wrap(true)
        .build();
    tenant_group.add(&tenant_hint);
    oauth_box.append(&tenant_group);

    // Shared mailbox input field for providers that support delegation (FR-40, N-8).
    // Hidden by default; shown when the detected provider supports shared mailboxes.
    let shared_mailbox_group = adw::PreferencesGroup::builder()
        .margin_start(12)
        .margin_end(12)
        .margin_top(4)
        .visible(false)
        .build();
    let shared_mailbox_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Shared mailbox (optional)"))
        .build();
    shared_mailbox_group.add(&shared_mailbox_row);
    let shared_mailbox_hint = gtk::Label::builder()
        .label(gettextrs::gettext(
            "Enter the email address of a shared mailbox you have access to, or leave blank",
        ))
        .css_classes(["dim-label", "caption"])
        .halign(gtk::Align::Start)
        .margin_start(12)
        .wrap(true)
        .build();
    shared_mailbox_group.add(&shared_mailbox_hint);
    oauth_box.append(&shared_mailbox_group);

    let oauth_password_fallback_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Use password instead"))
        .css_classes(["flat", "pill"])
        .halign(gtk::Align::Center)
        .margin_top(4)
        .build();
    oauth_password_fallback_btn.update_property(&[gtk::accessible::Property::Label(
        &gettextrs::gettext("Switch to password-based authentication"),
    )]);
    oauth_box.append(&oauth_password_fallback_btn);

    // OAuth error section (FR-24, AC-5) — shown when connection test fails after OAuth.
    let oauth_error_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .visible(false)
        .build();

    let oauth_error_label = gtk::Label::builder()
        .css_classes(["error"])
        .halign(gtk::Align::Start)
        .margin_start(12)
        .wrap(true)
        .build();
    oauth_error_box.append(&oauth_error_label);

    let oauth_error_action_label = gtk::Label::builder()
        .css_classes(["caption"])
        .halign(gtk::Align::Start)
        .margin_start(12)
        .wrap(true)
        .build();
    oauth_error_box.append(&oauth_error_action_label);

    let oauth_error_link = gtk::LinkButton::builder()
        .halign(gtk::Align::Start)
        .margin_start(12)
        .visible(false)
        .build();
    oauth_error_box.append(&oauth_error_link);
    oauth_box.append(&oauth_error_box);

    vbox.append(&oauth_box);

    // -- OAuth unavailable info section (FR-29, US-15, AC-6, N-7) --
    // Shown when the provider supports OAuth but this build lacks client credentials.
    let oauth_unavailable_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .visible(false)
        .build();
    oauth_unavailable_box.update_property(&[gtk::accessible::Property::Label(
        &gettextrs::gettext("OAuth unavailable notice"),
    )]);

    let oauth_unavailable_label = gtk::Label::builder()
        .css_classes(["dim-label", "body"])
        .halign(gtk::Align::Start)
        .margin_start(12)
        .margin_end(12)
        .wrap(true)
        .build();
    oauth_unavailable_box.append(&oauth_unavailable_label);

    vbox.append(&oauth_unavailable_box);

    // Shared state for OAuth setup (passed between async steps).
    let oauth_state: Rc<RefCell<Option<OAuthSetupState>>> = Rc::new(RefCell::new(None));

    // -- Privacy policy links (FR-37, FR-38, AC-17) --
    let privacy_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_top(4)
        .build();

    // Security guarantee (FR-38).
    let security_label = gtk::Label::builder()
        .label(gettextrs::gettext(privacy::password_security_notice()))
        .css_classes(["dim-label", "caption"])
        .wrap(true)
        .halign(gtk::Align::Start)
        .margin_start(12)
        .build();
    privacy_box.append(&security_label);

    // Link row for both privacy policies.
    let links_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Start)
        .margin_start(12)
        .build();

    let app_privacy_link = gtk::LinkButton::builder()
        .label(gettextrs::gettext("Privacy Policy"))
        .uri(privacy::APP_PRIVACY_POLICY_URL)
        .build();
    app_privacy_link.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Application privacy policy",
    ))]);
    links_box.append(&app_privacy_link);

    let autoconfig_privacy_link = gtk::LinkButton::builder()
        .label(gettextrs::gettext("Mozilla Privacy Policy"))
        .uri(privacy::AUTOCONFIG_PRIVACY_POLICY_URL)
        .build();
    autoconfig_privacy_link.update_property(&[gtk::accessible::Property::Label(
        &gettextrs::gettext("Mozilla autoconfig service privacy policy"),
    )]);
    links_box.append(&autoconfig_privacy_link);

    privacy_box.append(&links_box);
    vbox.append(&privacy_box);

    // -- Check button (FR-7) --
    let btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .build();

    let manual_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Manual setup"))
        .css_classes(["pill"])
        .build();
    manual_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Switch to manual server configuration",
    ))]);
    btn_box.append(&manual_btn);

    let check_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Check"))
        .css_classes(["suggested-action", "pill"])
        .build();
    btn_box.append(&check_btn);

    vbox.append(&btn_box);

    // -- Re-authorize existing account option (FR-32, US-24) --
    let reauth_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Authorize existing account again"))
        .css_classes(["pill", "flat"])
        .halign(gtk::Align::Center)
        .margin_top(4)
        .build();
    reauth_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Re-authorize an existing account whose credentials have expired",
    ))]);
    vbox.append(&reauth_btn);

    // -- Detection progress indicator (FR-14, AC-16, NFR-8) --
    let progress_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .visible(false)
        .build();

    let progress_spinner = gtk::Spinner::builder().spinning(true).build();
    progress_box.append(&progress_spinner);

    let progress_label = gtk::Label::builder()
        .css_classes(["caption"])
        .halign(gtk::Align::Start)
        .build();
    // Mark the label as a live region for screen readers (NFR-8).
    progress_label.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Detection progress status",
    ))]);
    progress_box.append(&progress_label);

    vbox.append(&progress_box);

    // -- Detection failure fallback (FR-23, FR-24, FR-25, US-18) --
    let fallback_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .visible(false)
        .build();

    let fallback_data = build_detection_failure_fallback();

    let fallback_label = gtk::Label::builder()
        .label(gettextrs::gettext(&fallback_data.user_message))
        .wrap(true)
        .halign(gtk::Align::Center)
        .css_classes(["body"])
        .build();
    fallback_box.append(&fallback_label);

    let fallback_manual_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Set up manually"))
        .css_classes(["suggested-action", "pill"])
        .halign(gtk::Align::Center)
        .build();
    fallback_manual_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Switch to manual server configuration",
    ))]);
    fallback_box.append(&fallback_manual_btn);

    let fallback_support_link = gtk::LinkButton::builder()
        .label(gettextrs::gettext("Help & FAQ"))
        .uri(&fallback_data.support_url)
        .halign(gtk::Align::Center)
        .build();
    fallback_support_link.update_property(&[gtk::accessible::Property::Label(
        &gettextrs::gettext("Open general support and frequently asked questions"),
    )]);
    fallback_box.append(&fallback_support_link);

    vbox.append(&fallback_box);

    // -- Account review screen (FR-26) --
    let review_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .visible(false)
        .build();
    review_box.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Account review",
    ))]);

    // Provider name label (FR-26a).
    let review_provider_label = gtk::Label::builder()
        .css_classes(["title-3"])
        .halign(gtk::Align::Start)
        .build();
    review_box.append(&review_provider_label);

    // Editable account name (FR-26a, FR-26c).
    let review_account_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Account name"))
        .build();
    let review_account_row = adw::EntryRow::builder()
        .title(gettextrs::gettext("Account name"))
        .build();
    review_account_row.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Editable account name",
    ))]);
    review_account_group.add(&review_account_row);
    review_box.append(&review_account_group);

    // System folders list (FR-26b).
    let review_folders_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Detected folders"))
        .build();
    review_box.append(&review_folders_group);

    // Confirm / Back buttons for review screen.
    let review_btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .build();

    let review_back_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Back"))
        .css_classes(["pill"])
        .build();
    review_back_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Go back to modify inputs",
    ))]);
    review_btn_box.append(&review_back_btn);

    let review_confirm_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Save account"))
        .css_classes(["suggested-action", "pill"])
        .build();
    review_confirm_btn.update_property(&[gtk::accessible::Property::Label(&gettextrs::gettext(
        "Confirm and save account",
    ))]);
    review_btn_box.append(&review_confirm_btn);

    review_box.append(&review_btn_box);
    vbox.append(&review_box);

    clamp.set_child(Some(&vbox));
    scrolled.set_child(Some(&clamp));
    toast_overlay.set_child(Some(&scrolled));
    toolbar_view.set_content(Some(&toast_overlay));
    dialog.set_child(Some(&toolbar_view));

    let on_done = std::rc::Rc::new(on_done);
    let on_done_close = on_done.clone();

    // -- Email change handler: detect provider and show/hide OAuth (FR-21, AC-1, AC-2) --
    email_row.connect_changed(clone!(
        #[weak]
        oauth_box,
        #[weak]
        oauth_provider_label,
        #[weak]
        oauth_signin_btn,
        #[weak]
        oauth_error_box,
        #[weak]
        oauth_unavailable_box,
        #[weak]
        oauth_unavailable_label,
        #[weak]
        tenant_group,
        #[weak]
        shared_mailbox_group,
        #[weak]
        password_group,
        #[weak]
        password_error,
        #[weak]
        password_warning,
        #[weak]
        check_btn,
        move |row| {
            let email = row.text().to_string();
            let email = email.trim();

            // Reset OAuth error on email change.
            oauth_error_box.set_visible(false);
            oauth_unavailable_box.set_visible(false);

            // Extract domain and attempt provider detection.
            if let Some(at_pos) = email.rfind('@') {
                let domain = &email[at_pos + 1..];
                if !domain.is_empty() && domain.contains('.') {
                    let db = load_merged_provider_database();
                    if let Some(candidate) = db.lookup_by_domain(domain) {
                        let options = determine_auth_options(&candidate.provider);
                        if options.oauth_available && options.oauth_credentials_present {
                            // Show OAuth as recommended (AC-1, AC-2).
                            let btn_label = gettextrs::gettext("Sign in with %s")
                                .replace("%s", &options.provider_name);
                            oauth_signin_btn.set_label(&btn_label);
                            let info_label = gettextrs::gettext("%s supports secure sign-in")
                                .replace("%s", &options.provider_name);
                            oauth_provider_label.set_label(&info_label);

                            // Show tenant field for multi-tenant providers (FR-10, US-4).
                            let needs_tenant = options
                                .oauth_config
                                .as_ref()
                                .is_some_and(|c| c.requires_tenant());
                            tenant_group.set_visible(needs_tenant);

                            // Show shared mailbox field for providers that support it.
                            shared_mailbox_group
                                .set_visible(candidate.provider.supports_shared_mailbox);

                            oauth_box.set_visible(true);
                            password_group.set_visible(false);
                            password_error.set_visible(false);
                            password_warning.set_visible(false);
                            check_btn.set_visible(false);
                            return;
                        }

                        // Provider supports OAuth but credentials are missing (FR-29, AC-6, N-7).
                        if options.oauth_unavailable_reason
                            == Some(OAuthUnavailableReason::MissingCredentials)
                        {
                            oauth_unavailable_label
                                .set_label(&oauth_unavailable_message(&options.provider_name));
                            oauth_unavailable_box.set_visible(true);
                        }
                    }
                }
            }

            // No OAuth (or credentials missing) — show password flow.
            oauth_box.set_visible(false);
            password_group.set_visible(true);
            check_btn.set_visible(true);
        }
    ));

    // -- OAuth sign-in button handler (FR-21, FR-22, FR-23, AC-3, AC-4, AC-5) --
    oauth_signin_btn.connect_clicked(clone!(
        #[weak]
        name_row,
        #[weak]
        email_row,
        #[weak]
        tenant_row,
        #[weak]
        shared_mailbox_row,
        #[weak]
        name_error,
        #[weak]
        email_error,
        #[weak]
        toast_overlay,
        #[weak]
        oauth_box,
        #[weak]
        oauth_error_box,
        #[weak]
        oauth_error_label,
        #[weak]
        oauth_error_action_label,
        #[weak]
        oauth_error_link,
        #[weak]
        progress_box,
        #[weak]
        progress_label,
        #[weak]
        review_box,
        #[weak]
        review_provider_label,
        #[weak]
        review_account_row,
        #[weak]
        review_folders_group,
        #[weak]
        name_group,
        #[weak]
        email_group,
        #[weak]
        password_group,
        #[weak]
        privacy_box,
        #[weak]
        btn_box,
        #[weak]
        manual_btn,
        #[strong]
        oauth_state,
        move |btn| {
            let display_name = name_row.text().trim().to_string();
            let email = email_row.text().trim().to_string();
            let tenant_input = tenant_row.text().trim().to_string();
            let tenant_value = if tenant_input.is_empty() {
                None
            } else {
                Some(tenant_input)
            };
            let shared_mailbox_input = shared_mailbox_row.text().trim().to_string();
            let shared_mailbox_value = if shared_mailbox_input.is_empty() {
                None
            } else {
                Some(shared_mailbox_input)
            };

            // Validate name and email only (password not needed for OAuth).
            name_error.set_visible(false);
            email_error.set_visible(false);
            oauth_error_box.set_visible(false);

            if display_name.is_empty() {
                name_error.set_label(&gettextrs::gettext("Display name must not be empty"));
                name_error.set_visible(true);
                return;
            }
            if email.is_empty() || !email.contains('@') {
                email_error.set_label(&gettextrs::gettext("Email address is not valid"));
                email_error.set_visible(true);
                return;
            }

            if !is_network_available() {
                let toast = adw::Toast::new(&gettextrs::gettext(
                    "No network connection. Please check your internet and try again.",
                ));
                toast_overlay.add_toast(toast);
                return;
            }

            // Detect provider and get OAuth config.
            let db = load_merged_provider_database();
            let candidate = match db.lookup_by_email(&email) {
                Some(c) => c,
                None => return,
            };
            let options = determine_auth_options(&candidate.provider);
            let oauth_config = match options.oauth_config {
                Some(c) => c,
                None => return,
            };

            // Substitute tenant placeholder in endpoint URLs (FR-10, US-4).
            let oauth_config = oauth_config.with_tenant(tenant_value.as_deref());

            // Disable buttons and show progress.
            btn.set_sensitive(false);
            manual_btn.set_sensitive(false);
            progress_box.set_visible(true);
            progress_label.set_label(&gettextrs::gettext("Opening browser for authorization…"));

            let provider = candidate.provider.clone();
            let email_clone = email.clone();
            let display_name_clone = display_name.clone();
            let tenant_clone = tenant_value.clone();
            let shared_mailbox_clone = shared_mailbox_value.clone();

            // Run the full OAuth flow on a background thread (FR-5).
            // Results are polled from the main loop via glib::timeout_add_local.
            let browser_pref = oauth_service::load_browser_preference();
            let oauth_rx = {
                let (tx, rx) = std::sync::mpsc::channel::<OAuthMessage>();
                std::thread::spawn(move || {
                    run_oauth_thread(oauth_config, tx, browser_pref);
                });
                rx
            };

            // Poll for OAuth result from the main loop.
            let oauth_state_clone = oauth_state.clone();
            glib::timeout_add_local(
                std::time::Duration::from_millis(50),
                clone!(
                    #[weak]
                    progress_label,
                    #[weak]
                    progress_box,
                    #[weak(rename_to = oauth_btn)]
                    btn,
                    #[weak]
                    manual_btn,
                    #[weak]
                    toast_overlay,
                    #[weak]
                    oauth_error_box,
                    #[weak]
                    oauth_error_label,
                    #[weak]
                    oauth_error_action_label,
                    #[weak]
                    oauth_error_link,
                    #[weak]
                    review_box,
                    #[weak]
                    review_provider_label,
                    #[weak]
                    review_account_row,
                    #[weak]
                    review_folders_group,
                    #[weak]
                    name_group,
                    #[weak]
                    email_group,
                    #[weak]
                    password_group,
                    #[weak]
                    privacy_box,
                    #[weak]
                    btn_box,
                    #[weak]
                    oauth_box,
                    #[upgrade_or]
                    glib::ControlFlow::Break,
                    move || {
                        match oauth_rx.try_recv() {
                            Ok(OAuthMessage::Progress(text)) => {
                                progress_label.set_label(&gettextrs::gettext(&text));
                                glib::ControlFlow::Continue
                            }
                            Ok(OAuthMessage::Error(_err)) => {
                                progress_box.set_visible(false);
                                oauth_btn.set_sensitive(true);
                                manual_btn.set_sensitive(true);
                                let toast = adw::Toast::new(&gettextrs::gettext(
                                    "Authorization failed. Please try again.",
                                ));
                                toast_overlay.add_toast(toast);
                                glib::ControlFlow::Break
                            }
                            Ok(OAuthMessage::TokensReceived {
                                access_token,
                                refresh_token,
                                expires_in,
                            }) => {
                                progress_label.set_label(&gettextrs::gettext(
                                    "Testing mail server connection…",
                                ));

                                // Test connections with OAuth token on a background thread (AC-3).
                                let test_rx = {
                                    let provider_thread = provider.clone();
                                    let email_thread = email_clone.clone();
                                    let token_thread = access_token.clone();
                                    let (tx, rx) =
                                        std::sync::mpsc::channel::<ConnectionTestMessage>();
                                    std::thread::spawn(move || {
                                        use crate::services::imap_checker::ImapChecker;
                                        use crate::services::imap_checker::MockImapChecker;
                                        use crate::services::smtp_checker::MockSmtpChecker;
                                        use crate::services::smtp_checker::SmtpChecker;

                                        let imap_result = MockImapChecker.check_imap(
                                            &email_thread,
                                            &token_thread,
                                            &provider_thread,
                                            None,
                                        );
                                        let smtp_result = MockSmtpChecker.check_smtp(
                                            &email_thread,
                                            &token_thread,
                                            &provider_thread,
                                            None,
                                        );
                                        let _ = tx.send(ConnectionTestMessage {
                                            imap_result,
                                            smtp_result,
                                        });
                                    });
                                    rx
                                };

                                // Poll for connection test results.
                                let provider_result = provider.clone();
                                let email_result = email_clone.clone();
                                let display_name_result = display_name_clone.clone();
                                let tenant_result = tenant_clone.clone();
                                let shared_mailbox_result = shared_mailbox_clone.clone();
                                let oauth_state_inner = oauth_state_clone.clone();
                                glib::timeout_add_local(
                                    std::time::Duration::from_millis(50),
                                    clone!(
                                        #[weak]
                                        progress_box,
                                        #[weak(rename_to = oauth_btn)]
                                        oauth_btn,
                                        #[weak]
                                        manual_btn,
                                        #[weak]
                                        oauth_error_box,
                                        #[weak]
                                        oauth_error_label,
                                        #[weak]
                                        oauth_error_action_label,
                                        #[weak]
                                        oauth_error_link,
                                        #[weak]
                                        review_box,
                                        #[weak]
                                        review_provider_label,
                                        #[weak]
                                        review_account_row,
                                        #[weak]
                                        review_folders_group,
                                        #[weak]
                                        name_group,
                                        #[weak]
                                        email_group,
                                        #[weak]
                                        password_group,
                                        #[weak]
                                        privacy_box,
                                        #[weak]
                                        btn_box,
                                        #[weak]
                                        oauth_box,
                                        #[upgrade_or]
                                        glib::ControlFlow::Break,
                                        move || {
                                            match test_rx.try_recv() {
                                                Err(std::sync::mpsc::TryRecvError::Empty) => {
                                                    glib::ControlFlow::Continue
                                                }
                                                Err(_) => {
                                                    progress_box.set_visible(false);
                                                    oauth_btn.set_sensitive(true);
                                                    manual_btn.set_sensitive(true);
                                                    glib::ControlFlow::Break
                                                }
                                                Ok(test_msg) => {
                                                    progress_box.set_visible(false);
                                                    oauth_btn.set_sensitive(true);
                                                    manual_btn.set_sensitive(true);

                                                    let imap_ok = test_msg.imap_result.is_ok();
                                                    let smtp_ok = test_msg.smtp_result.is_ok();

                                                    if imap_ok && smtp_ok {
                                                        // Success (AC-4): store state, show review.
                                                        let imap_success =
                                                            test_msg.imap_result.unwrap();
                                                        let smtp_success =
                                                            test_msg.smtp_result.unwrap();

                                                        let review_data = build_review_data(
                                                            &provider_result.display_name,
                                                            &email_result,
                                                            imap_success.has_inbox,
                                                            &imap_success.system_folders,
                                                        );

                                                        *oauth_state_inner.borrow_mut() =
                                                            Some(OAuthSetupState {
                                                                provider: provider_result.clone(),
                                                                email: email_result.clone(),
                                                                display_name: display_name_result
                                                                    .clone(),
                                                                access_token: access_token.clone(),
                                                                refresh_token: refresh_token
                                                                    .clone(),
                                                                expires_in,
                                                                imap_result: imap_success,
                                                                smtp_result: smtp_success,
                                                                oauth_tenant: tenant_result.clone(),
                                                                shared_mailbox:
                                                                    shared_mailbox_result.clone(),
                                                            });

                                                        show_review_screen(
                                                            &review_data,
                                                            &review_box,
                                                            &review_provider_label,
                                                            &review_account_row,
                                                            &review_folders_group,
                                                            &name_group,
                                                            &email_group,
                                                            &password_group,
                                                            &privacy_box,
                                                            &btn_box,
                                                        );
                                                        oauth_box.set_visible(false);
                                                    } else {
                                                        // Failure (AC-5): provider-specific error.
                                                        let is_imap_failure = !imap_ok;
                                                        let conn_error =
                                                            build_oauth_connection_error(
                                                                &provider_result,
                                                                is_imap_failure,
                                                            );
                                                        oauth_error_label.set_label(
                                                            &gettextrs::gettext(
                                                                &conn_error.user_message,
                                                            ),
                                                        );
                                                        oauth_error_action_label.set_label(
                                                            &gettextrs::gettext(
                                                                &conn_error.corrective_action,
                                                            ),
                                                        );
                                                        if let Some(ref url) =
                                                            conn_error.documentation_url
                                                        {
                                                            oauth_error_link.set_uri(url);
                                                            oauth_error_link.set_label(
                                                                &gettextrs::gettext(
                                                                    "Provider documentation",
                                                                ),
                                                            );
                                                            oauth_error_link.set_visible(true);
                                                        } else {
                                                            oauth_error_link.set_visible(false);
                                                        }
                                                        oauth_error_box.set_visible(true);
                                                    }

                                                    glib::ControlFlow::Break
                                                }
                                            }
                                        }
                                    ),
                                );

                                glib::ControlFlow::Break
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => {
                                glib::ControlFlow::Continue
                            }
                            Err(_) => {
                                progress_box.set_visible(false);
                                oauth_btn.set_sensitive(true);
                                manual_btn.set_sensitive(true);
                                glib::ControlFlow::Break
                            }
                        }
                    }
                ),
            );
        }
    ));

    // -- OAuth password fallback: switch back to password mode (AC-2) --
    oauth_password_fallback_btn.connect_clicked(clone!(
        #[weak]
        oauth_box,
        #[weak]
        password_group,
        #[weak]
        check_btn,
        move |_| {
            oauth_box.set_visible(false);
            password_group.set_visible(true);
            check_btn.set_visible(true);
        }
    ));

    // -- Check button handler: validate then gate on network (FR-5, FR-7) --
    check_btn.connect_clicked(clone!(
        #[weak]
        name_row,
        #[weak]
        email_row,
        #[weak]
        password_row,
        #[weak]
        name_error,
        #[weak]
        email_error,
        #[weak]
        password_error,
        #[weak]
        password_warning,
        #[weak]
        toast_overlay,
        #[weak]
        dialog,
        #[weak]
        progress_box,
        #[weak]
        progress_label,
        #[weak]
        check_btn,
        #[weak]
        manual_btn,
        #[weak]
        review_box,
        #[weak]
        review_provider_label,
        #[weak]
        review_account_row,
        #[weak]
        review_folders_group,
        #[weak]
        name_group,
        #[weak]
        email_group,
        #[weak]
        password_group,
        #[weak]
        privacy_box,
        #[weak]
        btn_box,
        #[weak]
        fallback_box,
        #[strong]
        on_done,
        move |_| {
            let display_name = name_row.text().to_string();
            let email = email_row.text().to_string();
            let password = password_row.text().to_string();

            let result = validate_wizard_fields(&display_name, &email, &password);

            // Reset error labels.
            name_error.set_visible(false);
            email_error.set_visible(false);
            password_error.set_visible(false);

            // Show field-specific errors (AC-14).
            apply_field_errors(&result, &name_error, &email_error, &password_error);

            // Show password warnings (FR-6).
            if result.password_warnings.is_empty() {
                password_warning.set_visible(false);
            } else {
                let msgs: Vec<&str> = result
                    .password_warnings
                    .iter()
                    .map(|w| w.message())
                    .collect();
                password_warning.set_label(&msgs.join("; "));
                password_warning.set_visible(true);
            }

            if !result.is_valid() {
                return;
            }

            // FR-13: proprietary provider rejection (US-9).
            // Check before any network request is made.
            if let Some(proprietary) = check_proprietary_provider(&email) {
                let msg = gettextrs::gettext(
                    "%s does not support standard email protocols and is not compatible with this application.",
                ).replace("%s", &proprietary.provider_name);
                email_error.set_label(&msg);
                email_error.set_visible(true);
                return;
            }

            // FR-7: network availability gate.
            if !is_network_available() {
                let toast = adw::Toast::new(&gettextrs::gettext(
                    "No network connection. Please check your internet and try again.",
                ));
                toast_overlay.add_toast(toast);
                return;
            }

            // Validation passed and network is available.
            // Show detection progress (FR-14, AC-16) and disable buttons.
            check_btn.set_sensitive(false);
            manual_btn.set_sensitive(false);
            progress_box.set_visible(true);

            // Extract domain for progress messages.
            let domain = email
                .trim()
                .rsplit('@')
                .next()
                .unwrap_or("server")
                .to_string();

            // Run detection progress on the main loop so UI stays responsive (AC-16).
            let steps = detection_sequence(&domain);
            run_detection_progress(
                steps,
                progress_label.clone(),
                progress_box.clone(),
                fallback_box.clone(),
                review_box.clone(),
                review_provider_label.clone(),
                review_account_row.clone(),
                review_folders_group.clone(),
                name_group.clone(),
                email_group.clone(),
                password_group.clone(),
                privacy_box.clone(),
                btn_box.clone(),
                check_btn.clone(),
                manual_btn.clone(),
                dialog.clone(),
                on_done.clone(),
                WizardData {
                    display_name: display_name.trim().to_string(),
                    email: email.trim().to_string(),
                    password,
                },
            );
        }
    ));

    // -- Manual setup button handler (FR-35, US-26) --
    // Carries over any entered data without validation (FR-36).
    manual_btn.connect_clicked(clone!(
        #[weak]
        name_row,
        #[weak]
        email_row,
        #[weak]
        password_row,
        #[weak]
        dialog,
        #[strong]
        on_done,
        move |_| {
            let display_name = name_row.text().trim().to_string();
            let email = email_row.text().trim().to_string();
            let password = password_row.text().to_string();

            on_done(Some(WizardAction::ManualSetup(WizardData {
                display_name,
                email,
                password,
            })));
            dialog.close();
        }
    ));

    // -- Fallback manual setup button handler (FR-23, FR-36, US-18) --
    // Carries over entered data from the wizard to manual setup.
    fallback_manual_btn.connect_clicked(clone!(
        #[weak]
        name_row,
        #[weak]
        email_row,
        #[weak]
        password_row,
        #[weak]
        dialog,
        #[strong]
        on_done,
        move |_| {
            let display_name = name_row.text().trim().to_string();
            let email = email_row.text().trim().to_string();
            let password = password_row.text().to_string();

            on_done(Some(WizardAction::ManualSetup(WizardData {
                display_name,
                email,
                password,
            })));
            dialog.close();
        }
    ));

    // -- Re-authorize button handler (FR-32, US-24) --
    // Carries over entered email and password for re-authorization.
    reauth_btn.connect_clicked(clone!(
        #[weak]
        name_row,
        #[weak]
        email_row,
        #[weak]
        password_row,
        #[weak]
        dialog,
        #[strong]
        on_done,
        move |_| {
            let display_name = name_row.text().trim().to_string();
            let email = email_row.text().trim().to_string();
            let password = password_row.text().to_string();

            on_done(Some(WizardAction::Reauthorize(WizardData {
                display_name,
                email,
                password,
            })));
            dialog.close();
        }
    ));

    // -- Review screen: Back button handler (FR-26) --
    // Hides the review screen and shows the input form again.
    review_back_btn.connect_clicked(clone!(
        #[weak]
        review_box,
        #[weak]
        name_group,
        #[weak]
        email_group,
        #[weak]
        password_group,
        #[weak]
        privacy_box,
        #[weak]
        btn_box,
        #[weak]
        oauth_box,
        #[weak]
        oauth_unavailable_box,
        #[weak]
        oauth_unavailable_label,
        #[weak]
        email_row,
        #[weak]
        check_btn,
        #[strong]
        oauth_state,
        move |_| {
            // Clear any stored OAuth state when going back.
            *oauth_state.borrow_mut() = None;

            review_box.set_visible(false);
            name_group.set_visible(true);
            email_group.set_visible(true);
            privacy_box.set_visible(true);
            btn_box.set_visible(true);
            oauth_unavailable_box.set_visible(false);

            // Re-detect provider to decide whether to show OAuth or password.
            let email = email_row.text().to_string();
            let email = email.trim();
            let mut show_oauth = false;
            if let Some(at_pos) = email.rfind('@') {
                let domain = &email[at_pos + 1..];
                if !domain.is_empty() && domain.contains('.') {
                    let db = load_merged_provider_database();
                    if let Some(candidate) = db.lookup_by_domain(domain) {
                        let options = determine_auth_options(&candidate.provider);
                        if options.oauth_available && options.oauth_credentials_present {
                            show_oauth = true;
                        } else if options.oauth_unavailable_reason
                            == Some(OAuthUnavailableReason::MissingCredentials)
                        {
                            oauth_unavailable_label
                                .set_label(&oauth_unavailable_message(&options.provider_name));
                            oauth_unavailable_box.set_visible(true);
                        }
                    }
                }
            }
            if show_oauth {
                oauth_box.set_visible(true);
                password_group.set_visible(false);
                check_btn.set_visible(false);
            } else {
                password_group.set_visible(true);
                check_btn.set_visible(true);
            }
        }
    ));

    // -- Review screen: Confirm button handler (FR-26) --
    // Saves the account with the (possibly edited) account name.
    review_confirm_btn.connect_clicked(clone!(
        #[weak]
        review_account_row,
        #[weak]
        name_row,
        #[weak]
        email_row,
        #[weak]
        password_row,
        #[weak]
        dialog,
        #[weak]
        toast_overlay,
        #[strong]
        oauth_state,
        #[strong]
        on_done,
        move |_| {
            let account_name = review_account_row.text().trim().to_string();
            let display_name = if account_name.is_empty() {
                name_row.text().trim().to_string()
            } else {
                account_name
            };
            let email = email_row.text().trim().to_string();

            // Check if this is an OAuth flow (AC-4, FR-23).
            let oauth = oauth_state.borrow_mut().take();
            if let Some(state) = oauth {
                // Use the (possibly edited) display name from the review screen.
                match create_oauth_account(
                    state.provider,
                    state.email.clone(),
                    display_name.clone(),
                    state.access_token,
                    state.imap_result,
                    state.smtp_result,
                    state.oauth_tenant,
                    state.shared_mailbox,
                ) {
                    Ok(_result) => {
                        on_done(Some(WizardAction::OAuthComplete(WizardData {
                            display_name,
                            email: state.email,
                            password: String::new(),
                        })));
                    }
                    Err(e) => {
                        let toast = adw::Toast::new(&format!(
                            "{}: {}",
                            gettextrs::gettext("Could not create account"),
                            e
                        ));
                        toast_overlay.add_toast(toast);
                        return;
                    }
                }
            } else {
                // Password-based flow (existing behavior).
                let password = password_row.text().to_string();
                on_done(Some(WizardAction::Check(WizardData {
                    display_name,
                    email,
                    password,
                })));
            }
            dialog.close();
        }
    ));

    dialog.connect_closed(move |_| {
        let _ = &on_done_close;
    });

    dialog.present(Some(parent));
}

/// Run detection progress animation on the main loop (FR-14, AC-16).
///
/// Steps through each detection phase with a short delay so the user can
/// see real-time updates. The UI thread is never blocked because we use
/// `glib::timeout_future` which yields back to the main loop.
///
/// When detection completes successfully, the review screen is shown (FR-26).
/// When detection fails, the fallback box is shown (FR-23, FR-24, FR-25, US-18).
#[allow(clippy::too_many_arguments)]
fn run_detection_progress(
    steps: Vec<DetectionStep>,
    progress_label: gtk::Label,
    progress_box: gtk::Box,
    fallback_box: gtk::Box,
    review_box: gtk::Box,
    review_provider_label: gtk::Label,
    review_account_row: adw::EntryRow,
    review_folders_group: adw::PreferencesGroup,
    name_group: adw::PreferencesGroup,
    email_group: adw::PreferencesGroup,
    password_group: adw::PreferencesGroup,
    privacy_box: gtk::Box,
    btn_box: gtk::Box,
    check_btn: gtk::Button,
    manual_btn: gtk::Button,
    _dialog: adw::Dialog,
    _on_done: std::rc::Rc<dyn Fn(WizardResult)>,
    _data: WizardData,
) {
    glib::MainContext::default().spawn_local(async move {
        for step in &steps {
            let msg = step.message();
            progress_label.set_label(&msg);
            // Update accessible description for screen readers (NFR-8).
            progress_label.update_property(&[gtk::accessible::Property::Description(
                &step.accessible_description(),
            )]);
            // Yield to the main loop with a short delay so the UI stays responsive
            // and the user can observe each step (AC-16).
            glib::timeout_future(std::time::Duration::from_millis(400)).await;
        }

        // Detection complete — hide progress and re-enable buttons.
        progress_box.set_visible(false);
        check_btn.set_sensitive(true);
        manual_btn.set_sensitive(true);

        // No real detection pipeline yet — show the failure fallback (FR-23).
        // Future slices will replace this with actual provider detection and show
        // the review screen on success via `show_review_screen()`.
        fallback_box.set_visible(true);

        // The review screen infrastructure is ready. When a real detection pipeline
        // returns a ConnectivityCheckResult, call show_review_screen() instead of
        // showing fallback_box. Example (for future use):
        // show_review_screen(
        //     &review_data,
        //     &review_box, &review_provider_label, &review_account_row,
        //     &review_folders_group, &name_group, &email_group,
        //     &password_group, &privacy_box, &btn_box,
        // );

        // Suppress unused variable warnings for review widgets until pipeline is wired.
        let _ = (&review_box, &review_provider_label, &review_account_row);
        let _ = (&review_folders_group, &name_group, &email_group);
        let _ = (&password_group, &privacy_box, &btn_box);
    });
}

/// Populate and show the account review screen (FR-26).
///
/// Hides the input form and displays the review with provider name,
/// editable account name, and detected folder indicators.
#[allow(dead_code, clippy::too_many_arguments)]
pub(crate) fn show_review_screen(
    data: &AccountReviewData,
    review_box: &gtk::Box,
    review_provider_label: &gtk::Label,
    review_account_row: &adw::EntryRow,
    review_folders_group: &adw::PreferencesGroup,
    name_group: &adw::PreferencesGroup,
    email_group: &adw::PreferencesGroup,
    password_group: &adw::PreferencesGroup,
    privacy_box: &gtk::Box,
    btn_box: &gtk::Box,
) {
    // Hide the input form.
    name_group.set_visible(false);
    email_group.set_visible(false);
    password_group.set_visible(false);
    privacy_box.set_visible(false);
    btn_box.set_visible(false);

    // Populate provider name (FR-26a).
    review_provider_label
        .set_label(&gettextrs::gettext("Provider: %s").replace("%s", &data.provider_name));

    // Populate editable account name (FR-26c).
    review_account_row.set_text(&data.account_name);

    // Clear any previous folder rows.
    while let Some(child) = review_folders_group.first_child() {
        // Skip the group title/header widget — only remove ActionRow children.
        if child.downcast_ref::<adw::ActionRow>().is_some() {
            review_folders_group.remove(&child);
        } else {
            // Move past non-removable children (header).
            break;
        }
    }

    // Add Inbox entry.
    let inbox_row = adw::ActionRow::builder()
        .title(gettextrs::gettext("Inbox"))
        .build();
    let inbox_icon = if data.has_inbox {
        "emblem-ok-symbolic"
    } else {
        "window-close-symbolic"
    };
    let inbox_suffix = gtk::Image::from_icon_name(inbox_icon);
    inbox_suffix.update_property(&[gtk::accessible::Property::Label(&if data.has_inbox {
        gettextrs::gettext("Found")
    } else {
        gettextrs::gettext("Not found")
    })]);
    inbox_row.add_suffix(&inbox_suffix);
    review_folders_group.add(&inbox_row);

    // Add system folder entries (FR-26b).
    for entry in &data.folder_entries {
        let row = adw::ActionRow::builder()
            .title(gettextrs::gettext(entry.role_label()))
            .build();

        let icon_name = if entry.is_detected() {
            "emblem-ok-symbolic"
        } else {
            "window-close-symbolic"
        };
        let suffix = gtk::Image::from_icon_name(icon_name);
        let accessible_label = if entry.is_detected() {
            gettextrs::gettext("Found")
        } else {
            gettextrs::gettext("Not found")
        };
        suffix.update_property(&[gtk::accessible::Property::Label(&accessible_label)]);
        row.add_suffix(&suffix);

        // Show the server folder name as subtitle if detected.
        if let Some(ref name) = entry.server_name {
            row.set_subtitle(name);
        }

        review_folders_group.add(&row);
    }

    // Show the review screen.
    review_box.set_visible(true);
}

/// Map validation errors to field-specific error labels.
fn apply_field_errors(
    result: &WizardValidationResult,
    name_error: &gtk::Label,
    email_error: &gtk::Label,
    password_error: &gtk::Label,
) {
    for err in &result.errors {
        match err {
            WizardFieldError::EmptyDisplayName => {
                name_error.set_label(&gettextrs::gettext("Display name must not be empty"));
                name_error.set_visible(true);
            }
            WizardFieldError::EmptyEmail => {
                email_error.set_label(&gettextrs::gettext("Email address must not be empty"));
                email_error.set_visible(true);
            }
            WizardFieldError::InvalidEmail => {
                email_error.set_label(&gettextrs::gettext("Email address is not valid"));
                email_error.set_visible(true);
            }
            WizardFieldError::EmptyPassword => {
                password_error.set_label(&gettextrs::gettext("Password must not be empty"));
                password_error.set_visible(true);
            }
        }
    }
}

/// Run the OAuth authorization flow on a background thread (FR-5, AC-3).
///
/// Binds a local redirect listener, opens the system browser for authorization,
/// waits for the callback, validates the state, exchanges the authorization code
/// for tokens, and sends progress/result messages back to the UI thread.
fn run_oauth_thread(
    oauth_config: crate::core::provider::OAuthConfig,
    tx: std::sync::mpsc::Sender<OAuthMessage>,
    oauth_browser_preference: Option<String>,
) {
    // Step 1: Bind redirect listener.
    let (listener, port) = match oauth_service::bind_redirect_listener() {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(OAuthMessage::Error(e.to_string()));
            return;
        }
    };

    // Step 2: Build session and open browser.
    let session = OAuthSession::new(oauth_config, port);
    let url = session.authorization_url();
    match oauth_service::open_browser_with_selection(&url, oauth_browser_preference.as_deref()) {
        Ok(result) => {
            if let Some(warning) = &result.warning {
                let _ = tx.send(OAuthMessage::Progress(warning.clone()));
            }
            let _ = tx.send(OAuthMessage::Progress(format!(
                "Waiting for authorization in {}…",
                result.browser_name
            )));
        }
        Err(e) => {
            let _ = tx.send(OAuthMessage::Error(e.to_string()));
            return;
        }
    }

    // Step 3: Wait for callback (blocks until browser redirects).
    let callback = match oauth_service::wait_for_callback(listener) {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(OAuthMessage::Error(e.to_string()));
            return;
        }
    };

    // Step 4: Validate state.
    if let Err(e) = session.validate_state(Some(&callback.state)) {
        let _ = tx.send(OAuthMessage::Error(e.to_string()));
        return;
    }

    let _ = tx.send(OAuthMessage::Progress(
        "Exchanging authorization code…".to_string(),
    ));

    // Step 5: Exchange code for tokens.
    let exchange_params = session.token_exchange_params(&callback.code);
    let token_response = match oauth_service::exchange_code_for_tokens(exchange_params) {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(OAuthMessage::Error(e.to_string()));
            return;
        }
    };

    // Step 6: Validate response has refresh token.
    let validated = match crate::core::oauth_flow::validate_token_response(token_response) {
        Ok(v) => v,
        Err(e) => {
            let _ = tx.send(OAuthMessage::Error(e.to_string()));
            return;
        }
    };

    let _ = tx.send(OAuthMessage::TokensReceived {
        access_token: validated.access_token,
        refresh_token: validated.refresh_token,
        expires_in: validated.expires_in,
    });
}

/// Load the provider database merged with any user-supplied custom providers.
///
/// Falls back to the bundled-only database if the user provider file is
/// absent or cannot be read/parsed.
fn load_merged_provider_database() -> ProviderDatabase {
    let user_content = crate::services::user_provider_service::load_user_provider_file()
        .ok()
        .flatten();
    build_merged_database(user_content.as_deref()).unwrap_or_else(|_| ProviderDatabase::bundled())
}
