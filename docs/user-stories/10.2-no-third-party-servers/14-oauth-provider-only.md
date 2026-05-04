# OAuth Authentication — Provider-Only Traffic

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As a user who authenticates via OAuth for a major mail provider, I want the OAuth flow to contact only my mail provider's authorization and token servers, with no additional third-party servers involved, so that my authentication does not disclose information to parties beyond my chosen provider.

## Blocked by
- `1-default-network-posture` (this is a specific guarantee within the overall no-third-party posture)

## Acceptance Criteria
- OAuth authentication flows contact only the user's mail provider's authorization and token endpoints.
- No additional third-party servers are involved in the authentication process (no intermediary, no analytics endpoint, no redirect through developer infrastructure).
- OAuth token refresh occurs directly between the application and the provider's token endpoint, with no intermediary.

## Mapping to Epic
- US-17
- FR-36, FR-37
- AC-10

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- N-7 in the epic clarifies that OAuth endpoints are treated as part of the user's mail provider infrastructure, not as third-party servers. This is a pragmatic distinction: the user has already entrusted their email to this provider.
- OQ-5 asks whether OAuth client IDs (which identify the application to the provider) should be disclosed in privacy documentation. This is a documentation concern addressed in story 17.
