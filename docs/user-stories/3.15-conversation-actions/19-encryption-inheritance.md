# Encryption Inheritance on Reply

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, when I reply to an encrypted message, I want the reply to default to the same encryption mode if I have a matching key, so that the conversation remains secure without manual configuration.

## Blocked by
`2-basic-reply`

## Acceptance Criteria
- When replying to an encrypted message, the reply defaults to the same encryption mode (sign, encrypt, or both) if the user has a matching key/certificate (FR-57, AC-20).
- This behavior is controlled by a global preference that can be toggled on or off (FR-57).
- Forward, edit-as-new, and resend do not inherit encryption settings automatically (FR-58).

## Mapping to Epic
- FR-57, FR-58
- AC-20

## HITL / AFK
AFK — the behavior is a single preference toggle with clear logic.

## Notes
- The encryption/signing mechanisms themselves are defined in the encryption epic (NG4). This story only covers the *inheritance* of encryption mode when initiating a reply — not key management, encryption UI, or the encryption process itself.
- If the encryption epic has not yet been implemented, this story can be deferred or implemented as a no-op preference that takes effect once encryption support lands.
