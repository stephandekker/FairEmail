# Self-Reply Detection

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, when I reply to a message I myself sent, I want the To field populated with the original recipients rather than my own address, so that the reply goes to the people I was corresponding with.

## Blocked by
`2-basic-reply`, `3-identity-auto-selection`

## Acceptance Criteria
- When replying to a message whose From address matches one of the user's configured identity addresses, the "To" field is set to the original message's recipients instead of the From address (FR-9, AC-3).
- The "CC" field is set to any original CC recipients (FR-9).
- The user's own addresses are excluded from both To and CC fields.
- Self-reply detection works for both reply and reply-all actions.
- Threading headers are set correctly (same as any other reply).

## Mapping to Epic
- US-5
- FR-9
- AC-3
- Design Note N-5

## HITL / AFK
AFK — logic is clearly specified.

## Notes
- This is an important edge case that prevents the common frustration of accidentally replying to oneself. The detection must check against all configured identity addresses, not just the primary one.
