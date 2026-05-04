# Trashed Messages and Drafts in Conversation View

## Parent Feature
#3.2 Conversation Threading

## User Story
As any user, I want the conversation view to show trashed messages and drafts within the thread context, so that I can find accidentally deleted messages and resume drafts where they belong.

## Blocked by
6-conversation-view

## Acceptance Criteria
- [ ] Trashed messages appear within their conversation in the conversation view (FR-37, AC-14).
- [ ] Draft messages appear within their conversation in the conversation view (FR-37, AC-14).
- [ ] Trashed messages and drafts are sorted according to the same ordering rules as other messages (received date, then folder role).

## HITL / AFK
AFK — straightforward inclusion of additional folder roles in the conversation query.

## Notes
- Per design note N-5, showing trashed messages is intentional: individual messages are rarely trashed deliberately, and showing them in context makes recovery easy. Drafts in context let the user see the messages they are responding to while composing.
