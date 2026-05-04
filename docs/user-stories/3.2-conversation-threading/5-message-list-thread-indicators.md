# Message-List Thread Indicators

## Parent Feature
#3.2 Conversation Threading

## User Story
As any user, in the message list I want each conversation entry to display the total message count — and optionally the unread message count — so that I can see at a glance how large and how current each thread is.

## Blocked by
3-global-threading-toggle

## Acceptance Criteria
- [ ] Each conversation entry in the message list displays the total number of messages in the thread (FR-41, AC-18).
- [ ] An optional display setting controls whether the unread message count is also shown within each conversation entry (FR-42).
- [ ] The unread-count setting is disabled by default (FR-42).
- [ ] When the unread-count setting is enabled, the unread count is correctly displayed and updated as messages are read (AC-18).

## HITL / AFK
AFK — display logic driven by aggregate counts on an already-computed thread identifier.

## Notes
- This story covers the message-list indicators only, not the conversation-detail view (that is story 6).
