# Reply-to-List

## Parent Feature
#3.15 Conversation Actions

## User Story
As a mailing-list participant, when a message has a List-Post header, I want a "Reply to list" action that sends my reply to the list address only, so that I address the list without accidentally emailing individuals.

## Blocked by
`2-basic-reply`, `3-identity-auto-selection`

## Acceptance Criteria
- "Reply to list" action is visible in the action menu only when the message has a List-Post header (FR-5, US-10, AC-5).
- The "To" field is set to the address extracted from the List-Post header (FR-17, AC-5).
- The "From" identity is set to the address that received the message (the To address from the original), so that the list recognizes the subscriber (FR-18).
- Threading headers are set as for a normal reply (FR-19).
- Subject, quoting, and other compose behaviors match basic reply.

## Mapping to Epic
- US-9, US-10
- FR-17, FR-18, FR-19
- AC-5

## HITL / AFK
AFK — behavior is well-defined by List-Post header parsing.

## Notes
- The List-Post header can contain a `mailto:` URI or be `NO` (indicating the list does not accept posts). The implementation should handle both cases — showing the action only when a valid mailto address can be extracted.
