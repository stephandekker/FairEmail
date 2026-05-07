# Permanent Delete (Expunge)

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, when I permanently delete a message from Trash (or explicitly choose permanent deletion), I want it to be irrecoverably removed from the server, and I want the expunge to be precise — only the messages I chose should be removed.

## Blocked by
7-delete-move-to-trash

## Acceptance Criteria
- "Permanently delete" sets the Deleted flag and issues an expunge command to the server.
- When the server supports per-UID expunge, only the targeted messages are expunged (not unrelated Deleted-flagged messages in the folder).
- When the server does not support per-UID expunge, the application either uses folder-wide expunge with appropriate user warning, or defers expunge to a batch operation.
- After expunge, the message is no longer retrievable on the server.
- The user has a configurable option to control whether expunge happens immediately or is deferred to manual/scheduled batch.

## HITL / AFK
**HITL** — the non-UID-EXPUNGE fallback may require a user warning/confirmation before folder-wide expunge.

## Estimation
Medium — per-UID vs. folder-wide expunge logic, configurability, and user warnings add complexity.

## Notes
- US-5, US-28, US-29, FR-27, FR-28, FR-29, FR-30, AC-5, AC-22 are the primary drivers.
- N-5 explains why per-UID expunge is critical for safety.
- OQ-3 (expunge timing for non-UID-EXPUNGE servers) is an open question in the epic. This story should document whichever approach is chosen but the epic does not prescribe a single answer.
