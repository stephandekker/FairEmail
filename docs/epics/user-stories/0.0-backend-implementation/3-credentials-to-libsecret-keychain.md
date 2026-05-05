# User Story 3 — Credentials → libsecret keychain

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Move account credentials out of the database and into the OS keychain via the freedesktop Secret Service API.

- A `CredentialStore` trait with read/write/delete keyed by `(account_uuid, role)`, where `role` is one of `imap-password`, `smtp-password`, `oauth-refresh-token`, plus a list-by-account method for cleanup-on-delete.
- A default implementation backed by the `secret-service` crate (decision D-2). Secrets are stored with the attributes `{ "application": "fairmail", "fairmail-account-id": "<uuid>", "role": "<role>" }` so that all of an account's secrets can be enumerated and removed by attribute query when the account is deleted.
- An in-memory mock implementation for tests — unit tests must not require a live D-Bus session bus.
- A migration that, on first run after the credential column has been retired, moves any plaintext credentials still present in the `accounts` row into the keychain in the same transaction that clears the column.
- A locked-keychain or unavailable-D-Bus path that returns a typed error to the caller, which the UI surfaces as "Cannot read account password: system keychain is locked" (or equivalent). The application does not crash.
- Credentials never appear in logs, error messages, or the connection-log dialog. Redaction is enforced at the trait boundary.

## Acceptance criteria

- [ ] After this story merges, the `accounts` row contains no plaintext credential. Inspecting the SQLite file with the `sqlite3` CLI shows no password text.
- [ ] After upgrading, account passwords are visible in the keychain (e.g. via `secret-tool search application fairmail`) with the documented attributes.
- [ ] Deleting an account removes the row **and** every keychain entry whose `fairmail-account-id` matches.
- [ ] With the D-Bus session bus killed (simulated unavailability), opening the application surfaces a clear error per affected operation rather than crashing.
- [ ] Unit tests use the in-memory mock and require neither libsecret nor a session bus.
- [ ] No log line, error message, or connection-log entry contains a credential string. A specific test asserts redaction.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #1 (needs the `accounts` table and migration framework so the plaintext column can be cleared).

## User stories addressed

From the parent epic:

- US-9, US-10, US-11, US-12

Functional requirements: FR-11, FR-12, FR-13, FR-14, FR-15. Non-functional: NFR-5.
