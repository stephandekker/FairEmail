# Jump Page Redirect Fallback

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user in an environment where native redirect mechanisms (local HTTP server, custom URI scheme) are unavailable or untrusted, I want the application to support a publisher-hosted jump page as a fallback redirect mechanism, so that the OAuth flow can still complete.

## Blocked by
- `2-core-oauth-authorization-flow`

## Acceptance Criteria
- The application supports a jump page (an intermediary web redirect hosted by the publisher) as a fallback when native deep-link redirection is unavailable.
- When the jump page is used, only the authorization code transits the publisher's server — no tokens are sent to or through the jump page.
- The OAuth flow completes successfully via the jump page path.
- The same CSRF protection (state parameter validation) and session timeout apply regardless of whether the native or jump page redirect is used.

## Mapping to Epic
- FR-11 (jump page as fallback redirect mechanism)
- NFR-8 (only authorization code transits publisher server, no tokens)
- Open Question OQ-1 (jump page trust model)

## HITL / AFK
AFK — the fallback is transparent to the user (though the user may notice a brief redirect through the publisher's domain).

## Notes
- Open Question OQ-1 from the epic asks about the trust model for the jump page: Should the user be informed when the jump page is in use? Should self-hosters be able to run their own? These questions are unresolved in the epic and should be addressed during design.
- On Linux desktop, native redirect via a local HTTP server (e.g. `http://localhost:<port>/callback`) is likely reliable. The jump page may be less necessary than on mobile, but should still be available as a fallback.
- The privacy implications of routing through a publisher-hosted page should be documented for the privacy-conscious user persona (P3).
