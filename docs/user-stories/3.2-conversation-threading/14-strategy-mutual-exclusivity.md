# Strategy Mutual Exclusivity

## Parent Feature
#3.2 Conversation Threading

## User Story
As any user, I want subject-based threading to be automatically disabled when I enable Gmail thread index (and vice versa), so that conflicting strategies do not produce confusing results.

## Blocked by
12-gmail-thread-index, 13-subject-based-fallback

## Acceptance Criteria
- [ ] Enabling Gmail thread index automatically disables subject-based threading (FR-4, AC-8).
- [ ] Enabling subject-based threading automatically disables Gmail thread index (FR-4, AC-8).
- [ ] The mutual exclusivity is enforced in the user interface at toggle time.

## HITL / AFK
AFK — a simple UI constraint between two toggles.

## Notes
_(none)_
