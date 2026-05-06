# APOP Support (POP3 Only)

## Parent Feature
#1.6 Authentication Methods

## User Story
As a legacy-server user whose POP3 server requires APOP, I want to enable APOP support and have the application use it for that connection, so that I can connect to my server without transmitting my password in the clear.

## Acceptance Criteria
- APOP is disabled by default.
- APOP can be enabled via an advanced setting.
- When APOP is enabled and the POP3 server's greeting contains a valid timestamp, the application authenticates using APOP instead of other password-based mechanisms.
- With APOP disabled (the default), the same POP3 server is authenticated using a standard password-based mechanism.
- APOP is available only for POP3 connections; it has no meaning for IMAP or SMTP.

## Blocked by
1-password-mechanism-negotiation

## HITL / AFK
AFK

## Notes
- OQ-5: APOP relies on MD5. The epic considers "disabled by default" sufficient posture; no additional warning is mandated.
- Design Note N-3: enabling APOP by default would cause unnecessary negotiation attempts on servers that handle it poorly.
