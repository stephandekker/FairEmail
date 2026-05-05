# User Story 5 — Real SMTP test

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Replace the SMTP mock implementation with a real SMTP client and wire it through the existing identity-dialog "Test Connection" flow.

- Add `lettre` (using its `tokio1`/`rustls` features per decision D-1).
- A real implementation for `SmtpChecker` that connects, performs the chosen TLS handshake (SSL/TLS implicit, STARTTLS, or none), authenticates, queries EHLO, and reports results. The mock stays in the codebase.
- A migration adds an `identities` table per FR-4: account_id (FK), email address, display name, SMTP host/port/encryption/username/realm, advanced flags (`use_ip_in_ehlo`, `custom_ehlo`, `login_before_send`), and a `max_message_size_cache` column.
- On a successful test, the server's advertised maximum message size from EHLO is captured and persisted on the identity row.
- Failed tests surface specific diagnostics (DNS, connect, TLS, authentication, EHLO failure).
- The identity dialog continues to compile and operate against the same `SmtpChecker` trait shape.

The actual *send* path (deliver a real message, append to Sent, store locally) is user story #11.

## Acceptance criteria

- [ ] Pressing "Test Connection" in the identity dialog with valid SMTP credentials against a real SMTP server succeeds.
- [ ] The server's advertised max message size is displayed in the dialog and persisted to `identities.max_message_size_cache`.
- [ ] Pressing "Test Connection" with an incorrect password fails with an authentication-error diagnosis.
- [ ] Pressing "Test Connection" against a non-SMTP port fails with a protocol/connect-error diagnosis.
- [ ] The `identities` table exists with the FR-4 columns; existing UI consumers compile against the same `SmtpChecker` trait shape.
- [ ] Mock `SmtpChecker` remains and is used by unit tests. `cargo test` passes without network access.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #1 (needs the database and migration framework for `identities`).

## User stories addressed

From the parent epic:

- US-23, US-24

Functional requirements: FR-31, FR-32, FR-33, FR-34. Plus the `identities` table from FR-4.
