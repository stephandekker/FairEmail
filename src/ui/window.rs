use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::Account;
use crate::services::AccountStore;
use crate::ui::add_account_dialog;
use crate::ui::edit_account_dialog;

/// Build the main application window with the account list and navigation pane.
pub(crate) fn build(app: &adw::Application, store: Rc<AccountStore>) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(gettextrs::gettext("Alarm Clock – Accounts"))
        .default_width(720)
        .default_height(480)
        .build();

    let split_view = adw::NavigationSplitView::new();

    // -- Sidebar: account list --
    let sidebar_header = adw::HeaderBar::new();
    let add_btn = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(gettextrs::gettext("Add account"))
        .accessible_role(gtk::AccessibleRole::Button)
        .build();
    sidebar_header.pack_start(&add_btn);

    let account_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .css_classes(["navigation-sidebar"])
        .accessible_role(gtk::AccessibleRole::List)
        .build();
    account_list.set_placeholder(Some(
        &gtk::Label::builder()
            .label(gettextrs::gettext("No accounts yet"))
            .css_classes(["dim-label"])
            .margin_top(24)
            .margin_bottom(24)
            .build(),
    ));

    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&account_list)
        .build();

    let sidebar_toolbar = adw::ToolbarView::new();
    sidebar_toolbar.add_top_bar(&sidebar_header);
    sidebar_toolbar.set_content(Some(&scroll));

    let sidebar_page = adw::NavigationPage::builder()
        .title(gettextrs::gettext("Accounts"))
        .child(&sidebar_toolbar)
        .build();

    // -- Content pane (placeholder) --
    let content_header = adw::HeaderBar::new();
    let content_label = gtk::Label::builder()
        .label(gettextrs::gettext("Select an account"))
        .css_classes(["dim-label"])
        .vexpand(true)
        .build();
    let content_toolbar = adw::ToolbarView::new();
    content_toolbar.add_top_bar(&content_header);
    content_toolbar.set_content(Some(&content_label));

    let content_page = adw::NavigationPage::builder()
        .title(gettextrs::gettext("Details"))
        .child(&content_toolbar)
        .build();

    split_view.set_sidebar(Some(&sidebar_page));
    split_view.set_content(Some(&content_page));

    window.set_content(Some(&split_view));

    // Shared mutable list of accounts for the sidebar model.
    let accounts: Rc<RefCell<Vec<Account>>> = Rc::new(RefCell::new(Vec::new()));

    // Populate from store on startup.
    {
        let loaded = store.load_all().unwrap_or_default();
        for acct in &loaded {
            let row = make_account_row(acct);
            account_list.append(&row);
        }
        *accounts.borrow_mut() = loaded;
    }

    // "Add account" button handler.
    add_btn.connect_clicked(clone!(
        #[weak]
        window,
        #[strong]
        store,
        #[strong]
        accounts,
        #[weak]
        account_list,
        move |_| {
            let store = store.clone();
            let accounts = accounts.clone();
            add_account_dialog::show(&window, move |result| {
                if let Some(account) = result {
                    if let Err(e) = store.add(account.clone()) {
                        eprintln!("Failed to persist account: {e}");
                        return;
                    }
                    let row = make_account_row(&account);
                    account_list.append(&row);
                    accounts.borrow_mut().push(account);
                }
            });
        }
    ));

    // Row activation handler: open edit dialog for the selected account.
    account_list.connect_row_activated(clone!(
        #[weak]
        window,
        #[strong]
        store,
        #[strong]
        accounts,
        #[weak]
        account_list,
        move |_, row| {
            let index = row.index() as usize;
            let account = {
                let list = accounts.borrow();
                match list.get(index) {
                    Some(a) => a.clone(),
                    None => return,
                }
            };
            let store = store.clone();
            let accounts = accounts.clone();
            let row_ref = row.clone();
            edit_account_dialog::show(&window, account, move |result| {
                if let Some(updated) = result {
                    if let Err(e) = store.update(updated.clone()) {
                        eprintln!("Failed to persist account update: {e}");
                        return;
                    }
                    // Replace the sidebar row to reflect colour/badge changes.
                    let new_row = make_account_row(&updated);
                    let idx = row_ref.index();
                    account_list.remove(&row_ref);
                    account_list.insert(&new_row, idx);
                    // Update the in-memory list.
                    let idx = idx as usize;
                    let mut list = accounts.borrow_mut();
                    if idx < list.len() {
                        list[idx] = updated;
                    }
                }
            });
        }
    ));

    window.present();
}

fn make_account_row(account: &Account) -> adw::ActionRow {
    use crate::core::Protocol;

    let subtitle = format!(
        "{} – {}:{}",
        account.protocol(),
        account.host(),
        account.port()
    );

    let row = adw::ActionRow::builder()
        .title(account.display_name())
        .subtitle(&subtitle)
        .activatable(true)
        .build();

    // FR-13, AC-6: display account avatar alongside the account name.
    if let Some(path) = account.avatar_path() {
        let avatar = gtk::Image::builder()
            .pixel_size(32)
            .valign(gtk::Align::Center)
            .build();
        avatar.set_from_file(Some(path));
        row.add_prefix(&avatar);
    }

    // FR-14: account colour stripe as a leading prefix indicator.
    if let Some(color) = account.color() {
        let hex = color.to_hex();
        let css_class = format!("account-color-{}", &hex[1..]);
        let stripe = gtk::Box::builder()
            .width_request(4)
            .height_request(32)
            .valign(gtk::Align::Center)
            .css_classes([css_class.as_str()])
            .build();
        let css = format!(".{css_class} {{ background-color: {hex}; border-radius: 2px; }}");
        let provider = gtk::CssProvider::new();
        provider.load_from_data(&css);
        gtk::style_context_add_provider_for_display(
            &stripe.display(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        row.add_prefix(&stripe);
    }

    // FR-11: visually distinguish POP3 accounts with a suffix badge.
    if account.protocol() == Protocol::Pop3 {
        let badge = gtk::Label::builder()
            .label("POP3")
            .css_classes(["caption", "accent"])
            .valign(gtk::Align::Center)
            .build();
        row.add_suffix(&badge);
    }

    row
}
