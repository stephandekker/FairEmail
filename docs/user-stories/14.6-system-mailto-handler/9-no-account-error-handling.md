## Parent Feature

#14.6 System mailto: Handler

## What to build

When the user clicks a `mailto:` link but no accounts are configured (or no account has a composable identity), the application displays a clear, non-technical error message explaining that an account must be set up before email can be sent. The error state should guide the user toward account setup (e.g. a button or link to the account configuration flow). The application must not crash, hang, or show a blank/unusable window.

This applies to both warm-start and cold-start scenarios.

Covers epic sections: FR-18, FR-19; AC-7.

## Acceptance criteria

- [ ] When no accounts are configured and a `mailto:` URI is received, a clear error message is displayed (not a crash, blank window, or silent failure)
- [ ] The error message is non-technical and understandable by a new user
- [ ] The error state provides a path to account setup (e.g. a button to open account configuration)
- [ ] The error handling works in both warm-start and cold-start scenarios
- [ ] After the user configures an account, subsequent `mailto:` links work normally

## Blocked by

- Blocked by `3-basic-warm-start-compose`

## User stories addressed

- US-11 (clear feedback when no account is configured, guidance toward setup)

## Type

AFK
