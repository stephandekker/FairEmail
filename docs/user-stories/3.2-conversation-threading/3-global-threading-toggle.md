# Global Threading Toggle

## Parent Feature
#3.2 Conversation Threading

## User Story
As a flat-list user, I want to disable conversation threading globally through a display setting so that every message appears as an individual row in the message list; and when I re-enable it, I want the grouped view to restore without data loss.

## Blocked by
1-rfc-header-thread-computation

## Acceptance Criteria
- [ ] A global display setting exists to enable or disable conversation threading (FR-30).
- [ ] Threading is enabled by default (FR-33).
- [ ] When threading is enabled, the message list groups messages by thread identifier, showing one entry per conversation (FR-31).
- [ ] When threading is disabled, the message list shows every message as an individual entry, ignoring thread identifiers (FR-32).
- [ ] Disabling threading causes every message to appear individually across all views — Inbox, Unified Inbox, folder views, search results (AC-3, US-5).
- [ ] Re-enabling threading restores the grouped view without data loss (AC-4).
- [ ] The toggle takes effect immediately without requiring a restart (FR-34).
- [ ] The threading preference persists across application restarts (US-6).

## HITL / AFK
AFK — a settings toggle with well-defined on/off behaviour.

## Notes
- This story requires the message-list view to be aware of thread identifiers computed in story 1 and to switch between grouped and flat display modes.
