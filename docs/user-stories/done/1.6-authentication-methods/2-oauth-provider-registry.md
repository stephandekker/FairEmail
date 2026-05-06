# OAuth Provider Registry

## Parent Feature
#1.6 Authentication Methods

## User Story
As a developer or administrator, I want a bundled registry of OAuth-supporting providers (including authorization endpoint, token endpoint, required scopes, redirect URI, and client identifier), so that adding OAuth support for a new provider requires only a configuration entry and no code changes.

## Acceptance Criteria
- A bundled registry exists containing OAuth configuration for at least Gmail, Outlook/Microsoft 365, Yahoo, and AOL.
- Each registry entry includes: authorization endpoint URL, token endpoint URL, required scopes, redirect URI, and client identifier.
- Adding a new provider to the registry makes it available in the OAuth sign-in flow without changes to authentication logic.
- Provider entries support provider-specific parameters (e.g. PKCE requirement, consent prompt type, account selection prompt).

## Blocked by
(none)

## HITL / AFK
HITL — the exact list of providers to include and their client IDs / registration details likely require human decision-making.

## Notes
- NFR-6 requires that provider extensibility be configuration-only.
- OQ-4 flags that the exact provider list is subject to change; a process for keeping it current should be established but is outside this story's scope.
- OQ-7 notes uncertainty about Microsoft Graph vs. standard OAuth paths — this may require two registry entries for Microsoft or a flag in the entry.
