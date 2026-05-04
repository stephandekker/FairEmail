# Fairmail Email client for Linux

## Overview

Fairmail is an Email client for Linux

## Claude skipping files

Do not read/use/write any of the files in: /claude-skip/

## System Requirements

## Tech Stack

- **Language:** Rust (stable toolchain, edition 2021)
- **UI:** GTK4 + libadwaita via `gtk4-rs` (`gtk4`, `libadwaita`, `glib`) crates
- **Build:** Cargo; Meson optional if packaging for Flatpak/system install
- **Async runtime:** `glib::MainContext` for UI tasks; `tokio` only if non-UI async work is needed
- **Platform:** Linux (developed and tested on GNOME)
- **Database:** Mocked until choice is made
- **Auth:** Mocked until choice is made

## Project Structure

```
fairmail/
├── CLAUDE.md
├── Cargo.toml
├── Cargo.lock
├── .env                          # Configuration
├── src/
│   ├── main.rs                   # Application entry point (adw::Application)
│   ├── application.rs            # AdwApplication subclass
│   ├── window.rs                 # Main AdwApplicationWindow
│   ├── ui/                       # Widgets, composite templates (.ui / .blp files)
│   ├── core/                     # Domain logic: accounts, emails, attachments (UI-free)
│   └── services/                 # I/O, persistence, notifications
├── data/
│   ├── resources/                # GResource bundle (UI files, icons, CSS)
│   └── *.desktop, *.metainfo.xml # Linux app metadata
├── tests/                        # Integration tests
└── docs/
    └── epics/
        └── user-stories/
            ├── tasks/
            └── bugs/
```

## Bug template
Bugs should be written to .md file in /docs/epics/user-stories/bugs using filename format: "yyyy-MM-dd_hh-mm-ss, {TITLE}.md" where the date-time is the discovery.
The content of the file:
- Title
- Description
- Steps to reproduce
- Expected
- Actual
- Additional context info

## Coding Standards
- Rust 2021 edition; code must pass `cargo fmt --check` and `cargo clippy -- -D warnings`
- Prefer `Result<T, E>` with `thiserror`-derived error types over `unwrap()`/`expect()`; `unwrap()` is only acceptable in tests or proven-unreachable cases (with a comment)
- No `panic!` in library/core code paths
- UI code uses GTK4 + libadwaita idioms:
  - Subclass widgets via `glib::Object` + `glib::wrapper!` rather than packing in code where a `.ui`/`.blp` template fits
  - Use `Adw*` widgets (`AdwApplicationWindow`, `AdwHeaderBar`, `AdwToastOverlay`, `AdwPreferences*`) instead of plain GTK equivalents where available
  - Follow the GNOME Human Interface Guidelines
- Keep `core/` UI-free so it stays unit-testable without a display server. 
- Very high preference to put business logic in core/ where it can be extensively unit tested. Only use business logic in UI if it significantly impacts UX or if no other option.
- Use `gettext-rs` (`gettext!`) for all user-facing strings — no hardcoded English in widgets
- Async on the UI thread uses `glib::MainContext::spawn_local`; never block the main loop with `std::thread::sleep` or sync I/O
- Resources (UI files, icons, CSS) loaded via `gio::Resource` (compiled from `data/resources/`), not from disk paths at runtime
- Module visibility: prefer `pub(crate)` over `pub` unless the item is genuinely part of a public API

## Running the Project
```bash
# Build (debug)
cargo build

# Run the app
cargo run

# Build optimized release
cargo build --release

# Format check / auto-format
cargo fmt --check
cargo fmt

# Lint (treat warnings as errors)
cargo clippy --all-targets -- -D warnings

# Run all tests (unit + integration)
cargo test

# Run a single test
cargo test <test_name>

# Compile GResource bundle (if not handled by build.rs)
glib-compile-resources data/resources/resources.gresource.xml \
    --target=target/resources.gresource

# Validate AppStream metadata before release
appstreamcli validate data/*.metainfo.xml

# Validate .desktop file
desktop-file-validate data/*.desktop
```

### System dependencies (Debian/Ubuntu)
```bash
sudo apt install build-essential libgtk-4-dev libadwaita-1-dev \
    libglib2.0-dev gettext desktop-file-utils appstream
```

### System dependencies (Fedora)
```bash
sudo dnf install gtk4-devel libadwaita-devel glib2-devel gettext \
    desktop-file-utils appstream
```
