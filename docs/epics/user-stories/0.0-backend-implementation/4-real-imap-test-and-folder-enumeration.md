# User Story 4 — Real IMAP test, folder enumeration, and connection log

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Replace the IMAP mock implementations with a real IMAP client and wire it through the existing wizard "Test Connection" flow.

- Add `async-imap` (and a `tokio` runtime per decision D-1, hosted on a dedicated worker thread).
- Real implementations for `ImapChecker`, `InboundTester`, and `ConnectionTester`. Mocks stay in the codebase as test doubles.
- TLS modes per the encryption setting on the account: implicit SSL/TLS, STARTTLS, none. Insecure-connection acceptance, DANE, DNSSEC, and pinned-certificate flags from the account row are honored.
- A migration adds a `sync_state` table for the per-account capability cache (`IDLE`, `CONDSTORE`, `QRESYNC`, `UTF8=ACCEPT`, advertised max message size, supported auth mechanisms).
- A migration adds a `folders` table per FR-4. After a successful test, enumerated folders are written to it with role detection from special-use markers (`\Inbox`, `\Sent`, `\Drafts`, `\Trash`, `\Junk`, `\Archive`).
- A migration adds a `connection_log` table (per-account, append-only). The IMAP client appends one row per protocol-level event: connect attempt, TLS handshake outcome, login result, capability list, list folders count, errors. The existing connection-log dialog reads from this table instead of the placeholder source.
- Failed tests surface specific diagnostics (DNS resolution failure, connection refused/timeout, TLS handshake failure, authentication failure, protocol mismatch, untrusted certificate with fingerprint).

POP3 is **not** implemented in this story (decision D-10).

## Acceptance criteria

- [ ] Pressing "Test Connection" in the wizard or edit dialog with valid IMAP credentials against a real IMAP server succeeds, lists folders, and returns within the connection timeout.
- [ ] Folder-role assignment dropdowns (Drafts/Sent/Archive/Trash/Spam) populate from real server data, with auto-detected roles preselected based on special-use markers.
- [ ] After a successful test, the `sync_state` capability cache row for that account contains the capabilities the server advertised.
- [ ] Pressing "Test Connection" with an incorrect password against the same server fails with an authentication-error diagnosis.
- [ ] Pressing "Test Connection" against a non-IMAP port (e.g. SMTP) fails with a protocol-mismatch hint.
- [ ] On TLS handshake failure, the certificate fingerprint is surfaced with a "Trust this certificate" affordance (the trust-store wiring itself is owned by the account record; this story populates the affordance).
- [ ] The connection-log dialog for the account shows real protocol events (connect, TLS, capability list, login result, list folders, errors) as they occur during the test.
- [ ] Mock IMAP implementations remain in the codebase and are used by all unit tests. `cargo test` passes without any network dependency.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #1 (needs the database for `sync_state`, `folders`, and `connection_log` tables).

## User stories addressed

From the parent epic:

- US-18, US-19, US-20 (capability detection portion; the downgrade fetch path lives in #9)
- US-22, US-36

Functional requirements: FR-22, FR-23, FR-24, FR-25, FR-26, FR-30. Plus the `folders`, `sync_state`, and `connection_log` tables from FR-4.
