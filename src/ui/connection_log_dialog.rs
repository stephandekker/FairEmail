use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::core::{format_log_timestamp, ConnectionLogEntry, ConnectionState};

/// Show a dialog displaying the connection log for an account (FR-46, US-42).
pub(crate) fn show(
    parent: &adw::ApplicationWindow,
    account_name: &str,
    state: ConnectionState,
    error_detail: Option<&str>,
    entries: &[ConnectionLogEntry],
) {
    let dialog = adw::Dialog::builder()
        .title(gettextrs::gettext("Connection Log"))
        .content_width(500)
        .content_height(400)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let scrolled = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();

    let clamp = adw::Clamp::builder()
        .maximum_size(600)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);

    // Account name and current state summary.
    let status_group = adw::PreferencesGroup::builder().title(account_name).build();

    let state_row = adw::ActionRow::builder()
        .title(gettextrs::gettext("Connection state"))
        .subtitle(state.to_string())
        .build();

    let state_icon = gtk::Image::builder()
        .icon_name(state.icon_name())
        .pixel_size(16)
        .valign(gtk::Align::Center)
        .css_classes([state.css_class()])
        .build();
    state_row.add_prefix(&state_icon);
    status_group.add(&state_row);

    // Show error detail if present (FR-45).
    if let Some(error) = error_detail {
        let error_row = adw::ActionRow::builder()
            .title(gettextrs::gettext("Error"))
            .subtitle(error)
            .css_classes(["error"])
            .build();
        let error_icon = gtk::Image::builder()
            .icon_name("dialog-warning-symbolic")
            .pixel_size(16)
            .valign(gtk::Align::Center)
            .build();
        error_row.add_prefix(&error_icon);
        status_group.add(&error_row);
    }

    vbox.append(&status_group);

    // Log entries (FR-46, US-42).
    let log_group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Log"))
        .build();

    if entries.is_empty() {
        let empty_row = adw::ActionRow::builder()
            .title(gettextrs::gettext("No log entries yet"))
            .css_classes(["dim-label"])
            .build();
        log_group.add(&empty_row);
    } else {
        // Show entries in reverse chronological order (newest first).
        for entry in entries.iter().rev() {
            let timestamp = format_log_timestamp(entry.timestamp_secs);
            let row = adw::ActionRow::builder()
                .title(&entry.message)
                .subtitle(&timestamp)
                .build();
            log_group.add(&row);
        }
    }

    vbox.append(&log_group);

    clamp.set_child(Some(&vbox));
    scrolled.set_child(Some(&clamp));
    toolbar_view.set_content(Some(&scrolled));
    dialog.set_child(Some(&toolbar_view));

    dialog.present(Some(parent));
}
