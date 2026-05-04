# Cancel Pending Operation with UI Revert

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, I want to cancel (delete) a pending operation before it has been committed to the server, so that I can undo a mistake without it ever reaching the server — and I want the UI to revert to reflect the cancellation.

## Acceptance Criteria
- The user can delete an individual pending operation from the queue.
- When a queued operation is deleted, the optimistic UI state it introduced is reverted to reflect the server's last known state.
- Deleting a queued move operation reverts the message to its original folder in the local view.
- Deleting a queued flag operation removes the flag from the local view.
- Operations that are currently executing (in-flight to the server) cannot be cancelled.
- The revert is immediate and visible in the message list.

## Complexity
Medium

## Blocked by
2-optimistic-ui-update
14-operations-queue-view

## HITL/AFK
AFK

## Notes
- Design Note N-7 explains the rationale: "cancel this action" must truly mean undo — the message returns to its pre-action state in the UI. Without this, deleting an operation could leave the UI showing a state that neither the server nor any pending operation supports.
- Edge case: if multiple operations are stacked on the same message (e.g. move then flag), cancelling the move must also handle the dependent flag operation. The epic doesn't explicitly address this — implementation should either cascade-cancel dependent operations or prevent cancellation when dependents exist. Flag this during design.
