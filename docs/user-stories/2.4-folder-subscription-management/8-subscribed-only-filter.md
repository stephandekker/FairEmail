# Subscribed-Only Folder Filter

## Parent Feature
#2.4 Folder Subscription Management

## User Story
As a subscription-aware user, I want a "subscribed only" toggle in the folder list menu that hides unsubscribed IMAP folders, so that I can focus on the folders I care about.

## Blocked by
`1-global-subscription-setting` — the filter toggle is only visible when the global subscription management setting is enabled.

## Acceptance Criteria
- A "subscribed only" toggle appears in the folder list's option menu when subscription management is enabled.
- The toggle is not visible when subscription management is disabled.
- When active, the filter hides all unsubscribed IMAP folders **except**:
  - The Inbox (always shown regardless of subscription state).
  - Any folder that has at least one visible child (to preserve the navigation hierarchy).
- Folders belonging to non-IMAP accounts (POP3, local) are unaffected by the filter.
- The filter preference persists across application restarts.
- Changes to the filter take effect immediately on the displayed folder list.

## Mapping to Epic
- Goals: G5
- User Stories: US-7, US-8, US-9, US-10
- Functional Requirements: FR-10, FR-11, FR-12, FR-13
- Acceptance Criteria: AC-5, AC-6
- Non-Functional Requirements: NFR-5

## HITL / AFK
**AFK** — well-defined filtering logic with clear edge-case rules.

## Estimation
Medium — involves filter logic with multiple exception rules (Inbox, parent-of-visible-child, non-IMAP), persistence, and immediate UI update.

## Notes
- Design Note N-7: even if the Inbox is technically unsubscribed (unusual but possible), hiding it would render the primary mail view inaccessible. The Inbox exception is unconditional.
- Design Note N-8: if a parent folder is unsubscribed but has a subscribed (visible) child, the parent must also be shown to preserve visual hierarchy. Without this, subscribed child folders would appear to float at the wrong nesting level.
