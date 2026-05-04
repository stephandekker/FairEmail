# Server-Side Draft Synchronization

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As a multi-device user, I want the application to synchronize my draft to the server-side Drafts folder after each save, so that I can continue editing on another device.

## Blocked by
1-local-draft-persistence

## Acceptance Criteria
- When server-side draft saving is enabled (default), a draft that has been auto-saved locally also appears in the account's IMAP Drafts folder on the server. (FR-21, AC-10)
- Server sync pushes the current (latest or undo-navigated) revision body only — revision history itself is not synchronized. (FR-24)
- A toggle in the compose window allows the user to disable server-side draft sync for the current compose session. (FR-22, AC-11)
- Toggling off prevents subsequent saves from pushing to the server, but local auto-saves continue unaffected. (AC-11)
- Server-side draft saving is **enabled by default**. (US-20)
- If the draft requires encryption, the server push is deferred until the user explicitly triggers a send or manual save that includes encryption. (FR-23, AC-15)
- Revision snapshots are never transmitted to the server — only the current draft body. (FR-12, NFR-8)

## Mapping to Epic
- FR-21, FR-22, FR-23, FR-24
- NFR-8 (privacy)
- US-18, US-19, US-20
- AC-10, AC-11, AC-15

## HITL / AFK
AFK — IMAP integration with well-defined push semantics.

## Notes
- **OQ-7 (Server sync frequency):** The epic flags that every content-changing local save triggers a server sync, which could be chatty for fast typists. Debouncing or rate-limiting may be needed. This story implements the 1:1 local-save-to-server-push behaviour as specified; debouncing is deferred to a future decision.

## Estimation
Medium — IMAP Drafts folder interaction, per-session toggle, encryption-deferral logic.
