# Credential Isolation in Diagnostic Logs

## Parent Feature
#1.6 Authentication Methods

## User Story
As a user, I want assurance that no authentication credential (password, token, or certificate private key) is ever written to log output, crash reports, or diagnostic exports, while still being able to see which mechanism was used for each connection.

## Acceptance Criteria
- The diagnostic log records which authentication mechanism was used for each connection.
- No password, token value, or certificate private key appears in log output.
- No credential appears in crash reports or diagnostic exports.
- Mechanism negotiation details (which mechanisms were advertised, which was selected) are loggable without exposing secrets.

## Blocked by
1-password-mechanism-negotiation

## HITL / AFK
AFK

## Notes
- NFR-4 is the governing requirement. This is a cross-cutting concern that should be verified against all auth paths (password, OAuth, certificate).
