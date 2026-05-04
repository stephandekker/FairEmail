# Error and Crash Reporting Opt-In Gate

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, I want error and crash reporting to be completely disabled by default, so that no diagnostic data leaves my device without my consent. As a user who wants to help improve the application, I want to be able to opt in with a clear explanation of what data will be sent and to whom.

## Blocked by
- `9-optional-service-gating-framework` (this service uses the generic gating framework)

## Acceptance Criteria
- Error/crash reporting is completely disabled by default.
- Enabling it requires toggling a setting and produces a disclosure of what data will be sent (stack traces, device information, application state) and to which service.
- Disabling it stops all reporting immediately with no residual connections or scheduled callbacks.
- The toggle appears in the consolidated optional-services view.
- No diagnostic data is transmitted unless the user has explicitly opted in.

## Mapping to Epic
- US-20, US-21
- FR-23
- AC-6

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This is the first concrete optional service implementation and serves as the reference pattern for subsequent service gates.
