# Privacy Documentation & Endpoint Transparency

## Parent Feature
#10.1 No Telemetry by Default

## User Story
As a verifier, I want the application's privacy policy, FAQ, and documentation to explicitly state that no data is sent except when error reporting is explicitly enabled, and to document the network endpoints used, so that I can independently audit network traffic against the application's claims.

## Acceptance Criteria
- [ ] The privacy policy explicitly states that no data is transmitted to the developer or any third party except when error reporting is explicitly enabled by the user.
- [ ] The FAQ or help documentation includes an entry explaining: what error reporting is, what data is sent, how it is anonymized, how long reports are retained, and how to enable or disable it.
- [ ] The network endpoints used for error report submission are documented in the privacy policy or FAQ.
- [ ] Documentation accurately describes the default-off posture and what opting in entails.
- [ ] The deletion request mechanism is documented (how to request removal of reports by referencing the anonymous identifier).

## Complexity Estimate
Small

## Blocked by
5-error-report-capture-and-transmission
9-report-retention-and-deletion

## Notes
- This story is intentionally last in build order because the documentation must accurately reflect the implemented behavior. It cannot be finalized until the error-reporting service, endpoints, and retention policy are determined.
- Per epic design note N-6: the no-telemetry stance is a core value proposition. Documentation should prominently advertise it, not bury it.
- This story covers FR-20, FR-21, FR-22, US-14, US-15, US-16, AC-9.
