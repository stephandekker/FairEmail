# User Story 7 — FTS5 search index

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Add full-text search over messages without exposing it to the UI.

- A migration creates the `messages_fts` FTS5 virtual table mirroring `subject` and `body_text` from `messages`, content-rowid linked to `messages.id`.
- The mirror is kept consistent with `messages` either via SQLite triggers (`AFTER INSERT`/`AFTER UPDATE`/`AFTER DELETE`) or via explicit upserts in the same transaction as the message write — pick one and apply it consistently.
- The body-text source for FTS is the `body_text` column produced by user story #6 (decision D-3: `mail-parser` + `html2text`).
- This story does **not** define a search API or query syntax for the UI. Search-feature epics (5.x) consume the index later.

## Acceptance criteria

- [ ] Inserting a `messages` row populates `messages_fts` with the same id, subject, and body text. A test asserts this via direct SQL.
- [ ] Updating `subject` or `body_text` on a `messages` row updates `messages_fts`.
- [ ] Deleting a `messages` row deletes the matching `messages_fts` row.
- [ ] An FTS5 `MATCH` query against a known indexed term returns the expected row(s). A test asserts this via direct SQL against an in-memory SQLite database.
- [ ] Re-running migrations on a database that already has `messages_fts` is a no-op.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #6 (needs `messages` and `body_text`).

## User stories addressed

From the parent epic:

- US-5

Functional requirements: FR-43, FR-44, FR-45.
