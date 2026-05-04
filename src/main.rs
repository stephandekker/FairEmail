pub mod core;
pub mod services;
#[cfg(feature = "ui")]
mod ui;

#[cfg(feature = "ui")]
fn main() {
    use glib::clone;
    use gtk4::prelude::*;
    use libadwaita as adw;
    use std::rc::Rc;

    use crate::services::{AccountStore, SettingsStore};

    fn data_dir() -> std::path::PathBuf {
        let base = glib::user_data_dir().join("alarm-clock");
        std::fs::create_dir_all(&base).expect("could not create data directory");
        base
    }

    let app = adw::Application::builder()
        .application_id("com.example.AlarmClock")
        .build();

    app.connect_activate(clone!(move |app| {
        let dir = data_dir();
        let store = Rc::new(AccountStore::new(dir.join("accounts.json")));
        let settings_store = Rc::new(SettingsStore::new(dir.join("settings.json")));
        ui::window::build(app, store, settings_store);
    }));

    app.run();
}

#[cfg(not(feature = "ui"))]
fn main() {
    eprintln!("This binary requires the 'ui' feature. Build with: cargo build --features ui");
}
