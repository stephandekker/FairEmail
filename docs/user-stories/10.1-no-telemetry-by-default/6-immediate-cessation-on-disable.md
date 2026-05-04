# Immediate Cessation on Consent Revocation

## Parent Feature
#10.1 No Telemetry by Default

## User Story
As any user who disables error reporting, I want transmission of error data to cease immediately — no queued reports should be sent after I disable the setting — so that revocation has instant effect.

## Acceptance Criteria
- [ ] When the user disables error reporting, no further error reports are transmitted from that moment onward.
- [ ] Any queued or buffered reports that have not yet been sent are discarded (not sent) when error reporting is disabled.
- [ ] Toggling error reporting on and then off again results in immediate cessation — no delayed or deferred reports are sent after disabling.
- [ ] No application update, configuration migration, or account change silently re-enables error reporting once the user has disabled it.

## Complexity Estimate
Small

## Blocked by
5-error-report-capture-and-transmission

## Notes
- This story focuses specifically on the revocation edge cases (queue flushing, deferred sends, silent re-enablement). The basic toggle behavior is established in story 3; this story hardens it.
- This story covers FR-6 (disable side), US-13, NFR-7, AC-13.
