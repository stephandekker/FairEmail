# Create and Persist Operation Record

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, when I perform an action on a message (mark read, flag, move, delete, etc.), I want that action to be durably recorded locally so that it survives application crashes, unexpected termination, and system reboots — ensuring nothing I do is ever lost.

## Acceptance Criteria
- When a user initiates any mail action, a persistent operation record is created in durable local storage before the UI acknowledges the action.
- Each operation record contains: operation type, target entity (message or folder), owning account, arguments (e.g. destination folder, flag color), and creation timestamp.
- Operation records survive application crash: force-terminating and restarting shows the same operations still pending.
- Operation records survive system reboot.
- The operation queue handles at least 1,000 pending operations without degradation.

## Complexity
Medium

## Blocked by
(none — this is the foundation story)

## HITL/AFK
AFK — no human review needed during implementation beyond normal code review.

## Notes
- This story deliberately does not prescribe the storage format or schema (per NG2 in the epic). The implementation must choose a durable persistence mechanism but the choice is free.
- FR-5 lists all supported operation types. This story only needs to support the *record structure* for all types; individual type semantics are layered in subsequent stories.
- NFR-2 (durability) is the key non-functional constraint: the record must be persisted *before* the UI acknowledges the action to the user.
