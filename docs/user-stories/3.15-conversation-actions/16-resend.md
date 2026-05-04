# Resend

## Parent Feature
#3.15 Conversation Actions

## User Story
As an administrator, I want to resend a message with its original headers preserved, so that the message appears to recipients as if it came from the original sender.

## Blocked by
`1-action-menu-infrastructure`

## Acceptance Criteria
- A "Resend" action is available in the action menu (FR-39).
- The action is shown in a disabled/dimmed state when message headers have not been downloaded; the user can trigger a header download from a related menu (FR-43, AC-14).
- The action creates a draft preserving the original From, To, CC, Subject, Date, and Message-ID headers (FR-39, AC-13).
- The user may edit the body and attachments before sending, but by default they are transmitted as-is (FR-40).
- No subject prefix, signature, or default CC/BCC is applied (FR-41, AC-18).
- A new conversation thread identifier is generated — the resent message does not link to the original conversation (FR-42).
- Messages addressed to "undisclosed-recipients:" are not resendable (FR-44).
- A warning informs the user that DKIM/SPF/DMARC validation is likely to fail (FR-45, AC-15).
- The feature is gated behind an opt-in setting (N-3).

## Mapping to Epic
- US-25, US-26, US-27, US-28, US-29
- FR-39, FR-40, FR-41, FR-42, FR-43, FR-44, FR-45
- NFR-7
- AC-13, AC-14, AC-15
- Design Note N-3

## HITL / AFK
HITL — resend header handling is nuanced and standards-sensitive. The generated headers should be reviewed for RFC 2822 §3.6.6 compliance, and the warning text reviewed for clarity.

## Notes
- OQ-3 asks whether the application should add Resent-From, Resent-To, Resent-Date, and Resent-Message-ID headers per RFC 2822 §3.6.6, or send the message exactly as-is. The implications for DKIM verification differ. This should be resolved before implementation.
- OQ-7 asks whether delivery/read receipts on resent messages should go to the original sender or the resending user. This ambiguity should be resolved before implementation.
