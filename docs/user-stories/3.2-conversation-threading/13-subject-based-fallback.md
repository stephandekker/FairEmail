# Subject-Based Fallback Threading

## Parent Feature
#3.2 Conversation Threading

## User Story
As a user receiving mail from systems with poor threading headers, I want an optional mode that groups messages by sender and subject line within a recent time window, so that those messages are still grouped into conversations.

## Blocked by
1-rfc-header-thread-computation

## Acceptance Criteria
- [ ] When enabled, subject-based threading groups messages that share the same sender address and the same subject line, received within a fixed 48-hour time window (FR-13, AC-6).
- [ ] Subject-based threading is disabled by default (FR-14, US-10).
- [ ] Messages whose received dates are more than 48 hours apart are not grouped (AC-7, FR-16).
- [ ] The 48-hour time window is fixed and not user-configurable (FR-16).
- [ ] Subject-based threading does not apply to delivery status notifications or other automated report messages (FR-15).
- [ ] Subject-based threading is the lowest-priority strategy — it does not override RFC-header-based, common-reference, or Gmail thread index results (FR-1, FR-3).

## HITL / AFK
AFK — a well-constrained heuristic with clear matching rules.

## Notes
- Per design note N-4, the combination of sender + subject + short window is intentionally conservative to avoid spurious merges.
- OQ-6 flags a discrepancy between the FAQ (which says subject-based threading is not supported) and the actual capability. This should be clarified in user-facing documentation when this story is delivered.
