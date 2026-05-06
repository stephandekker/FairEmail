# Clear Client Certificate Selection

## Parent Feature
#1.6 Authentication Methods

## User Story
As a user who has selected a client certificate, I want to be able to clear the selection and revert to password-based or OAuth-based authentication, so that I am not locked into certificate auth.

## Acceptance Criteria
- The user can clear the certificate selection for incoming and/or outgoing server independently.
- After clearing and providing a password, the account reverts to password-based authentication on the next connection.
- After clearing and completing OAuth, the account reverts to OAuth-based authentication.
- No stale certificate reference remains after clearing.

## Blocked by
7-client-certificate-authentication

## HITL / AFK
AFK

## Notes
(none)
