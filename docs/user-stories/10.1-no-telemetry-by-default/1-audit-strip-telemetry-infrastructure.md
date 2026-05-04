# Audit & Ensure Architectural Absence of Telemetry

## Parent Feature
#10.1 No Telemetry by Default

## User Story
As a privacy-conscious user, I want the application to contain no analytics infrastructure whatsoever — not merely disabled analytics, but the complete absence of usage tracking, behavioral metrics, and session recording — so that there is no hidden capability that could be enabled without my knowledge.

## Acceptance Criteria
- [ ] The application codebase contains zero analytics, behavioral-tracking, or session-recording subsystems — neither active nor dormant.
- [ ] A fresh installation with default settings generates zero network connections to any analytics, telemetry, or error-reporting endpoint (verifiable by network traffic monitoring).
- [ ] No application feature is degraded, hidden, or restricted based on whether error reporting is enabled or disabled.
- [ ] No third-party dependency bundles analytics or telemetry that runs automatically on application startup.
- [ ] A code audit (grep for known telemetry patterns, dependency review) confirms no dormant telemetry exists.

## Complexity Estimate
Small

## Blocked by
(none — this is the foundational story)

## Notes
- The existing FairEmail Android codebase already removed Bugsnag from the Java source (only stub references remain in patches/ and metadata/). This story ensures the desktop application starts from a clean baseline and that no new telemetry dependencies are introduced.
- Per epic design note N-1: the absence of analytics should be architectural, not configurational. Do not include an analytics subsystem "for future use."
- This story covers FR-1, FR-2, US-1, US-2, NFR-1, NFR-6, AC-1, AC-10.
