# User Story 11 — Real SMTP send via the engine

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Make sending a message work end-to-end through the sync engine and persistent queue.

- A new `pending_operations.kind = "send"` payload carries the identity id and a reference to the composed message bytes (either inline or via a content-store hash if the draft was already stored).
- When the engine processes a send op, it: (a) reads SMTP credentials from the keychain (decision D-2), (b) opens an SMTP connection via `lettre`, (c) authenticates and submits, (d) on success, appends a copy to the IMAP Sent folder using the inbound connection's IMAP `APPEND`, (e) writes the message bytes to the content store, (f) inserts a `messages` row in the Sent folder with appropriate flags (`\Seen`, `\Answered` if applicable), (g) deletes the `pending_operations` row.
- The "login before send" identity flag (FR-54 of epic 1.4): when set, the engine verifies an inbound IMAP login succeeds before attempting submission.
- Failure handling matches #8: transient errors retry with backoff; permanent errors mark the op `failed` and surface to the UI.
- The compose UI's "Send" button writes the draft and inserts the send op; it does not block on delivery.

## Acceptance criteria

- [ ] Composing and sending a message via a configured identity delivers it. A test using a real SMTP server (or recorded interaction) confirms.
- [ ] After successful send, the message exists on the server's Sent folder (via IMAP `APPEND`).
- [ ] After successful send, the message is also in the local content store and `messages` table, linked to the local Sent folder, with the `\Seen` flag.
- [ ] With "login before send" enabled, a deliberately-broken inbound credential causes the send to abort before SMTP submission.
- [ ] A simulated SMTP transient failure leaves the op queued for retry; reconnecting succeeds drains it. A simulated permanent failure (auth rejected) marks the op `failed` and surfaces to the UI.
- [ ] The compose UI's Send button returns immediately; a stress test sending 50 messages does not block the GTK thread.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #3 (SMTP credentials live in the keychain).
- Blocked by #5 (real SMTP client and `identities` table).
- Blocked by #8 (engine processes the queued send op).

## User stories addressed

From the parent epic:

- US-25

Functional requirements: FR-35, FR-36.
