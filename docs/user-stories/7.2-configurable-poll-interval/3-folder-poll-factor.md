## Parent Feature

#7.2 Configurable Poll Interval

## What to build

Add a per-folder poll factor (integer multiplier, default 1) that scales the account's base poll interval for that folder. The scheduler uses this factor to determine how many poll cycles to skip before actually checking the folder.

- Each folder in polling mode has a configurable poll factor (default 1).
- The effective interval for a folder is: `account poll interval × folder poll factor`.
- The scheduler tracks a per-folder poll counter and only syncs the folder when the counter reaches the factor value, then resets.
- The poll factor is exposed in the folder configuration UI alongside the push/poll toggle.

Covers epic section: §7.2 (FR-14) and §6.2 (US-8).

## Acceptance criteria

- [ ] Each folder has a configurable poll factor setting (integer multiplier, default 1).
- [ ] A folder with poll factor 3 on an account with a 10-minute interval is effectively polled every 30 minutes (AC-7).
- [ ] The poll factor is visible and editable in the folder configuration UI.
- [ ] A poll factor of 1 results in the folder being checked every account poll cycle (no skipping).

## Blocked by

- Blocked by 2-folder-push-poll-mode-toggle

## User stories addressed

- US-8 (per-folder poll factor multiplier)
