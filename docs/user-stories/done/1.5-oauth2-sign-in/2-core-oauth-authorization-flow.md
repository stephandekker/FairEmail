# Core OAuth Authorization Flow

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user adding a new account with a supported provider, I want the application to open my system browser to the provider's sign-in page using the OAuth2 authorization code flow with PKCE, exchange the resulting code for access and refresh tokens, and store those tokens durably, so that I can authenticate without giving the application my password.

## Blocked by
- `1-bundled-oauth-provider-database`

## Acceptance Criteria
- The application opens the user's system browser (not an embedded web view) to the provider's authorization endpoint.
- The authorization request uses the OAuth2 authorization code flow with PKCE (code verifier + code challenge), regardless of whether the provider mandates PKCE.
- Every authorization request includes a cryptographic `state` parameter; the redirect response is rejected if the state is missing or mismatched.
- The authorization session expires after at most 20 minutes; a stale redirect is rejected.
- On receiving a valid authorization code, the application exchanges it for an access token and a refresh token.
- If the token exchange does not return a refresh token, the authorization is treated as failed and the user is informed.
- Access token, refresh token, and expiry timestamp are stored durably (survive restarts) with at least the same protection as account passwords.
- Each account stores independent token state; tokens are never shared between accounts.
- Tokens are never displayed to the user in plain text under normal operation.

## Mapping to Epic
- FR-4 (authorization code flow with PKCE)
- FR-5 (system browser, not embedded web view)
- FR-6 (cryptographic state parameter, validated on redirect)
- FR-7 (20-minute session timeout)
- FR-8 (exchange code for tokens)
- FR-9 (mandatory refresh token)
- FR-12 (durable token storage)
- FR-13 (tokens encrypted at rest like passwords)
- FR-14 (per-account independent token state)
- NFR-4 (CSRF via state parameter)
- NFR-5 (session expiry)
- NFR-3 (no plain-text token display)
- US-1, US-2, US-20, US-21, US-22

## HITL / AFK
HITL — the user must interact with the browser to authenticate with their provider.

## Notes
- The existing Android app uses the AppAuth library for PKCE and state management. The desktop app will need an equivalent mechanism, likely involving a local HTTP server or custom URI scheme to receive the redirect.
- Design Note N-2: PKCE is used for all providers as defense-in-depth, not just those that require it.
- Design Note N-4: Mandatory refresh token — if not returned, the flow fails. This is deliberate.
- Open Question OQ-6 (token storage encryption scope) is relevant here — whether tokens go in a system keyring vs. encrypted database is a design decision. Flag this for the implementer.
