# Thread Time Range

## Parent Feature
#3.2 Conversation Threading

## User Story
As a mailing-list power user, I want to configure the time window within which the application searches for related messages when building threads, so that I can widen it for long-lived discussions or narrow it to avoid spurious grouping.

## Blocked by
1-rfc-header-thread-computation

## Acceptance Criteria
- [ ] All threading lookups (except subject-based, which has its own window) are constrained to a configurable time range centered on the received date of the message being threaded (FR-20).
- [ ] The default time range is 128 days (FR-21).
- [ ] The user can adjust the time range through an advanced/miscellaneous setting, expressed as a power-of-two number of days from 1 day to a very large upper bound (FR-22).
- [ ] Messages received outside the configured time range are not considered for thread matching, even if their identifiers match (FR-23, AC-10).
- [ ] Adjusting the time range affects only future threading operations; existing threads are not retroactively split or merged (AC-11).

## HITL / AFK
AFK — a numeric setting with clear constraints. The power-of-two scale is well-defined.

## Notes
- The interaction between the global time range and subject-based threading's fixed 48-hour window is an open question in the epic (OQ-3). This story implements them as separate windows per the epic's current specification.
