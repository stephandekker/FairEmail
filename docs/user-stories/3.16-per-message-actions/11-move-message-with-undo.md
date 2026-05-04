## Parent Feature

#3.16 Per-Message Actions

## What to build

Allow users to move one or more messages to any folder within the same account via a folder picker dialog. After a move, provide a non-blocking undo action for a configurable grace period (default 5 seconds). The move is not committed to the server until the grace period expires or the user explicitly confirms. If the application terminates during the grace period, the action commits on next launch (FR-51, FR-53, NFR-4).

This story establishes the undo grace-period infrastructure (Design Note N-11) that is reused by delete and other destructive actions.

## Acceptance criteria

- [ ] User can move a message to any folder in the same account via a folder picker
- [ ] After moving, a transient undo notification appears for the configured grace period
- [ ] Pressing undo within the grace period returns the message to its original folder (AC-15)
- [ ] If undo is not pressed, the move commits to the server after the grace period
- [ ] Grace period duration is user-configurable (default 5 seconds)
- [ ] If the application terminates during grace period, the move commits on next launch
- [ ] Move operation works from the message list and single-message view

## Blocked by

None — can start immediately.

## User stories addressed

- US-44 (move to any folder via folder picker)
- US-46 (undo option for configurable grace period)
