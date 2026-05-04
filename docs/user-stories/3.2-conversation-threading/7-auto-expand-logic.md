# Conversation View — Auto-Expand Logic

## Parent Feature
#3.2 Conversation Threading

## User Story
As any user, when I open a conversation, I want one message to be automatically expanded according to sensible rules so that I immediately see the most relevant message without manual interaction.

## Blocked by
6-conversation-view

## Acceptance Criteria
- [ ] When there is exactly one message in the conversation, it is expanded (FR-39 rule 1).
- [ ] When there is exactly one unread message, it is expanded (FR-39 rule 2, AC-13).
- [ ] When there are no unread messages and exactly one starred message, the starred message is expanded (FR-39 rule 3).
- [ ] Otherwise (multiple unread, or no unread and no/multiple starred), the most recent message is expanded (FR-39 rule 4, AC-13).
- [ ] Exactly one message is auto-expanded on open — never zero, never more than one.

## HITL / AFK
AFK — deterministic rule evaluation.

## Notes
_(none)_
