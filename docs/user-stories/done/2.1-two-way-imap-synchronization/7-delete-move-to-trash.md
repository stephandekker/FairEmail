# Delete (Move to Trash)

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As a cautious user, when I delete a message, I want it to be moved to the account's configured Trash folder by default — not permanently removed — so that I have a safety net and can recover it if needed.

## Blocked by
6-move-message-local-to-server

## Acceptance Criteria
- "Delete" action moves the message to the account's configured Trash folder (uses the move pipeline from story 6).
- The message appears in Trash on the server and disappears from its original folder (verifiable via webmail).
- The message remains recoverable from Trash until explicitly expunged or auto-cleaned.
- The user is never surprised by permanent deletion — "delete" always means "move to Trash" unless they explicitly choose permanent deletion.

## HITL / AFK
**AFK** — standard delete-to-trash behavior.

## Estimation
Small — reuses the move pipeline; this story is primarily about the default "delete = move to Trash" contract.

## Notes
- US-4, US-27, FR-26, AC-4 are the primary drivers.
