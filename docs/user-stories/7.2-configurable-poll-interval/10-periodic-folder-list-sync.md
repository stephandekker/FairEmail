## Parent Feature

#7.2 Configurable Poll Interval

## What to build

Add an optional setting to periodically synchronize the folder list itself, so that server-side folder changes (new folders, renamed folders, deleted folders) are detected between full connection cycles.

- The application provides a toggle (global or per-account) to enable periodic folder-list sync.
- When enabled, the folder list is refreshed on a periodic basis in addition to the initial sync at connection time.
- The cadence may be tied to the poll interval or operate on an independent schedule.

Covers epic section: §7.9 (FR-38).

## Acceptance criteria

- [ ] A setting exists to enable periodic folder-list synchronization.
- [ ] When enabled, the folder list is refreshed periodically (not only at initial connection).
- [ ] Server-side folder additions, renames, and deletions are detected during periodic refresh.
- [ ] The setting defaults to off (or a reasonable default that does not add unexpected load).

## Blocked by

- Blocked by 1-account-poll-interval-setting-and-scheduler

## Notes

- **OQ-7 from the epic** (folder-list sync cadence): Whether this should be tied to the poll interval or have its own independent cadence is an open question. The implementer should choose and document the approach.

## User stories addressed

- No explicit user story in §6 of the epic; this covers FR-38 directly.
