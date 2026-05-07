# User Story: Tenant-Specific OAuth and Graph API Profile

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **user of a corporate Microsoft 365 tenant**, I want the application to prompt me for my tenant identifier and adjust OAuth endpoints accordingly, and as a **user of a provider that supports Graph API for mail sending**, I want that capability to be bundled in the provider profile, so that I can authenticate against my organisation's tenant and use REST-based mail sending where supported.

This slice extends OAuth support with:
- Tenant placeholders in OAuth endpoint URLs. When present, the application prompts the user for a tenant identifier and substitutes it before initiating the OAuth flow (FR-22).
- A separate Graph API profile (same attribute set as OAuth) for providers that support REST-based mail sending alongside IMAP-based receiving (FR-23).
- Independent enable/disable/debug-only control for Graph profiles (FR-24).

## Acceptance Criteria
- [ ] OAuth endpoint URLs may contain tenant placeholders (e.g. `{tenant}`).
- [ ] When tenant placeholders are present, the setup flow prompts the user for a tenant identifier before initiating the OAuth flow (AC-8).
- [ ] The tenant identifier is substituted into all placeholder positions in the endpoint URLs.
- [ ] A provider entry may include a Graph API profile with the same attributes as an OAuth profile.
- [ ] Graph profiles can be independently enabled, disabled, or restricted to debug mode.
- [ ] When a Graph profile is enabled, the application uses it for REST-based mail sending alongside IMAP for receiving.

## Blocked by
`7-oauth-profile-support`

## HITL / AFK
**HITL** — The tenant prompt UX may need design review to ensure clarity for non-technical users. Implementation can proceed, but the prompt wording and flow should be reviewed.

## Notes
- The existing Android app supports Microsoft tenant-based OAuth with a `tenant` attribute in the OAuth XML element. The desktop app should support the same semantic.
- The Graph API profile is primarily used for Microsoft 365 providers that support `Mail.Send` via Graph alongside IMAP for reading. This is a niche but important capability for enterprise users.
