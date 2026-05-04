# Mark Read — Local to Server Round-Trip

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As a multi-device user, when I mark a message as read in the desktop client, I want the Seen flag to be set on the IMAP server within one sync cycle, so that the message appears as read on all my other devices and in webmail.

## Blocked by
1-persistent-operation-queue

## Acceptance Criteria
- Marking a message as read in the UI immediately updates the local message state to "read."
- A "set Seen flag" operation is persisted to the operation queue.
- On the next sync cycle (or immediately if connected), the operation is executed against the server via the appropriate IMAP command.
- After successful server confirmation, the operation is removed from the queue and the local state is marked as "confirmed by server."
- Verifiable via webmail or another IMAP client: the message shows as read.
- If the server is unreachable, the operation remains in the queue (offline behavior is covered in a later story).

## HITL / AFK
**AFK** — straightforward flag-set propagation.

## Estimation
Small — this is the thinnest possible end-to-end slice proving the local→server pipeline.

## Notes
- This story is intentionally narrow: only the Seen flag, only local→server. It serves as the tracer bullet that proves the entire operation-queue-to-server-execution pipeline works end-to-end.
- US-1, FR-1, AC-1 are the primary drivers.
