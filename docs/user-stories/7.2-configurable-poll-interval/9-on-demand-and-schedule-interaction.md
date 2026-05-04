## Parent Feature

#7.2 Configurable Poll Interval

## What to build

Ensure that on-demand (manual sync) accounts are fully exempt from automatic polling, and that accounts with a schedule exemption continue polling during sync-schedule blackout periods.

- Accounts configured as on-demand never initiate automatic polling, regardless of per-account interval, global interval, or any other setting. Polling occurs only when the user explicitly triggers a sync.
- Accounts with an "ignore schedule" exemption continue to poll at their configured interval even during global sync schedule blackout periods (defined by epic 7.3).
- Changes to the poll interval take effect on the next scheduled poll cycle without requiring an application restart.

Covers epic section: §7.8 (FR-35, FR-36, FR-37) and §6.8 (US-23, US-24).

## Acceptance criteria

- [ ] An account marked as on-demand never polls automatically, regardless of the global or per-account interval (AC-11).
- [ ] An account exempt from the sync schedule continues polling during schedule blackout periods (AC-16).
- [ ] On-demand status takes precedence over all interval settings (global, per-account, network toggles).
- [ ] The "ignore schedule" flag is configurable per account in account settings.

## Blocked by

- Blocked by 1-account-poll-interval-setting-and-scheduler
- Blocked by 5-global-poll-interval-override

## Notes

- The schedule blackout feature is defined in epic 7.3, not this epic. This story only ensures the exemption flag is respected by the poll scheduler. If epic 7.3 is not yet implemented, the "ignore schedule" flag can be stored and persisted but will have no observable effect until schedule support exists.

## User stories addressed

- US-23 (on-demand accounts never auto-poll)
- US-24 (schedule-exempt accounts poll during blackouts)
