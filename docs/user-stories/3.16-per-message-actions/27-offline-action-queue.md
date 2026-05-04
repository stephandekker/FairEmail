## Parent Feature

#3.16 Per-Message Actions

## What to build

Ensure that all per-message actions that modify server state are queueable when offline. The queue must survive application restart and replay operations in order when connectivity returns. This covers flag, move, delete, keywords, subject edit, attachment deletion, and any other server-syncing action (FR-78, NFR-2).

## Acceptance criteria

- [ ] Performing a flag toggle while offline queues the operation locally
- [ ] Performing a move while offline queues the operation locally
- [ ] Performing a delete while offline queues the operation locally
- [ ] Queued operations survive application restart
- [ ] When connectivity returns, queued operations replay in order
- [ ] The user sees local state updates immediately (optimistic UI), even while offline
- [ ] Conflicts (e.g. message deleted on server while offline) are handled gracefully

## Blocked by

None — can start immediately (but should be validated after individual action stories are complete).

## User stories addressed

- (Cross-cutting requirement FR-78; touches all server-syncing actions)

## Notes

- The source application (EntityOperation / DaoOperation) already has a comprehensive operation queue. This story ensures the desktop application provides equivalent infrastructure. It can be started early as foundational infrastructure or validated as an integration pass after actions are built.
