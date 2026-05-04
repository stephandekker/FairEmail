# Forward-To Shortcut

## Parent Feature
#3.15 Conversation Actions

## User Story
As a delegator, I want a "forward to" shortcut that shows my recently-used forwarding recipients, so that I can forward messages to frequent contacts in one step without typing their address.

## Blocked by
`10-basic-forward`

## Acceptance Criteria
- A "Forward to" option is available in the action menu or as a sub-menu of Forward (FR-25).
- The shortcut presents a list of recently-used forwarding recipients (FR-25, AC-7).
- Selecting a recipient from the list immediately opens a forward compose with that recipient in the "To" field (AC-7).
- The recent-recipients list is maintained automatically based on previous forward actions.
- The list is ordered by recency or frequency of use.

## Mapping to Epic
- US-13
- FR-25
- AC-7
- Design Note N-9

## HITL / AFK
HITL — the presentation of the recent-recipients list (inline menu, sub-menu, or popup) and how many entries to show are UX decisions that benefit from design review.

## Notes
- N-9 explains the rationale: some users forward many messages per day to a small set of destinations (e.g., a ticketing system). This is a workflow accelerator, not a core action.
- The mechanism for tracking recent forward recipients (how many to remember, when to prune) is an implementation detail left to the designer.
