# User Story 10 — IMAP IDLE for push delivery

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Make new mail surface promptly without manual refresh by maintaining IMAP IDLE per account.

- After initial sync of an account, the engine enters `IDLE` on the inbox if the capability cache (from #4) shows IDLE is supported.
- IDLE is renewed before the 29-minute IMAP timeout (RFC 2177): the engine exits and re-enters IDLE on a fixed interval.
- On an IDLE response indicating new messages or flag changes, the engine triggers an incremental sync of that folder (using the path from #9), fetches the deltas, writes them to the database, and emits change notifications.
- On a transient disconnect (network drop, server reset), the engine reconnects with backoff. While disconnected, no IDLE is held.
- For servers without IDLE: fall back to periodic polling on a fixed interval (the *policy* of the interval is owned by epic 7.2; this story uses a sensible default constant).
- For v1, only the inbox is IDLE'd per account; per-folder IDLE policy is epic 7.1.

## Acceptance criteria

- [ ] An account with an IDLE-capable server enters IDLE on the inbox after initial sync. A connection-log entry records the IDLE-enter event.
- [ ] A new message arriving server-side (delivered via SMTP injection in the test) appears in the local `messages` table within seconds, without any user action and without polling. A test asserts the time-to-arrival.
- [ ] IDLE is exited and re-entered before the 29-minute timeout. A test fast-forwards a clock and asserts the renewal.
- [ ] A simulated network drop terminates IDLE; reconnect with backoff succeeds; IDLE resumes. The connection-log entries reflect each transition.
- [ ] Against a server lacking IDLE, the engine falls back to periodic polling at the default interval. A test asserts the poll cadence.
- [ ] The GTK main loop is unaffected during IDLE. A UI responsiveness test passes.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #8 (engine owns the connection).
- Blocked by #9 (IDLE responses trigger incremental sync, not full refetch).

## User stories addressed

From the parent epic:

- US-28

Functional requirements: FR-29, FR-30 (IDLE enter/exit logged), FR-42 (change notifications on new mail).
