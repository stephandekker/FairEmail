# Error Report Capture & Transmission

## Parent Feature
#10.1 No Telemetry by Default

## User Story
As a user who has opted in, when the application encounters an error or abnormal state, I want it to send a report containing only technical error information — never my email content, credentials, contacts, or personal information — so that my privacy is maintained even while helping.

## Acceptance Criteria
- [ ] When error reporting is enabled and an error occurs, a report is transmitted to the configured error-reporting endpoint.
- [ ] Reports contain only: (a) exception type and stack trace, (b) application version and build information, (c) basic OS/architecture info, and (d) the anonymous identifier.
- [ ] Reports **never** contain: email message content, email headers, account credentials, email addresses, contact information, folder names, server hostnames, IP addresses, hardware serial numbers, or any other personally-identifiable information.
- [ ] The error-reporting service's own telemetry/analytics/session-tracking features are explicitly disabled in the application's configuration of that service.
- [ ] When error reporting is disabled and an error occurs, no report is transmitted (zero network traffic to the reporting endpoint).
- [ ] Error reports contain no more data than the minimum necessary to identify and diagnose the error.

## Complexity Estimate
Medium

## Blocked by
3-error-reporting-preference-toggle
4-anonymous-identifier-generation

## Notes
- Epic open question OQ-1 asks whether to use a new error-reporting service, restore the original (Bugsnag), or defer entirely. This story defines the behavioral contract regardless of service choice. The implementer must select a service that supports: disabling its own meta-telemetry, automatic report expiry (or API-based purging), and anonymous identifiers.
- Per epic design note N-4: many error-reporting services collect meta-telemetry by default. The application must explicitly opt out of these features.
- This story covers FR-12, FR-13, FR-15, US-8, NFR-4, AC-3, AC-4, AC-6, AC-11.
