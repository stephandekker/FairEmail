# User Story 8 — Sync engine + pending operations + mark-read

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Introduce the background sync engine that owns network I/O for accounts, with one operation type wired through end-to-end.

- A migration adds the `pending_operations` table per FR-4: id, account_id (FK), kind, payload, state (`pending`/`in-flight`/`failed`), retry count, last error, created timestamp.
- A `tokio` multi-threaded runtime hosted on a dedicated worker thread (decision D-1). The engine spawns one task per active account that owns its IMAP connection.
- Per-account ordered processing of `pending_operations` with batching where the protocol allows (e.g. several STORE flag changes coalesced into one IMAP command).
- One operation type implemented: **STORE flags** for read/unread (mark-read / mark-unread).
- A change-notification mechanism: the engine emits typed events (`MessageFlagsChanged`, `OperationFailed`, etc.) on a `tokio::sync::broadcast` channel. The GTK side subscribes via a `glib::MainContext` idle source that drains the receiver and forwards to UI listeners.
- Transient failures (network down, timeout, 5xx-equivalent) re-queue with exponential backoff (5s, 30s, 2m, 10m, 1h, capped). Permanent failures (auth rejected, message gone, quota) mark the operation row `failed` and surface to the UI status area.
- Toggling read in the UI: writes the new flag to `messages` immediately, inserts a `pending_operations` row before the call returns, and the UI subscribes to the change notification to refresh views.

The engine in this story handles only mark-read. Move/delete/send/folder ops are added by later stories (#11, #12). IMAP IDLE is #10. Incremental fetch via CONDSTORE is #9.

## Acceptance criteria

- [ ] Toggling read on a message in the UI updates `messages.flags` immediately and inserts a `pending_operations` row before the UI handler returns.
- [ ] The engine drains `pending_operations` in insertion order per account and applies the STORE flag on the server. A test using the real-IMAP path against a controlled server confirms.
- [ ] With the network disconnected at toggle time, the row remains in `pending_operations` with state `pending`. Reconnecting drains it. The UI does not block waiting.
- [ ] A simulated transient failure causes the operation to be re-queued with backoff (no infinite tight retry loop).
- [ ] A simulated permanent failure (e.g. auth-rejected error code) marks the operation `failed` and emits a status notification consumable by the UI.
- [ ] Change notifications fire when `messages.flags` changes; a UI test subscriber receives the event.
- [ ] The GTK main loop is never blocked: a stress test marking 1000 messages does not freeze the UI thread.
- [ ] Mock implementations of all touched traits remain. `cargo test` passes without network.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #3 (engine reads credentials from the keychain to authenticate).
- Blocked by #6 (engine reads/writes `messages` and uses real IMAP fetch infra).

## User stories addressed

From the parent epic:

- US-26, US-29 (basic broadcast + glib bridge)
- US-30, US-31

Functional requirements: FR-4 (`pending_operations`), FR-37, FR-38 (single IMAP connection per account, owned by engine), FR-39 (STORE flag batching), FR-40, FR-41, FR-42. Non-functional: NFR-1, NFR-2, NFR-11.
