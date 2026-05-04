# Thread-Level Move Operations

## Parent Feature
#3.2 Conversation Threading

## User Story
As any user, when I move, archive, or trash a conversation, I want the option to apply that action to all messages in the thread — and separately control whether sent messages are included — so that entire conversations are handled as a unit without disrupting my Sent folder.

## Blocked by
3-global-threading-toggle

## Acceptance Criteria
- [ ] A behaviour setting ("move all messages when moving conversations") causes move/archive/trash actions on a conversation to apply to all messages in the thread (FR-44, AC-16).
- [ ] A separate behaviour setting ("also move sent messages when moving a conversation") includes sent messages in thread-level move operations (FR-45).
- [ ] The "also move sent messages" setting is independently toggleable but implicitly enabled when "move all messages" is enabled (FR-45).
- [ ] When "also move sent messages" is disabled, sent messages are excluded from thread-level move operations even when "move all messages" is enabled (AC-17).
- [ ] Both settings are disabled by default (FR-46).

## HITL / AFK
AFK — well-defined boolean settings governing move scope.

## Notes
- The implicit-enable behaviour of "also move sent messages" when "move all messages" is turned on needs careful UX — the epic states it is "implicitly enabled" but independently toggleable, meaning the user can turn it back off after enabling "move all messages."
