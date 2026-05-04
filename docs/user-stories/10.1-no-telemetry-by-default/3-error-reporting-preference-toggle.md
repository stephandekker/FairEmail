# Error Reporting Preference Toggle

## Parent Feature
#10.1 No Telemetry by Default

## User Story
As a helpful contributor, I want a clearly-labeled setting in the application's preferences that allows me to enable or disable anonymous error reporting, so that I can make an informed, deliberate choice to help improve the application — and revoke that choice at any time.

## Acceptance Criteria
- [ ] A single, clearly-labeled boolean setting (on/off) for error reporting exists in the application's settings area.
- [ ] The default value of this setting is **off** (disabled).
- [ ] The setting persists across application restarts.
- [ ] Changing the setting takes effect immediately within the current session, without requiring a restart.
- [ ] The setting label and surrounding context make it clear what the user is consenting to.

## Complexity Estimate
Small

## Blocked by
1-audit-strip-telemetry-infrastructure

## Notes
- This story establishes the preference storage and UI toggle only. Actual report transmission is covered by story 5. Immediate cessation behavior on disable is covered by story 6.
- This story covers FR-4, FR-5, FR-6 (partial — the "takes effect immediately" contract), FR-7, US-4, US-12, NFR-2, AC-2.
