# Reply-All

## Parent Feature
#3.15 Conversation Actions

## User Story
As a conversationalist, when I reply-all to a message, I want the To field set to the original sender and the CC field populated with all other original recipients (minus my own addresses), so that everyone in the conversation receives my response.

## Blocked by
`2-basic-reply`, `3-identity-auto-selection`

## Acceptance Criteria
- Selecting "Reply all" from the action menu opens a compose window (FR-8).
- The "To" field is set to the original sender's reply-to or from address (FR-8).
- The "CC" field is populated with all original To and CC recipients, excluding any of the user's own configured identity addresses (FR-8, AC-2).
- Reply-all is hidden or disabled when there are no additional recipients beyond the sender (US-8).
- Subject, quoting, and threading headers behave identically to basic reply (FR-10, FR-11, FR-12).
- Reply-all works offline for downloaded messages (NFR-4, AC-22).

## Mapping to Epic
- US-7, US-8
- FR-8, FR-10, FR-11, FR-12
- AC-2, AC-22

## HITL / AFK
AFK — recipient logic is well-specified.

## Notes
- OQ-5 in the epic asks whether reply-all should warn on very large recipient lists (50+). This story implements the basic behavior; a warning threshold could be added as a follow-up if the open question is resolved.
