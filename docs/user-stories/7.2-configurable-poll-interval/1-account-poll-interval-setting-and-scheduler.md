## Parent Feature

#7.2 Configurable Poll Interval

## What to build

The foundational vertical slice: persist a configurable poll interval per account, expose it in the account configuration UI, and wire it to a scheduler that triggers sync cycles at the configured cadence.

- Each account stores a poll interval in minutes (default 15).
- The UI presents a selector with the defined set of choices: 0, 1, 2, 5, 10, 15, 30, 60, 120, 240, 480, 1440 minutes.
- Any value below 1 minute (other than 0) is clamped to 1.
- For POP3 accounts, the interval always drives a full periodic poll cycle.
- For IMAP accounts, the interval is stored identically; its interpretation as keep-alive vs. full poll is deferred to later slices that handle push interaction. In this slice, treat it as a poll interval for scheduling purposes.
- The scheduler triggers a sync cycle for each account at its configured interval. If the network is unavailable, the cycle is skipped and resumes when connectivity returns.
- Changing the interval takes effect on the next cycle without requiring an application restart.

Covers epic sections: §7.1 (FR-1 through FR-4, FR-8, FR-9), §7.8 (FR-37), and the default-interval aspects of §6.1.

## Acceptance criteria

- [ ] A newly created IMAP account has a default poll interval of 15 minutes (AC-1).
- [ ] A newly created POP3 account has a default poll interval of 15 minutes (AC-2).
- [ ] The user can select a poll interval from the defined set (0, 1, 2, 5, 10, 15, 30, 60, 120, 240, 480, 1440) for any account (AC-3).
- [ ] The scheduler triggers sync for each account at its configured interval.
- [ ] Changing a poll interval takes effect on the next cycle without application restart (AC-17).
- [ ] The interval setting label and hint explain its dual role (keep-alive for push vs. poll for non-push) as per FR-9.

## Blocked by

None — can start immediately.

## User stories addressed

- US-1 (set keep-alive / poll interval for IMAP)
- US-2 (set poll interval for POP3)
- US-3 (default 15-minute interval)
- US-4 (range of interval choices)
