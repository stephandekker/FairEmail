# Read Receipt (MDN)

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, when a message requests a read receipt, I want a "Send read receipt" action that transmits an MDN to the requesting address, so that I can acknowledge receipt when appropriate.

## Blocked by
`1-action-menu-infrastructure`

## Acceptance Criteria
- A "Send read receipt" action is visible in the action menu only when the message has a Disposition-Notification-To header (FR-5, AC-21).
- The action generates a valid Message Disposition Notification per RFC 3798, directed to the address in the Disposition-Notification-To header (FR-46, AC-21).
- Read receipt sending is manual (user-initiated) by default (FR-47).
- A global preference allows the user to opt into automatic receipt sending (FR-48).
- The user can choose between standard and legacy MDN formats (FR-49).
- The generated MDN conforms to RFC 3798 (NFR-2).

## Mapping to Epic
- US-30, US-31
- FR-46, FR-47, FR-48, FR-49
- NFR-2
- AC-21

## HITL / AFK
AFK — MDN generation follows a well-defined RFC.

## Notes
- The distinction between "standard" and "legacy" MDN formats (FR-49) refers to different RFC revisions and field formats. The implementation should support both for interoperability with older mail systems.
- Automatic receipt sending (FR-48) should respect the user's privacy by defaulting to off and requiring explicit opt-in.
