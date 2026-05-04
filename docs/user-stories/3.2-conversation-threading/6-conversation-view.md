# Conversation View — Chronological Message List

## Parent Feature
#3.2 Conversation Threading

## User Story
As any user, when I open a conversation, I want to see all messages in the thread displayed in chronological order, with each message individually expandable and collapsible, so that I can navigate long threads efficiently.

## Blocked by
3-global-threading-toggle

## Acceptance Criteria
- [ ] Opening a conversation displays all messages in the thread (FR-35).
- [ ] Messages are ordered primarily by received date, with a secondary sort by folder role: Inbox > Outbox > Drafts > Sent > Trash > Spam > other system folders > user folders > Archive (FR-36).
- [ ] Each message is individually expandable and collapsible (FR-35).
- [ ] Messages are automatically marked as read upon expansion, unless the user has disabled auto-mark-read in account settings (FR-40).
- [ ] All threading-related display elements (expand/collapse controls, message entries) are reachable via keyboard and carry appropriate screen-reader labels (NFR-8).

## HITL / AFK
AFK — well-defined ordering and interaction rules.

## Notes
- This story covers the basic conversation view layout and expand/collapse mechanics. Auto-expand logic (which message to expand on open) is a separate story (7). Trashed messages/drafts visibility is story 9. Sent/received indentation is story 13.
