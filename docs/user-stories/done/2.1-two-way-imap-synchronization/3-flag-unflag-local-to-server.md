# Flag/Unflag and Answered — Local to Server

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, when I flag/star a message, unflag it, or mark it as answered/forwarded, I want those flags to persist on the IMAP server, so that they are visible from all my devices and clients.

## Blocked by
2-mark-read-local-to-server

## Acceptance Criteria
- Flagging a message creates a "set Flagged" operation and updates local state immediately.
- Unflagging a message creates a "remove Flagged" operation.
- Marking as answered creates a "set Answered" operation.
- Each operation executes against the server and the flag change is verifiable via another client.
- Removing the Seen flag (mark unread) also works via this path.
- If the folder is read-only (server reports it as such), the application does not attempt to write flag changes back and informs the user.

## HITL / AFK
**AFK** — extends the pattern established by the mark-read story.

## Estimation
Small — reuses the pipeline from story 2; adds flag types.

## Notes
- US-2, US-6, FR-1, FR-33, AC-2 are the primary drivers.
- Draft flag is included in FR-31 but draft-specific semantics (OQ-1) are out of scope here.
