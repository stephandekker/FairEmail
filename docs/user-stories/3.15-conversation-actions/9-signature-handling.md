# Signature Handling for Conversation Actions

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, I want signatures applied automatically to replies and forwards (with per-action control), placed where I prefer, and suppressed on follow-up replies in the same thread, so that my messages have consistent branding without manual effort or visual noise.

## Blocked by
`2-basic-reply`

## Acceptance Criteria
- Signatures are appended by default to reply, reply-all, reply-to-list, and forward actions (FR-53, AC-18).
- Each action type has an independent toggle for signature inclusion (e.g., "include signature on reply" vs. "include signature on forward") (FR-53).
- A "signature only on first reply" option suppresses the signature on second and subsequent replies within the same thread (FR-54, AC-19).
- Signature placement is configurable: before quoted text, after quoted text, or at the very end of the message (FR-55).
- Resend and edit-as-new never append a signature automatically (FR-56, AC-18).

## Mapping to Epic
- FR-53, FR-54, FR-55, FR-56
- AC-18, AC-19
- Design Notes N-6, N-7

## HITL / AFK
AFK — behavior is well-specified.

## Notes
- "Signature only on first reply" (N-6) requires detecting whether the user has previously replied in the same thread. This likely requires checking whether the thread already contains a sent message from the user.
- The signature content itself is defined per-identity in the identity/account configuration (outside this epic). This story only governs when and where the signature is inserted.
