use libadwaita as adw;
use libadwaita::prelude::*;

use glib::clone;

use crate::core::certificate::{CertificateDecision, CertificateInfo};

/// Show a dialog presenting untrusted certificate details and allowing
/// the user to accept or reject it (FR-19, AC-8, NFR-8).
///
/// `on_decision` is called with the user's choice.
pub(crate) fn show(
    parent: &adw::Dialog,
    cert_info: &CertificateInfo,
    on_decision: impl Fn(CertificateDecision) + 'static,
) {
    let dialog = adw::AlertDialog::builder()
        .heading(gettextrs::gettext("Untrusted Certificate"))
        .build();

    // Build the body with certificate details (FR-19a, FR-19b).
    let body = build_certificate_body(cert_info);
    dialog.set_body(&body);
    dialog.set_body_use_markup(true);

    // Add responses: Reject (default) and Accept (FR-19c).
    dialog.add_response("reject", &gettextrs::gettext("Reject"));
    dialog.add_response("accept", &gettextrs::gettext("Accept Certificate"));

    dialog.set_response_appearance("accept", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("reject"));
    dialog.set_close_response("reject");

    dialog.connect_response(
        None,
        clone!(move |_dialog, response| {
            let decision = if response == "accept" {
                CertificateDecision::Accept
            } else {
                CertificateDecision::Reject
            };
            on_decision(decision);
        }),
    );

    // Present as a child of the wizard dialog's widget.
    // AlertDialog uses present() with a widget reference.
    if let Some(child) = parent.child() {
        dialog.present(Some(&child));
    }
}

/// Build the formatted body text for the certificate dialog (FR-19a, FR-19b).
fn build_certificate_body(cert_info: &CertificateInfo) -> String {
    let mut parts = Vec::new();

    // Server hostname
    parts.push(format!(
        "{}: <b>{}</b>",
        gettextrs::gettext("Server"),
        glib::markup_escape_text(&cert_info.server_hostname)
    ));

    // Fingerprint (FR-19a, AC-8)
    parts.push(format!(
        "\n{}: <tt>{}</tt>",
        gettextrs::gettext("Fingerprint (SHA-256)"),
        glib::markup_escape_text(&cert_info.fingerprint)
    ));

    // DNS names (FR-19a)
    parts.push(format!("\n{}:", gettextrs::gettext("DNS names")));

    let has_mismatch = cert_info.has_hostname_mismatch();

    if cert_info.dns_names.is_empty() {
        parts.push(format!(
            "  <i>{}</i>",
            gettextrs::gettext("(none — certificate has no DNS names)")
        ));
    } else {
        for name in &cert_info.dns_names {
            let escaped = glib::markup_escape_text(name);
            if has_mismatch {
                // Visually highlight mismatch (FR-19b): show names in warning color
                parts.push(format!("  • <span foreground=\"#e5a50a\">{escaped}</span>"));
            } else {
                parts.push(format!("  • {escaped}"));
            }
        }
    }

    // Mismatch warning (FR-19b)
    if has_mismatch {
        parts.push(format!(
            "\n<span foreground=\"#e5a50a\"><b>{}</b></span>",
            gettextrs::gettext("Warning: The certificate does not cover the server hostname.")
        ));
    }

    // General warning about accepting
    parts.push(format!(
        "\n{}",
        gettextrs::gettext(
            "Accepting this certificate allows connecting to this server without standard trust verification."
        )
    ));

    parts.join("\n")
}
