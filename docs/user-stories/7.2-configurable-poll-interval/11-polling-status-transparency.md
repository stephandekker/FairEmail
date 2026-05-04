## Parent Feature

#7.2 Configurable Poll Interval

## What to build

Surface the current polling state of each account so the user can determine whether an account is using push or polling, and what the effective interval is.

- The account status display or operations log indicates whether each account is currently using push, polling, or both.
- The effective interval (accounting for global override, auto-tuning adjustments, and folder poll factors) is reported.
- This information is visible without requiring the user to navigate deep into settings — it should be accessible from the account list or account detail view.

Covers epic NFR-4 and AC-18.

## Acceptance criteria

- [ ] The account status or operations log indicates whether each account is currently using push or polling (AC-18).
- [ ] The effective poll interval is reported for each account (AC-18).
- [ ] When a global override is active, the display reflects the overridden interval, not the per-account setting.
- [ ] When auto-tuning has adjusted the interval, the display reflects the tuned value.

## Blocked by

- Blocked by 1-account-poll-interval-setting-and-scheduler
- Blocked by 5-global-poll-interval-override
- Blocked by 7-auto-tuning-keep-alive
- Blocked by 8-push-poll-interaction-and-fallback

## User stories addressed

- No explicit user story in §6 of the epic; this covers NFR-4 and AC-18 directly.
