# User Story 2 — Settings + Order stores moved to SQLite

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Move the two remaining JSON-backed stores into the SQLite database from user story #1.

- A migration adds a `settings` table (key/value, value as JSON-encoded text) and an `account_order` representation (either a column on `accounts` or a tiny dedicated table — choose the simpler one at implementation time).
- `SettingsStore` and `OrderStore` keep their public traits; SQLite-backed implementations replace the JSON-file impls as the production wiring. Mocks remain.
- On first run, if `settings.json` and/or `order.json` exist, their contents are imported into the new tables in a single transaction; each file is then renamed `*.migrated`. Migration is idempotent.
- Existing UI consumers (preferences dialogs, account-list ordering) continue to work unchanged.

Decision D-8 (two-release compat window) applies.

## Acceptance criteria

- [ ] On a fresh install, the `settings` table and the `account_order` representation exist after migrations run.
- [ ] On an upgrade install with pre-existing `settings.json` and `order.json`, their contents are imported on first run; both files are renamed `*.migrated`.
- [ ] Re-running the application is a no-op: no duplicate keys, no reorder.
- [ ] `SettingsStore` and `OrderStore` public method signatures are unchanged.
- [ ] Mock implementations of both stores continue to compile and are used in unit tests.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #1 (needs the SQLite database and migration framework).

## User stories addressed

From the parent epic:

- US-4 (`settings` and account-order tables only)
- US-6 (settings.json + order.json portions)
- US-7, US-8

Functional requirements: FR-4 (`settings`, `account_order`), FR-6 (settings, order), FR-7, FR-9, FR-10.
