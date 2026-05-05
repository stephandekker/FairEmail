# User Story 9 — Incremental sync (CONDSTORE/QRESYNC)

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Replace the "always fetch everything" path from user story #6 with proper incremental sync against modern IMAP servers, plus a graceful downgrade for older ones.

- On reconnect to a folder, read the cached `uidvalidity` and `highestmodseq`.
- If the server advertises CONDSTORE/QRESYNC (capability cache from #4): issue a `SELECT ... (CONDSTORE)` and `FETCH ... CHANGEDSINCE <highestmodseq>` to pull only messages whose flags or content have changed. Persist the new `HIGHESTMODSEQ`.
- If the server advertises neither: fall back to a UID-set diff — list current UIDs, compare with the local set, fetch only the difference.
- On `UIDVALIDITY` change: invalidate cached UIDs for that folder, drop stale `messages`/`message_folders` rows for that folder (keeping `.eml` files for any hash still referenced elsewhere via the reference-count delete from #6), and re-fetch from scratch.
- Flag-change notifications from the server (e.g. another client marks a message read) are reflected in `messages.flags` and emit change notifications on the engine's broadcast channel.

## Acceptance criteria

- [ ] Second sync of an unchanged folder fetches **zero** message bodies. A test against a controlled CONDSTORE-capable server asserts via fetch counters.
- [ ] One server-side new message between syncs results in exactly one `BODY[]` fetch on the next sync.
- [ ] One server-side flag change between syncs results in zero `BODY[]` fetches but updates `messages.flags` and emits a change notification.
- [ ] A simulated `UIDVALIDITY` change invalidates cached UIDs: stale rows for that folder are dropped, the folder is re-fetched, and `.eml` files referenced only by the dropped rows are deleted (others retained).
- [ ] Against a server with neither CONDSTORE nor QRESYNC, the UID-set-diff fallback fetches only new messages on the second sync. A test asserts.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #6 (needs `messages` table and full-fetch path as the baseline being replaced).
- Blocked by #8 (engine owns the connection lifecycle that drives sync).

## User stories addressed

From the parent epic:

- US-20 (downgrade fetch path)
- US-27 (full incremental sync, including `UIDVALIDITY` invalidation)

Functional requirements: FR-28.
