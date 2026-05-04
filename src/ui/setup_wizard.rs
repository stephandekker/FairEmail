use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use glib::clone;

use crate::core::account_review::AccountReviewData;
use crate::core::detection_failure::build_detection_failure_fallback;
use crate::core::detection_progress::{detection_sequence, DetectionStep};
use crate::core::privacy;
use crate::core::proprietary_provider::check_proprietary_provider;
use crate::core::wizard_validation::{
    validate_wizard_fields, WizardFieldError, WizardValidationResult,
};
use crate::services::network::is_network_available;

/// Result passed back from the wizard: the validated name, email, and password,
/// or `None` if the user cancelled / closed the dialog.
pub(crate) type WizardResult = Option<WizardAction>;

/// The action chosen in the wizard: auto-detect or manual setup (FR-35).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum WizardAction {
    /// User clicked "Check" — proceed with auto-detection.
    Check(WizardData),
    /// User clicked "Manual setup" — open manual configuration (US-26).
    ManualSetup(WizardData),
}

/// Validated wizard data ready for the next step (provider detection / account creation).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct WizardData {
    pub display_name: String,
    pub email: String,
    pub password: String,
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
        move |_| {
            review_box.set_visible(false);
            name_group.set_visible(true);
            email_group.set_visible(true);
            password_group.set_visible(true);
            privacy_box.set_visible(true);
            btn_box.set_visible(true);
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
            let password = password_row.text().to_string();

            on_done(Some(WizardAction::Check(WizardData {
                display_name,
                email,
                password,
            })));
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
