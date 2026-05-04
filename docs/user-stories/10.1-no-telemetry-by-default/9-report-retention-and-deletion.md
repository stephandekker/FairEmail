# Report Retention & Deletion

## Parent Feature
#10.1 No Telemetry by Default

## User Story
As a user who has opted in, I want submitted error reports to be automatically deleted from the remote service after a bounded retention period (no longer than one month), so that my data does not accumulate indefinitely.

## Acceptance Criteria
- [ ] Error reports submitted to the remote service are automatically deleted after a retention period not exceeding one calendar month.
- [ ] The user can request manual deletion of their reports at any time by referencing their anonymous identifier.
- [ ] The deletion mechanism (automatic expiry configuration, or scheduled purge job) is verifiable.

## Complexity Estimate
Small–Medium

## Blocked by
5-error-report-capture-and-transmission

## Notes
- Epic open question OQ-5 asks what the deletion request mechanism should be (in-app button, email, web form). This story does not prescribe the mechanism — it requires that one exists and is documented. The simplest approach for v1 may be an email-based request (matching the source application's approach), with an in-app button as a future enhancement.
- The automatic expiry may be a configuration of the error-reporting service itself (e.g. Sentry's data retention settings) or a scheduled job. This depends on the service chosen in story 5.
- This story covers FR-16, FR-17, US-10, AC-8.
