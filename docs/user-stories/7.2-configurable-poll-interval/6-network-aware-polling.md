## Parent Feature

#7.2 Configurable Poll Interval

## What to build

Add network-aware polling controls: two global toggles (metered and unmetered) that suppress or allow periodic polling based on network type, plus a per-account exemption flag that overrides the global toggles.

- Synchronization preferences include two independent toggles: "Poll on metered network" and "Poll on unmetered network."
- When a network-type toggle is disabled, no periodic polling occurs on that network type. Accounts rely on push (if available) or wait until the network type changes.
- Each account has an "exempt from network restrictions" flag. When enabled, the account polls at its configured interval regardless of the network-type toggles.
- The scheduler checks current network type and respects these settings before initiating a poll cycle.

Covers epic section: §7.5 (FR-23 through FR-25) and §6.5 (US-15, US-16).

## Acceptance criteria

- [ ] Synchronization preferences include metered and unmetered polling toggles.
- [ ] With "poll on metered network" disabled and the device on a metered connection, no automatic polling occurs (AC-12).
- [ ] With "poll on unmetered network" disabled and the device on an unmetered connection, no automatic polling occurs.
- [ ] An account marked as "exempt from network restrictions" polls on schedule even when the network-type toggle would otherwise suppress polling (AC-13).
- [ ] The exemption flag is configurable per account in account settings.

## Blocked by

- Blocked by 1-account-poll-interval-setting-and-scheduler

## User stories addressed

- US-15 (enable/disable polling by network type)
- US-16 (per-account exemption from network restrictions)
