# Provider-Specific Authentication Handling

## Parent Feature
#1.6 Authentication Methods

## User Story
As a user connecting to a provider with known authentication quirks, I want the application to accommodate those quirks automatically based on the server hostname, so that I do not need manual workarounds.

## Acceptance Criteria
- Provider-specific authentication adaptations are keyed to the server hostname.
- Adaptations activate only for the relevant provider and do not affect other connections.
- Known quirks accommodated include (but are not limited to): provider-specific greeting identifiers, multi-line authentication sequences for certain POP3 OAuth flows, and provider-mandated request parameters.
- No user intervention is required for provider-specific handling to activate.

## Blocked by
1-password-mechanism-negotiation, 3-oauth-browser-authorization-flow

## HITL / AFK
HITL — identifying the full set of provider quirks requires research and possibly testing against live servers.

## Notes
- OQ-3 flags uncertainty about SMTP + XOAUTH2: some providers accept XOAUTH2 for SMTP while others use the OAuth token as a password via PLAIN. This story should document the per-provider strategy.
- Design Note N-8: hostname-keyed workarounds ensure narrow activation.
