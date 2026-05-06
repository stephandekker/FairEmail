# OAuth Browser Authorization Flow

## Parent Feature
#1.6 Authentication Methods

## User Story
As a mainstream user adding a Gmail, Outlook, Yahoo, or other supported provider account, I want the application to open a browser-based sign-in page where I authorize access, then return to the application and authenticate using XOAUTH2 without typing my password into the email client.

## Acceptance Criteria
- The application opens the provider's consent page in the user's default browser.
- The authorization code is received via redirect URI.
- The authorization code is exchanged for an access token and refresh token.
- The access token is presented to the mail server using the XOAUTH2 SASL mechanism.
- Provider-specific requirements are supported: account selection prompts, specific consent prompts, additional parameters, and PKCE where required.
- The flow is operable via keyboard and compatible with screen readers (NFR-8).
- A link to the provider's privacy policy is displayed during the authorization flow where the provider has published one.

## Blocked by
2-oauth-provider-registry

## HITL / AFK
AFK — implementation is deterministic once the registry is populated.

## Notes
- US-8 (privacy policy link) is included here as it is a small addition to the same flow rather than a separate slice.
- The OAuth account is marked as a distinct auth type (Design Note N-5): it uses XOAUTH2 exclusively and does not fall back to password mechanisms.
