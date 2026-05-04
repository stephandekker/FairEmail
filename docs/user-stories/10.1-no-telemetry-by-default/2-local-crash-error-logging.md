# Local Crash & Error Logging

## Parent Feature
#10.1 No Telemetry by Default

## User Story
As any user, when the application crashes or encounters an error while error reporting is disabled, I want the crash information to be recorded only locally on my machine, so that I can choose to share it manually if I wish but nothing is sent automatically.

## Acceptance Criteria
- [ ] When the application encounters an unhandled exception or crash with error reporting disabled, error information (stack trace, error type, application version) is written to a local log file accessible to the user.
- [ ] No data is transmitted to any external service when an error occurs and error reporting is disabled.
- [ ] The local crash log file location is discoverable by the user (e.g. documented in help, or accessible via a menu item).
- [ ] Local crash logging works identically regardless of whether error reporting is enabled or disabled.

## Complexity Estimate
Small

## Blocked by
1-audit-strip-telemetry-infrastructure

## Notes
- Epic open question OQ-2 asks whether local crash logs should always be written or only in development/beta modes. This story assumes always-on local logging for the desktop application, since desktop users are more likely to be technically capable of reading logs and the privacy cost is zero (data stays local). If this decision is overridden, adjust accordingly.
- This story covers FR-3, US-3, AC-12.
- This story is distinct from error *reporting* (remote transmission). It is purely local file I/O.
