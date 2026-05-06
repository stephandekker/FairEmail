# Authentication Error Reporting

## Parent Feature
#1.6 Authentication Methods

## User Story
As a user, when authentication fails I want a clear error message distinguishing the cause (invalid credentials, unsupported mechanism, expired token, certificate rejection, server error, or disabled mechanism), so that I can take corrective action without guessing.

## Acceptance Criteria
- Authentication failure messages distinguish between: invalid credentials, unsupported mechanism, expired or revoked token, certificate rejection, and server-side errors.
- When authentication fails because all enabled mechanisms have been exhausted, the error mentions the possibility that a required mechanism has been disabled in advanced settings.
- When a user disables a mechanism that their server requires, the error helps them understand the cause.
- NFR-7: when a provider has deprecated a mechanism (e.g. Google disabling password access), messaging guides toward the supported alternative (OAuth or app-specific passwords).

## Blocked by
1-password-mechanism-negotiation, 11-global-mechanism-toggles

## HITL / AFK
HITL — error message copy and UX presentation may benefit from design review.

## Notes
- This story aggregates FR-35, FR-36, and US-16 into a single slice because they all concern the same error-reporting surface.
