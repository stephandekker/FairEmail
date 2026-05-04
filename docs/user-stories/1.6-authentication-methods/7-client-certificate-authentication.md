# Client Certificate Authentication (EXTERNAL)

## Parent Feature
#1.6 Authentication Methods

## User Story
As an enterprise user whose organization uses client-certificate authentication, I want to select an installed certificate for my incoming-mail and/or outgoing-mail connections independently, and authenticate without providing a password via mutual TLS.

## Acceptance Criteria
- The user can select an installed client certificate for the incoming-mail server endpoint.
- The user can select a different installed client certificate for the outgoing-mail server endpoint (independently).
- When a client certificate is selected and no password is provided, the application presents the certificate during the TLS handshake and relies on EXTERNAL authentication — no password prompt appears.
- The selected certificate is identified by a stable alias; the application re-resolves the alias to the current certificate and private key at each connection attempt.
- When a certificate is configured and no password is provided, no password-based mechanism is attempted (FR-7).

## Blocked by
1-password-mechanism-negotiation

## HITL / AFK
HITL — certificate selection UX and interaction with system certificate stores may need design review.

## Notes
- OQ-6 flags that if the system certificate store is empty, the UX may be confusing. Consider providing guidance, but the epic does not mandate it.
- Design Note N-6: per-endpoint certificate selection supports environments where IMAP and SMTP servers have different CAs.
