# Multi-Tenant Support (Office 365)

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user with an organizational Office 365 account, I want to specify my organization's tenant identifier during setup, so that the OAuth flow authenticates against the correct directory and I can sign in with my organizational credentials.

## Blocked by
- `2-core-oauth-authorization-flow`
- `3-setup-wizard-oauth-integration`

## Acceptance Criteria
- For providers that support or require a tenant identifier (e.g. Microsoft), the setup flow presents a field for the user to enter their tenant.
- The provider configuration uses a placeholder (e.g. `{tenant}`) in endpoint URLs that is substituted with the user-supplied value at authorization time.
- If no tenant is supplied, a sensible default is used (e.g. `common` for Microsoft, allowing both personal and organizational accounts).
- Authentication succeeds for a user in the specified organization's directory.
- The tenant value is stored with the account so that re-authorization uses the same tenant.

## Mapping to Epic
- FR-10 (tenant field, substituted into endpoint URLs)
- US-4 (specify organizational tenant)
- AC-5 (organizational tenant restricts OAuth to that directory, auth succeeds)

## HITL / AFK
HITL — the user enters a tenant identifier (or accepts the default).

## Notes
- The existing Android codebase detects tenant support via `{tenant}` placeholders in the provider's endpoint URLs and exposes a UI field when detected. The desktop implementation should follow the same pattern.
- The epic does not specify whether a tenant directory lookup or validation should happen before initiating the OAuth flow. Implementers may choose to validate after the flow completes.
