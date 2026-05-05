# User Story 12 — Folder management end-to-end

**MoSCoW:** MUST · **Type:** AFK

## Parent Feature

[#0.0 Backend Implementation](../../0.0-backend-implementation.md) · [Decisions](../../0.0-backend-implementation-decisions.md)

## What to build

Replace the mock `FolderSyncService` so that folder operations issued from the UI actually reach the IMAP server.

- New `pending_operations.kind` values: `folder-create`, `folder-rename`, `folder-delete`. Payloads carry the folder id and any rename target.
- A real `FolderSyncService` whose trait methods enqueue rows in `pending_operations` and await the engine's completion notification, rather than performing IMAP work inline.
- The engine handles each op by issuing the corresponding IMAP command (`CREATE`, `RENAME`, `DELETE`), updating `folders` rows on success, and emitting `FolderListChanged` change notifications.
- Failure handling matches #8: transient errors retry; permanent errors (folder name in use, permission denied) mark the op `failed` and surface a specific diagnostic to the UI.

## Acceptance criteria

- [ ] Creating a folder in the application creates it on the IMAP server. The local `folders` table reflects the new row.
- [ ] Renaming a folder in the application renames it on the server. Messages associated with the folder remain associated after the rename.
- [ ] Deleting a folder in the application deletes it on the server. Messages whose only folder association was the deleted folder are removed; their `.eml` files are reclaimed via the reference-count delete from #6 if no other row references the same hash.
- [ ] A `FolderListChanged` change notification fires after each operation; UI subscribers refresh accordingly.
- [ ] A simulated permission-denied response (e.g. attempting to delete a system folder) surfaces a specific error and leaves local state unchanged.
- [ ] The mock `FolderSyncService` remains in the codebase for unit tests. `cargo test` passes without network.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.

## Blocked by

- Blocked by #8 (engine + `pending_operations`).

## User stories addressed

From the parent epic:

- US-34, US-35

Functional requirements: FR-39 (engine processes folder ops), FR-40 (failure handling), FR-42 (change notifications).
