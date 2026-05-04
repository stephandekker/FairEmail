# Anonymous Identifier Generation & Storage

## Parent Feature
#10.1 No Telemetry by Default

## User Story
As a user who has opted in to error reporting, I want error reports to be associated only with a random anonymous identifier that cannot be linked to my email accounts, my device identity, or my real-world identity, so that reports cannot be used to track or profile me.

## Acceptance Criteria
- [ ] When error reporting is first enabled (or on first launch if no identifier exists yet), a random anonymous identifier is generated and stored locally.
- [ ] The identifier is not derived from any hardware identifier, MAC address, hostname, account credential, email address, or other personally-identifiable information.
- [ ] The identifier persists across application restarts (stored locally).
- [ ] Reinstalling the application produces a new identifier (no cross-installation correlation).
- [ ] It is computationally infeasible to link the identifier to a real user identity, device, or account using only the data available to the error-reporting service.

## Complexity Estimate
Small

## Blocked by
3-error-reporting-preference-toggle

## Notes
- Epic open question OQ-4 asks whether the identifier should rotate periodically or remain stable. This story assumes a single stable identifier per installation (matching the source application's behavior), as it provides better debugging correlation. If periodic rotation is desired, add a follow-up story.
- This story covers FR-14, US-9 (partial), NFR-5, AC-7 (partial — generation side).
