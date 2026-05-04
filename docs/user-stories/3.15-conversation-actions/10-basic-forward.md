# Basic Forward

## Parent Feature
#3.15 Conversation Actions

## User Story
As a delegator, when I forward a message, I want the To field empty, the subject prefixed with "Fwd:", the original body quoted, and all attachments included, so that recipients see the forwarded content with full context.

## Blocked by
`1-action-menu-infrastructure`, `3-identity-auto-selection`

## Acceptance Criteria
- Selecting "Forward" from the action menu opens a compose window with an empty "To" field (FR-20, AC-6).
- The subject is prefixed with "Fwd:" (or a configured alternative), with deduplication rules matching "Re:" (FR-21).
- The original message body is quoted using the same quoting rules as reply (FR-22).
- All attachments from the original message — both inline images and regular file attachments — are copied to the new draft (FR-23, AC-6).
- The "From" identity defaults to the account that holds the message being forwarded (FR-51).
- Forward works offline for downloaded messages (NFR-4, AC-22).

## Mapping to Epic
- US-11, US-12
- FR-20, FR-21, FR-22, FR-23, FR-51
- AC-6, AC-22

## HITL / AFK
AFK — straightforward compose-window setup.

## Notes
- Forward thread linkage configuration (whether the forward stays in the original thread or starts a new one) is a separate story (12-forward-thread-linkage).
- The "forward to" shortcut for recent recipients is a separate story (11-forward-to-shortcut).
