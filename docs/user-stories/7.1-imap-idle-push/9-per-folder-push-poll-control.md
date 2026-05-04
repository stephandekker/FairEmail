# Per-Folder Push/Poll Control

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, I want to independently configure each folder to use push (IDLE) or poll mode, so that I can have push on my Inbox and important project folders but polling on archive or bulk-mail folders to reduce resource usage.

## Blocked by
- `4-multi-folder-idle`
- `3-poll-mode-fallback`

## Acceptance Criteria
- Each folder has an independent, persistent setting controlling whether it uses push or poll when its server supports IDLE; the default is push (FR-37).
- The user can switch any folder to poll-only mode via the folder's properties, without affecting other folders on the same account (FR-38, AC-5).
- Manually setting a folder to "poll only" causes the application to stop maintaining an IDLE session for that folder and instead check it on the polling schedule (AC-5).
- Switching a folder back to push mode re-establishes an IDLE session for it.
- The per-folder setting is independent of the account-level auto-optimization state — a user can set specific folders to poll even when the account is in push mode, and vice versa.

## Mapping to Epic
- US-5, US-6
- FR-37, FR-38
- AC-5

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Provider-specific defaults (e.g. defaulting non-Inbox folders to poll on certain providers) are covered in story 17.
- The UI for this setting should be accessible from the folder's properties view. The exact UI design is not prescribed by the epic.
