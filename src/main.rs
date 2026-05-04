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

    use crate::services::AccountStore;

    fn data_dir() -> std::path::PathBuf {
        let base = glib::user_data_dir().join("alarm-clock");
        std::fs::create_dir_all(&base).expect("could not create data directory");
        base
    }

    let app = adw::Application::builder()
        .application_id("com.example.AlarmClock")
        .build();

    app.connect_activate(clone!(move |app| {
        let store = Rc::new(AccountStore::new(data_dir().join("accounts.json")));
        ui::window::build(app, store);
    }));

    app.run();
}

#[cfg(not(feature = "ui"))]
fn main() {
    eprintln!("This binary requires the 'ui' feature. Build with: cargo build --features ui");
}
