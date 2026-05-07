//! Queue view dialog showing pending operations (AC-16).
//!
//! Displays operation type, target message/folder, status (queued, in progress,
//! failed), and error message for each operation. Updates in real-time via
//! periodic polling.

use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::pending_operation::PendingOperation;
use crate::core::pending_operation_view::summarize_operation;

/// Show the pending operations queue dialog (AC-16).
///
/// The dialog displays all operations and refreshes every second so the
/// queue view updates in real-time as operations execute.
pub(crate) fn show(parent: &adw::ApplicationWindow, db_path: std::path::PathBuf) {
    let dialog = adw::Dialog::builder()
        .title(gettextrs::gettext("Operation Queue"))
        .content_width(560)
        .content_height(460)
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

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    let group = adw::PreferencesGroup::builder()
        .title(gettextrs::gettext("Pending Operations"))
        .build();
    content_box.append(&group);

    clamp.set_child(Some(&content_box));
    scrolled.set_child(Some(&clamp));
    toolbar_view.set_content(Some(&scrolled));
    dialog.set_child(Some(&toolbar_view));

    // Initial population.
    let ops = load_ops_from_db(&db_path);
    populate_group(&group, &ops);

    // Real-time refresh: poll every second while the dialog is open.
    let group_ref = Rc::new(group);
    let db_path_ref = Rc::new(db_path);
    let dialog_alive = Rc::new(RefCell::new(true));

    let alive_for_close = dialog_alive.clone();
    dialog.connect_closed(move |_| {
        *alive_for_close.borrow_mut() = false;
    });

    let group_timer = group_ref.clone();
    let db_timer = db_path_ref.clone();
    let alive_timer = dialog_alive.clone();
    glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        if !*alive_timer.borrow() {
            return glib::ControlFlow::Break;
        }
        let ops = load_ops_from_db(&db_timer);
        populate_group(&group_timer, &ops);
        glib::ControlFlow::Continue
    });

    dialog.present(Some(parent));
}

/// Load all pending operations from the database.
fn load_ops_from_db(db_path: &std::path::Path) -> Vec<PendingOperation> {
    let conn = match rusqlite::Connection::open(db_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    crate::services::pending_ops_store::load_all_ops(&conn).unwrap_or_default()
}

/// Rebuild the preferences group contents from the current operation list.
fn populate_group(group: &adw::PreferencesGroup, ops: &[PendingOperation]) {
    // Remove existing rows.
    while let Some(child) = group.first_child() {
        // PreferencesGroup has internal children (header labels); only remove
        // rows we added. We walk children and remove ActionRows.
        if child.downcast_ref::<adw::ActionRow>().is_some() {
            group.remove(&child);
        } else {
            // Skip internal children — break to avoid infinite loop.
            break;
        }
    }

    if ops.is_empty() {
        let empty_row = adw::ActionRow::builder()
            .title(gettextrs::gettext("No pending operations"))
            .css_classes(["dim-label"])
            .build();
        group.add(&empty_row);
        return;
    }

    for op in ops {
        let summary = summarize_operation(op);

        let subtitle = if let Some(ref err) = summary.error {
            format!("{} — {}", summary.status, err)
        } else {
            summary.status.clone()
        };

        let title = format!("{}: {}", summary.operation_type, summary.target);
        let row = adw::ActionRow::builder()
            .title(title)
            .subtitle(&subtitle)
            .build();

        // Status icon: different icon per state.
        let (icon_name, css_class) = match op.state {
            crate::core::pending_operation::OperationState::Pending => {
                ("emblem-synchronizing-symbolic", "accent")
            }
            crate::core::pending_operation::OperationState::InFlight => {
                ("media-playback-start-symbolic", "accent")
            }
            crate::core::pending_operation::OperationState::Failed => {
                ("dialog-warning-symbolic", "error")
            }
        };

        let icon = gtk::Image::builder()
            .icon_name(icon_name)
            .pixel_size(16)
            .valign(gtk::Align::Center)
            .css_classes([css_class])
            .build();
        row.add_prefix(&icon);

        group.add(&row);
    }
}
