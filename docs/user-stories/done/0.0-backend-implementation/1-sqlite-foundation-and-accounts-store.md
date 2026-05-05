# User Story 1 — SQLite foundation + accounts store

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

The first vertical slice that puts SQLite in the application. End-to-end:

- The application opens **one** SQLite database at startup at `$XDG_DATA_HOME/fairmail/fairmail.db` (overridable via `FAIRMAIL_DATA_DIR`), with WAL journal mode, foreign keys on, and `synchronous=NORMAL`.
- A versioned migration framework runs pending migrations on startup and records the schema version (PRAGMA `user_version` or a `schema_migrations` table).
- The first migration creates the `accounts` table per the FR-4 column list in the epic. Credentials remain a column for now — moving them to the keychain is user story #3.
- `AccountStore`'s public trait is unchanged; a new SQLite-backed implementation replaces the JSON-file impl as the production wiring in `main.rs`. The mock implementation stays available for tests.
- On first run, if `accounts.json` exists in the previous data directory, its contents are imported into the `accounts` table in a single transaction, then the file is renamed `accounts.json.migrated` (not deleted). Migration is idempotent.
- The wizard, account list, and edit dialog continue to compile and work without modification.

Decisions D-7 (single shared DB), D-4 (schema shape), D-8 (two-release compat window), D-10 (POP3 column accepted) apply.

## Acceptance criteria

- [ ] On a fresh install, opening the application creates `$XDG_DATA_HOME/fairmail/fairmail.db` with `journal_mode=wal` and `foreign_keys=on`. The `accounts` table exists with the FR-4 columns.
- [ ] Setting `FAIRMAIL_DATA_DIR=/tmp/test-data` and starting the application places `fairmail.db` under that path; unsetting reverts to the XDG default.
- [ ] On an upgrade install with a pre-existing `accounts.json` containing one or more accounts, opening the application imports those accounts into the `accounts` table inside a single transaction. After migration, `accounts.json.migrated` exists at the original path; `accounts.json` does not.
- [ ] Re-running the application a second time is a no-op: no duplicate rows, no error.
- [ ] If migration fails partway (simulated by injecting an error), the JSON file is **not** renamed and the database has no partial accounts.
- [ ] The wizard, account list, and edit dialog operate against `AccountStore` unchanged. No public method on `AccountStore` has changed signature.
- [ ] The mock `AccountStore` continues to compile and is the implementation used in unit tests.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

None — can start immediately.

## User stories addressed

From the parent epic:

- US-1, US-2, US-3
- US-4 (the `accounts` table only; other tables are introduced by later slices)
- US-6 (accounts.json portion only)
- US-7 (AccountStore trait shape preserved)
- US-8 (mock retained)
- US-38 (XDG path conventions, env-var override)

Functional requirements: FR-1, FR-2, FR-3, FR-4 (accounts row), FR-5, FR-6 (accounts), FR-7 (accounts), FR-9, FR-10, FR-51.
