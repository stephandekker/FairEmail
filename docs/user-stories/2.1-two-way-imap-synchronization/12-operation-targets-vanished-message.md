# Graceful Handling of Operations Targeting Vanished Messages

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, if I have queued an operation (e.g. move, flag) on a message that another client deletes from the server before my operation executes, I want the application to handle this gracefully — no crash, no ghost entries — and inform me that the operation could not complete.

## Blocked by
6-move-message-local-to-server, 11-message-removal-detection

## Acceptance Criteria
- When executing an operation against a message that no longer exists on the server, the operation is cancelled gracefully.
- The local message entry is removed (it no longer exists on the server).
- The user is informed that the operation could not complete (via the operation queue UI or a notification).
- No crash, no ghost message, no inconsistent state.
- If the vanished message was part of a batch operation, only the affected entry fails; the rest of the batch proceeds.

## HITL / AFK
**AFK** — the resolution is deterministic; the user is informed after the fact.

## Estimation
Small — straightforward error path on top of the move/flag execution logic.

## Notes
- US-18, FR-16, AC-15 are the primary drivers.
- OQ-7 (handling vanished messages during batch operations) is relevant; the epic's source application retries individually on batch failure.
