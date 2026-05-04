use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use glib::clone;

use crate::core::privacy;
use crate::core::wizard_validation::{
    validate_wizard_fields, WizardFieldError, WizardValidationResult,
};
use crate::services::network::is_network_available;

/// Result passed back from the wizard: the validated name, email, and password,
/// or `None` if the user cancelled / closed the dialog.
pub(crate) type WizardResult = Option<WizardData>;

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

    let check_btn = gtk::Button::builder()
        .label(gettextrs::gettext("Check"))
        .css_classes(["suggested-action", "pill"])
        .build();
    btn_box.append(&check_btn);

    vbox.append(&btn_box);

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

            // FR-7: network availability gate.
            if !is_network_available() {
                let toast = adw::Toast::new(&gettextrs::gettext(
                    "No network connection. Please check your internet and try again.",
                ));
                toast_overlay.add_toast(toast);
                return;
            }

            // Validation passed and network is available.
            // This slice does NOT proceed further (no provider detection / account creation).
            // For now, return the validated data so the caller can use it in a future slice.
            on_done(Some(WizardData {
                display_name: display_name.trim().to_string(),
                email: email.trim().to_string(),
                password,
            }));
            dialog.close();
        }
    ));

    dialog.connect_closed(move |_| {
        let _ = &on_done_close;
    });

    dialog.present(Some(parent));
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
