# User Story 13 — Desktop notifications (real)

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Replace the mock `NotificationChannelManager` with a real freedesktop notification client and hook it into the engine's new-message change notifications.

- A real `NotificationChannelManager` implementation backed by [`notify-rust`](https://crates.io/crates/notify-rust) (or direct D-Bus to `org.freedesktop.Notifications`). The mock stays in the codebase.
- A subscriber on the engine's change-notification broadcast channel listens for new-message events and dispatches a desktop notification per affected account.
- Per-account grouping: each account's notifications are tagged with a stable `application` and `category` such that desktop environments that support notification grouping (GNOME, KDE) collapse them under the account name.
- A per-account "notifications enabled" flag on `accounts` is honored. A per-folder "notifications enabled" flag on `folders` is honored. Both are read at dispatch time.
- This story ships only the dispatch path. Notification *content* (preview, sender name, click-to-open, quick-reply) and richer policy (Do Not Disturb, summary mode, per-time-of-day rules) belong to epic 8.1; here, a minimal "1 new message in <folder> on <account>" body is sufficient.

## Acceptance criteria

- [ ] A new mail arrival (delivered via the IDLE path from #10 or via incremental sync from #9) triggers a freedesktop desktop notification.
- [ ] The notification is grouped under the account's display name on a GNOME desktop. (Verified manually; CI does not need a desktop session.)
- [ ] Disabling notifications on an account (via the existing UI toggle) suppresses notifications for that account. A test asserts the dispatcher consults the flag.
- [ ] Disabling notifications on a specific folder suppresses notifications for that folder while other folders on the same account continue to notify.
- [ ] The mock `NotificationChannelManager` remains in the codebase. Unit tests use the mock and do not require a session bus.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #10 (needs the engine's new-message change notifications fired by IDLE/sync to be the source of truth).

## User stories addressed

From the parent epic:

- US-32, US-33

Functional requirements: FR-46, FR-47, FR-48.
