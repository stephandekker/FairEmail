# Browser Selection and Compatibility

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a privacy-conscious user, I want the OAuth flow to open in my preferred system browser (or a privacy-focused browser I have configured), and I want the application to warn me if my browser has known compatibility issues, so that my credentials are entered in a trusted context and the flow completes reliably.

## Blocked by
- `2-core-oauth-authorization-flow`

## Acceptance Criteria
- The application prefers the user's system default browser for the OAuth flow.
- The user can configure which browser is used for OAuth flows.
- The application prefers privacy-focused browsers when available (and the user hasn't overridden the choice).
- If the selected or default browser has known compatibility issues with OAuth redirect handling, the application warns the user.
- If no suitable browser is available, the application falls back to an embedded secure browser surface with a clear indication that the context has changed.

## Mapping to Epic
- FR-31 (prefer privacy-focused browsers, allow user configuration)
- FR-32 (warn on known compatibility issues)
- FR-33 (embedded fallback with clear indication)
- US-20 (system browser, not embedded web view)
- AC-13 (OAuth opens in system browser)

## HITL / AFK
HITL — user may need to configure their preferred browser; warnings require acknowledgment.

## Notes
- On Linux desktop, the system browser is typically determined by `xdg-open` or the `BROWSER` environment variable. The application may need to enumerate installed browsers to offer a selection.
- Open Question OQ-8 from the epic asks how a browser compatibility list should be maintained on desktop. This is less of an issue on Linux than on Android (fewer problematic browser variants), but the mechanism should exist for edge cases.
- The embedded fallback (FR-33) may not be feasible on Linux without bundling a web engine. This should be flagged during design — the implementation may need to treat "no suitable browser" as a hard error rather than falling back to embedded.
