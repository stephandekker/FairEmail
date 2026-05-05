# User Story 6 — Messages table, content store, and full-folder fetch

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

End-to-end pipeline that pulls messages off an IMAP server, writes them to disk, and indexes them in SQLite.

- A migration adds the `messages` table per FR-4 (id, account_id, uid, modseq, message-id, in-reply-to, references, parsed from/to/cc/bcc, subject, date received/sent, flags bitmask, size, `content_hash`, `thread_id`, `server_thread_id`) plus the `message_folders` join table.
- A `ContentStore` trait with `put`, `get`, `delete`, `exists`, plus a filesystem implementation rooted at `$XDG_DATA_HOME/fairmail/messages/` that writes each `.eml` to `<root>/<aa>/<bb>/<sha256>.eml`. Writes are atomic (temp file + rename in the same directory). `put` is idempotent (same bytes → same hash → no duplicate file).
- An in-memory mock `ContentStore` for tests.
- A reference-count delete path (decision D-6): when a `messages` row is deleted, count remaining rows with the same `content_hash`; if zero, delete the file.
- A first-pass fetch routine that, given an account and folder, fetches every message's raw RFC 5322 bytes (`BODY[]`), writes the bytes to the content store, parses headers with `mail-parser`, derives `body_text` for FTS (`text/plain` preferred, otherwise HTML stripped via `html2text` per decision D-3), and inserts the `messages` and `message_folders` rows in a single transaction.
- The folder's `uidvalidity` and `highestmodseq` are recorded on the `folders` row after the fetch. (Using them for incremental sync is user story #9.)
- A small dev-only sanity surface — either a CLI flag or a debug menu item — that triggers a fetch of one folder and prints row counts. Production UI for the inbox is owned by epics 3.x.

FTS5 indexing is **not** implemented in this story; that's user story #7.

## Acceptance criteria

- [ ] After running the dev fetch path against a real IMAP server, every message in the chosen folder has a corresponding `.eml` file under `$XDG_DATA_HOME/fairmail/messages/<aa>/<bb>/<sha256>.eml`.
- [ ] Two identical messages (same raw bytes, e.g. delivered to two folders) share one `.eml` file. The `messages` table has two rows linked via `message_folders` to the appropriate folder rows.
- [ ] Deleting a `messages` row removes the `.eml` only when no other row references the same `content_hash`. A test asserts both branches.
- [ ] The folder's `uidvalidity` and `highestmodseq` are recorded after the fetch.
- [ ] `body_text` derivation prefers `text/plain` over `text/html`. A test asserts the fallback path strips HTML.
- [ ] Mock `ContentStore` allows unit tests to run with no disk access. `cargo test` passes without network.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #1 (database/migration framework).
- Blocked by #4 (real IMAP fetch capability and `folders` table).

## User stories addressed

From the parent epic:

- US-13, US-14, US-15, US-16
- US-21
- US-27 (record `uidvalidity`/`highestmodseq` only — using them is #9)

Functional requirements: FR-4 (`messages`, `message_folders`), FR-16, FR-17, FR-18, FR-19, FR-20, FR-21 (rebuild path itself is #14, but `delete` semantics are here), FR-27. Plus thread-id columns per decision D-5.
