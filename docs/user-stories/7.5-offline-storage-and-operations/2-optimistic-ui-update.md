# Optimistic UI Update on Action

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, when I perform an action on a message, I want the UI to reflect the change instantly — without waiting for the server — so that I am never blocked by network latency or disconnection.

## Acceptance Criteria
- When a user marks a message as read, the message list updates to show "read" state immediately, regardless of connectivity.
- When a user moves a message to another folder, it disappears from the source folder view and appears in the destination folder view immediately.
- When a user flags/unflags a message, the flag state updates in the UI immediately.
- When a user deletes a message, it disappears from the current view immediately.
- The local UI state always reflects the user's most recent action, even when multiple actions are performed in rapid succession.
- Any user action completes its local acceptance and UI update in well under one second (NFR-1).

## Complexity
Medium

## Blocked by
1-persist-operation-record

## HITL/AFK
AFK

## Notes
- The optimistic UI update is described in the epic as "non-negotiable" (Design Note N-1). No "pending" spinners or "waiting for server" states are acceptable for basic mail actions.
- This story covers the *local state change* that accompanies operation creation. The two are logically coupled: create the record AND update the UI atomically from the user's perspective.
