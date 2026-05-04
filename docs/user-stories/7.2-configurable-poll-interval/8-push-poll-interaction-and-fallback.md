## Parent Feature

#7.2 Configurable Poll Interval

## What to build

Implement the dual-purpose behavior of the poll interval setting for IMAP accounts with push capability, the "poll on keep-alive" option, and the automatic fallback to polling when push is unavailable.

- For IMAP accounts with push capability and interval set to 0: use push only, no periodic polling. Maintain a persistent connection and rely on the server to notify of changes.
- For IMAP accounts with push capability and a non-zero interval at or above the keep-alive threshold (9 minutes): the interval controls the keep-alive signal frequency. Push remains the primary delivery mechanism.
- For IMAP accounts with push capability and a non-zero interval below the keep-alive threshold, or for accounts where push is not supported: the interval controls a full periodic poll cycle.
- An optional "poll on keep-alive" setting triggers a sync check alongside each keep-alive signal on a push connection, catching changes the push channel may have missed.
- When an account's interval is set to 0 (push only) and push is not supported by the server, the application automatically falls back to polling at the default interval (15 minutes) and informs the user.

Covers epic sections: §7.1 (FR-5, FR-6, FR-7), §7.7 (FR-32 through FR-34), and §6.7 (US-20, US-21, US-22), plus §6.1 (US-5).

## Acceptance criteria

- [ ] Setting interval to 0 on an IMAP account with push results in push-only mode with no periodic polling (AC-4).
- [ ] Setting interval to 0 on a POP3 account or IMAP without push results in automatic fallback to 15-minute polling (AC-5).
- [ ] An IMAP account with push and interval ≥ 9 minutes uses the interval as keep-alive cadence, not full poll.
- [ ] An IMAP account with push and interval < 9 minutes (non-zero) triggers full periodic polling instead of keep-alive.
- [ ] The "poll on keep-alive" option triggers a sync check alongside each keep-alive signal (FR-33).
- [ ] The user is informed when fallback to polling occurs.

## Blocked by

- Blocked by 1-account-poll-interval-setting-and-scheduler
- Blocked by 2-folder-push-poll-mode-toggle

## Notes

- **OQ-1 from the epic** (keep-alive threshold): The epic uses 9 minutes as the threshold. Whether this should be adjustable is an open question. This story implements 9 minutes as a fixed threshold.
- **OQ-8 from the epic** (poll on keep-alive default): Whether the "poll on keep-alive" option should default to on or off is unresolved. The implementer should document the chosen default.
- **OQ-6 from the epic** (NOOP vs IDLE keep-alive): The source application offers a NOOP option as a workaround for broken IDLE. Whether to expose this as a user-facing option or handle it via auto-tuning is unresolved.

## User stories addressed

- US-5 (push-only / always connected mode)
- US-20 (understand interval as keep-alive vs. poll)
- US-21 (poll on keep-alive option)
- US-22 (fallback to polling when push unavailable)
