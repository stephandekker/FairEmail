# Create Top-Level Folder

## Parent Feature
#2.4 Folder Subscription Management

## User Story
As a folder organizer, I want to create a new top-level folder for an IMAP account via a prominent action in the folder list view, so that I can organize my mail without resorting to webmail.

## Blocked by
_(none — folder creation is independent of subscription management)_

## Acceptance Criteria
- A prominent action (e.g., floating button) in the folder list allows creating a new top-level folder.
- The action is only available for IMAP accounts (not POP3 or local-only).
- The creation form accepts at minimum: folder name (required), display name (optional), color (optional), synchronize on/off, poll on/off, download messages on/off, auto-classify on/off, notification on/off, unified-inbox membership on/off, navigation-pane visibility on/off, sync-days, keep-days, and auto-delete on/off.
- The folder name is validated: empty names are rejected with a clear error.
- The folder name is validated: duplicate names (same full name in the same account) are rejected with a clear error.
- The folder is marked "to be created" locally and the actual server operation runs during the next sync cycle.
- Upon successful server-side creation, the folder is automatically subscribed on the server.
- Upon successful server-side creation, the "to be created" flag is cleared and a folder sync is triggered so the folder appears with its server-assigned properties.
- The initial sync, notification, and unified-inbox settings are determined by the values chosen in the creation form — not influenced by subscription state.

## Mapping to Epic
- Goals: G2
- User Stories: US-12, US-14, US-15, US-16
- Functional Requirements: FR-14, FR-16, FR-17, FR-19, FR-20, FR-21, FR-38
- Acceptance Criteria: AC-8, AC-10
- Non-Functional Requirements: NFR-1, NFR-3, NFR-5

## HITL / AFK
**HITL** — the creation form layout and the prominent action placement need UX review before finalizing.

## Estimation
Medium — involves a new form/dialog, local queuing, server-side execution during sync, auto-subscribe, and validation logic.

## Notes
- FR-37 specifies that subscription state is independent of other folder properties. Ensure the creation form does not couple subscription to sync or notification settings.
