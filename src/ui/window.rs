use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::Account;
use crate::services::AccountStore;
use crate::ui::add_account_dialog;

/// Build the main application window with the account list and navigation pane.
pub(crate) fn build(app: &adw::Application, store: Rc<AccountStore>) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(gettext::gettext("Alarm Clock – Accounts"))
        .default_width(720)
        .default_height(480)
        .build();

    let split_view = adw::NavigationSplitView::new();

    // -- Sidebar: account list --
    let sidebar_header = adw::HeaderBar::new();
    let add_btn = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(gettext::gettext("Add account"))
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
            .label(gettext::gettext("No accounts yet"))
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
        .title(gettext::gettext("Accounts"))
        .child(&sidebar_toolbar)
        .build();

    // -- Content pane (placeholder) --
    let content_header = adw::HeaderBar::new();
    let content_label = gtk::Label::builder()
        .label(gettext::gettext("Select an account"))
        .css_classes(["dim-label"])
        .vexpand(true)
        .build();
    let content_toolbar = adw::ToolbarView::new();
    content_toolbar.add_top_bar(&content_header);
    content_toolbar.set_content(Some(&content_label));

    let content_page = adw::NavigationPage::builder()
        .title(gettext::gettext("Details"))
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

    window.present();
}

fn make_account_row(account: &Account) -> adw::ActionRow {
    adw::ActionRow::builder()
        .title(account.display_name())
        .subtitle(format!(
            "{} – {}:{}",
            account.protocol(),
            account.host(),
            account.port()
        ))
        .activatable(true)
        .build()
}
