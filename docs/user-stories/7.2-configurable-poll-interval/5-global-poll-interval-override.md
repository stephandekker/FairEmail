## Parent Feature

#7.2 Configurable Poll Interval

## What to build

Add a global poll interval setting in the synchronization preferences that overrides per-account intervals when set to a non-zero value. Include a battery-usage warning and an informational hint about resource cost.

- The synchronization preferences screen includes a global poll interval selector with the same range as the account-level setting (0–1440 minutes).
- When set to a non-zero value, the global interval overrides per-account intervals for scheduling periodic sync cycles. Per-account intervals are preserved but inactive while the global override is in effect.
- When set to 0, per-account intervals govern (no override).
- When the global interval is set to a value greater than 0 and less than 15 minutes, a prominent warning is displayed: "This can cause high battery usage!"
- An informational hint near the setting explains that periodic checking compares local and remote messages every cycle and is resource-intensive compared to push.

Covers epic section: §7.4 (FR-18 through FR-22) and §6.4 (US-12, US-13, US-14).

## Acceptance criteria

- [ ] The synchronization preferences include a global poll interval selector with the defined range.
- [ ] Setting the global poll interval to 5 minutes displays a battery-usage warning (AC-9).
- [ ] Setting the global poll interval to a non-zero value causes all non-on-demand accounts to poll at that interval (AC-10).
- [ ] Setting the global poll interval to 0 allows per-account intervals to govern.
- [ ] An informational hint explains the resource cost of periodic polling (FR-22).

## Blocked by

- Blocked by 1-account-poll-interval-setting-and-scheduler

## User stories addressed

- US-12 (global poll interval for uniform cadence)
- US-13 (warning for frequent global interval)
- US-14 (informational hint about resource cost)
