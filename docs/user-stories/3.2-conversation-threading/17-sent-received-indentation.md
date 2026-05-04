# Sent / Received Message Indentation

## Parent Feature
#3.2 Conversation Threading

## User Story
As any user, I want an optional setting to visually indent sent and received messages differently within the conversation view (e.g. left-align received, right-align sent), so that the flow of the conversation is easier to follow.

## Blocked by
6-conversation-view

## Acceptance Criteria
- [ ] An optional display setting controls whether sent and received messages are visually differentiated by indentation in the conversation view (FR-43).
- [ ] The setting is disabled by default (FR-43).
- [ ] The setting applies only when using the card-style message display (FR-43).
- [ ] The indentation is accessible — screen readers convey the sent/received distinction (NFR-8).

## HITL / AFK
AFK — a CSS/layout toggle scoped to one display mode.

## Notes
- The epic specifies this applies only to card-style message display. If the application does not yet have a card-style display mode, this story should be deferred until that mode exists, or the constraint should be revisited.
