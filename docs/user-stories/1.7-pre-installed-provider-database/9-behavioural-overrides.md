# User Story: Behavioural Overrides (Quirks)

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **user of a provider with known connection quirks** (e.g. unreliable IDLE support, partial-fetch incompatibility, TLS version restrictions), I want the application to apply the correct workarounds automatically based on the provider profile, so that I do not experience mysterious connection failures or need to manually tune settings.

This slice adds behavioural override support to the provider data model and ensures overrides are applied automatically at account creation:
- Polling/keepalive interval override (FR-25).
- NOOP-based keepalive instead of IDLE (FR-26).
- Disable partial/incremental message fetching (FR-27).
- Restrict maximum TLS version (FR-28).
- Disable IP-address-based connections (FR-29).
- Flag: provider requires manual IMAP/SMTP enablement (FR-30).
- Flag: provider requires app-specific password with 2FA (FR-31).
- All overrides applied automatically without user awareness (FR-32).

## Acceptance Criteria
- [ ] A provider entry can specify a keepalive interval (in minutes) that overrides the application default.
- [ ] A provider entry can specify NOOP-based keepalive instead of IDLE.
- [ ] A provider entry can disable partial/incremental message fetching.
- [ ] A provider entry can restrict the maximum TLS version.
- [ ] A provider entry can disable IP-address-based connections.
- [ ] A provider entry can flag that manual IMAP/SMTP enablement is required.
- [ ] A provider entry can flag that an app-specific password is required with 2FA.
- [ ] When a provider with any of these overrides is used for account creation, the created account's connection settings reflect those overrides without user intervention (AC-9).
- [ ] No user-facing UI is needed to activate overrides — they are applied silently from the provider profile.

## Blocked by
`2-server-settings-prefill`

## HITL / AFK
**AFK** — Overrides are data-driven flags applied at account creation time. Fully testable with fixture provider entries.

## Notes
- Design Note N-3 in the epic emphasises that quirks belong in the catalogue as data, not as conditional logic in connection code. The implementation should apply overrides by reading provider fields and setting account properties, not by checking provider identity in connection-handling code.
- The IMAP-enablement and app-password flags are *data* in this story. The *user-facing guidance* (displaying messages about these requirements) is covered in story 10.
