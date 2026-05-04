# Single-Folder IDLE Session on Inbox

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As a set-and-forget user, when I add an account whose server supports IDLE, I want push to be activated automatically on my Inbox so that new mail is detected within seconds of arriving on the server, without any manual configuration.

## Blocked by
- `1-capability-detection`

## Acceptance Criteria
- When a server supports IDLE (as detected in story 1), the application opens a long-lived IDLE session on the Inbox folder automatically.
- The IDLE session monitors for: new message arrival, message removal/expunge, and flag changes.
- When the server signals a change during IDLE, the application immediately triggers a synchronization action for the Inbox.
- A new message delivered to the Inbox is detected and surfaced within 5 seconds under normal network conditions (NFR-1, AC-4).
- No manual user action is required to activate push on the Inbox (AC-1).
- The IDLE session uses the same transport security (TLS) as regular IMAP connections (NFR-8).

## Mapping to Epic
- US-1
- FR-5, FR-6, FR-7
- NFR-1, NFR-3, NFR-8
- AC-1, AC-4

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story covers only a single folder (Inbox) on a single account. Multi-folder IDLE is story 4.
- This story does not cover what happens after detection (notification display, message download) — only that the change-detection signal fires promptly (NG1).
- The existing codebase starts a dedicated thread per folder running `ifolder.idle(false)` in a loop with MessageCountListener and MessageChangedListener. The new implementation should follow the same per-folder-session model (Design Note N-1).
