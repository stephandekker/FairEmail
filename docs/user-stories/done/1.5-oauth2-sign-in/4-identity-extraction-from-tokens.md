# Identity Extraction from Tokens

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user completing OAuth sign-in, I want the application to automatically extract my email address and display name from the authorization response (ID token claims or a provider-specific user-info endpoint), so that my account and sending identity are pre-filled correctly without manual entry.

## Blocked by
- `2-core-oauth-authorization-flow`

## Acceptance Criteria
- When the authorization response includes identity information (email, display name) in an ID token, the application extracts and uses it to pre-fill account and identity configuration.
- For providers that do not include identity information in the token (e.g. Mail.ru), the application fetches it from a provider-specific user-info endpoint.
- The provider configuration flags whether a user-info fetch is required (analogous to the `askAccount` flag in the existing codebase).
- If neither the token nor the user-info endpoint yields a usable email address, the application prompts the user to enter it manually.
- Extracted identity information is used to set the default sending identity's email address and display name.

## Mapping to Epic
- FR-34 (extract identity from token claims)
- FR-35 (fetch from user-info endpoint if needed)
- FR-36 (prompt user if neither method works)
- AC-12 (Mail.ru: fetch email from user-info endpoint)
- Design Note N-9 (user-info fallback for Mail.ru)

## HITL / AFK
AFK — identity extraction is automatic after the OAuth flow completes. Manual entry is a fallback edge case.

## Notes
- The existing Android codebase handles this in the OAuth fragment, with a per-provider `askAccount` flag. The desktop implementation needs an equivalent mechanism in the provider database.
- This slice is narrow: it only covers extracting identity info and feeding it to account/identity creation. The account creation itself is in story 3.
