## Parent Feature

#7.2 Configurable Poll Interval

## What to build

Add a bulk-apply operation that sets the polling mode for all system folders or all user folders of an account in one action. When enabling polling for system folders in bulk, automatically cap the account's interval at 15 minutes if it is currently higher.

- The account or folder management UI provides a bulk action to set poll mode for all system folders or all user folders at once.
- When the user bulk-enables polling for system folders and the account's poll interval exceeds 15 minutes, the application automatically reduces the interval to 15 minutes and informs the user why.
- The bulk action does not change the poll factor of individual folders; it only toggles push/poll mode.

Covers epic section: §7.3 (FR-16, FR-17) and §6.3 (US-10, US-11).

## Acceptance criteria

- [ ] The user can apply poll settings to all system folders or all user folders in one action.
- [ ] Enabling polling for all system folders via bulk apply, when the account interval exceeds 15 minutes, automatically reduces the account interval to 15 minutes (AC-8).
- [ ] The user is informed when the account interval is automatically reduced.
- [ ] Bulk apply does not alter individual folder poll factors.

## Blocked by

- Blocked by 2-folder-push-poll-mode-toggle

## User stories addressed

- US-10 (bulk apply poll settings to system/user folders)
- US-11 (auto-cap account interval on bulk enable)
