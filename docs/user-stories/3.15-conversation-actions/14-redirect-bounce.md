# Redirect / Bounce (DSN)

## Parent Feature
#3.15 Conversation Actions

## User Story
As an administrator, I want to send a hard bounce (DSN) to the envelope sender, so that the sending system receives a standards-compliant delivery failure notification.

## Blocked by
`1-action-menu-infrastructure`

## Acceptance Criteria
- A "Bounce" action is available in the action menu, gated behind an explicit opt-in setting (e.g., "experimental features" or "advanced" toggle) (FR-32, AC-10).
- The action is visible only when the message has a Return-Path header (FR-5, AC-10).
- The action is suppressed when the Return-Path matches any of the user's own identity addresses (FR-33, AC-10).
- Before sending, a warning about potential impact on email provider sending reputation is displayed (FR-34, US-20).
- The bounce generates a valid Delivery Status Notification per RFC 3464, directed to the bounce address (Return-Path header value) (FR-31, AC-11).
- The generated DSN conforms to RFC 3464 (NFR-2).

## Mapping to Epic
- US-18, US-19, US-20, US-21
- FR-31, FR-32, FR-33, FR-34
- NFR-2, NFR-7
- AC-10, AC-11
- Design Note N-3

## HITL / AFK
HITL — DSN generation is complex and standards-sensitive. The generated message format should be reviewed for RFC 3464 compliance, and the warning text reviewed for clarity.

## Notes
- OQ-2 in the epic asks whether "redirect" (resend to a new recipient with Resent-* headers) should be a separate action from "bounce" (DSN to envelope sender). Currently, only DSN-style bounce exists. If redirect is added later, it would be a new story.
- OQ-6 raises the concern of bounce loops (e.g., Outlook.com responding to bounces with its own bounces). Beyond self-address suppression, detecting and preventing bounce storms (e.g., refusing to bounce a message that is itself a DSN) should be considered during implementation.
