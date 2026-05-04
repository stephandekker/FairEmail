# Switch Between OAuth and Password Authentication

## Parent Feature
#1.6 Authentication Methods

## User Story
As a user with an OAuth-authenticated account, I want the option to switch to password-based authentication (e.g. using an app-specific password), and vice versa, without deleting and re-adding the account, so that I am not locked into either method.

## Acceptance Criteria
- An OAuth account can be switched to password-based authentication by providing a password.
- A password-based account can be switched to OAuth by completing the browser authorization flow.
- Switching preserves the account's folders, messages, and all other settings — only the authentication method changes.
- After switching from OAuth to password, the application uses password-based mechanism negotiation (story 1).
- After switching from password to OAuth, the application uses XOAUTH2 exclusively.

## Blocked by
3-oauth-browser-authorization-flow

## HITL / AFK
AFK

## Notes
- Design Note N-5 clarifies that auth type is a top-level attribute, not a preference flag. The switch changes this attribute cleanly.
