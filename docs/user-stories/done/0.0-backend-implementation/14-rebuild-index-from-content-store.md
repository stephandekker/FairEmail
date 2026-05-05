# User Story 14 — Rebuild index from content store

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

An operator-invoked recovery path that reconstructs the SQLite index from the on-disk `.eml` files, so that a corrupt or deleted database is not a data-loss event.

- A `--rebuild-index` (name TBD) CLI flag on the application that, on startup, walks `$XDG_DATA_HOME/fairmail/messages/` and re-parses every `.eml` file.
- For each `.eml`: compute the SHA-256 (verifying the filename), parse headers with `mail-parser`, derive `body_text`, and insert/upsert a row in `messages` (and the FTS5 mirror). The `content_hash` matches the file basename so the link to disk is restored.
- Folder associations: where the `.eml` carries headers from which a folder hint can be inferred (e.g. `X-Folder` if previously preserved, or the `Received` chain), use them. Otherwise, drop the message into a synthetic per-account "Recovered" folder. The user re-binds messages to their canonical folders on next sync via UID matching against the server.
- Account association: `.eml` files do not carry an account id. The rebuild path requires that `accounts` rows still exist (or were re-imported via #1 from `accounts.json.migrated`). Messages whose account cannot be inferred go into the most recently used account or, on prompt, are discarded.
- The path is **non-destructive** to the content store: it only reads files, never deletes them.
- Documented in operator notes (a short section in `docs/architecture.md` or a sibling README under `docs/`).

## Acceptance criteria

- [ ] Deleting `fairmail.db` and starting the application with `--rebuild-index` reconstructs the `messages` table from the on-disk `.eml` files. A test creates a known message set, deletes the database, runs rebuild, and asserts row count and a sampled subject match.
- [ ] After rebuild, the FTS5 index is consistent: a known indexed term returns the expected row.
- [ ] `.eml` files on disk are not modified or deleted during rebuild.
- [ ] Rebuild on an empty content store is a no-op.
- [ ] Rebuild over an existing populated database is idempotent (uses upsert on `content_hash`, no duplicate rows).
- [ ] Operator-facing documentation explains how to run the rebuild and what it does and does not reconstruct.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #6 (needs `messages` table and the content-store layout to walk).

## User stories addressed

From the parent epic:

- US-17

Functional requirements: FR-21. Non-functional: NFR-4 (recoverability).
