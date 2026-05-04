# Insecure Connection Protection

## Parent Feature
#1.6 Authentication Methods

## User Story
As a user, I want the application to refuse to send my password using PLAIN or LOGIN over an unencrypted connection by default, so that my credentials are never exposed on the network — while still allowing an explicit per-account override for trusted local networks.

## Acceptance Criteria
- When a connection uses no encryption (neither TLS nor STARTTLS), the application refuses to authenticate using PLAIN or LOGIN by default.
- The refusal produces a clear connection error or warning — the password is not sent in the clear.
- An explicit per-account "allow insecure connections" flag overrides this protection.
- Enabling the flag for an unencrypted server permits the connection and authentication to proceed.
- The flag defaults to off (secure by default).

## Blocked by
1-password-mechanism-negotiation

## HITL / AFK
AFK

## Notes
- This story covers FR-30 and FR-31. The TLS layer itself (cipher suites, certificate validation) is out of scope per NG1 (covered by epic 7.9).
