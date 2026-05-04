## Parent Feature

#7.2 Configurable Poll Interval

## What to build

Implement automatic tuning of the keep-alive interval based on observed server behavior. The application detects premature connection drops, reduces the interval in steps, and resets tuning state when the user manually changes the interval.

- A global or per-account "automatically tune keep-alive interval" setting (default enabled).
- When auto-tuning is active and the server prematurely terminates an idle connection, the application reduces the account's keep-alive interval by 2 minutes and retries.
- Auto-tuning does not reduce the interval below a floor of 9 minutes.
- The application tracks consecutive keep-alive successes and failures per account to inform tuning decisions.
- When the user manually changes an account's poll interval, all auto-tuning state (success/failure counters, tuned status) is reset, and tuning restarts from the new baseline.
- When auto-tuning detects persistently unreliable push behavior, it may recommend switching the account to polling-only mode at a 15-minute interval.

Covers epic section: §7.6 (FR-26 through FR-31) and §6.6 (US-17, US-18, US-19).

## Acceptance criteria

- [ ] An "automatically tune keep-alive interval" setting exists (default enabled).
- [ ] When auto-tuning detects a premature server disconnect, the keep-alive interval is reduced by 2 minutes down to a floor of 9 minutes (AC-14).
- [ ] The application tracks success/failure counters per account (FR-29).
- [ ] Manually changing an account's interval resets its auto-tuning state (AC-15).
- [ ] Persistently unreliable push may trigger a recommendation to switch to polling at 15 minutes (FR-31).

## Blocked by

- Blocked by 1-account-poll-interval-setting-and-scheduler

## Notes

- **OQ-2 from the epic** (auto-tune step size): The epic specifies a fixed 2-minute step. Whether a more adaptive algorithm should be used is an open question flagged in the epic. This story implements the fixed-step approach as specified.
- **OQ-3 from the epic** (auto-optimization recommendation): Whether the recommendation to switch to polling should be automatic or require user confirmation is unresolved. This story should document the chosen approach during implementation.

## User stories addressed

- US-17 (auto-reduce keep-alive on premature disconnect)
- US-18 (auto-tuning floor of 9 minutes)
- US-19 (manual change resets auto-tuning state)
