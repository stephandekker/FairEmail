# Conflict Resolution for Missing Messages and Folders

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, when an operation targets a message or folder that no longer exists on the server, I want the system to handle it gracefully — silently dropping non-destructive conflicts and informing me about destructive ones — so that normal conflicts do not clutter my attention but important ones are not hidden.

## Acceptance Criteria
- If an operation targets a message that no longer exists on the server, it is abandoned.
- For non-destructive operations on missing messages (mark read, flag), abandonment is silent — no error notification.
- For destructive or structural operations on missing messages (move, copy), the user is notified.
- If a move or copy operation targets a folder that no longer exists on the server, the operation fails permanently and the user is notified.
- If the server's current message state already matches the operation's intent (e.g. message is already flagged), the operation is treated as successfully completed.
- Temporary local message copies created during move operations are cleaned up if the operation fails permanently, preventing orphaned messages.

## Complexity
Medium

## Blocked by
7-permanent-failure-handling

## HITL/AFK
AFK

## Notes
- OQ-3 in the epic asks whether silent abandonment of non-destructive conflicts is appropriate for power users who want full transparency. The initial implementation follows the source application's approach (silent drop). This can be revisited.
- Design Note N-5 explains the rationale: bothering the user about a flag on a deleted message adds noise without value.
