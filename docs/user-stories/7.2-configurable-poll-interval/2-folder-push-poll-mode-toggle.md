## Parent Feature

#7.2 Configurable Poll Interval

## What to build

Add a per-folder boolean setting that controls whether a folder uses push (IMAP IDLE, when available) or periodic polling. Wire the setting through the folder configuration UI and apply correct defaults based on folder type.

- Each folder has an independent push/poll toggle.
- The Inbox defaults to push mode (polling disabled).
- System folders other than Inbox (Drafts, Sent, Trash, Spam, Archive) default to polling mode (push disabled).
- User-created folders default to push mode but the user can switch them to polling.
- The folder configuration screen displays an informational note explaining that most servers limit push to a handful of folders.
- The scheduler respects this setting: folders in poll mode are included in the account's periodic sync cycle; folders in push mode are not polled by the scheduler (they rely on push, handled elsewhere).

Covers epic sections: §7.2 (FR-10 through FR-13, FR-15) and §6.2 (US-6, US-7, US-9).

## Acceptance criteria

- [ ] Each folder has a configurable push/poll mode toggle in the folder settings UI.
- [ ] The Inbox of a new account defaults to push mode (AC-6).
- [ ] System folders (Sent, Drafts, Trash, Spam, Archive) of a new account default to polling mode (AC-6).
- [ ] User-created folders default to push mode.
- [ ] The folder settings screen displays an informational note about server push limits (FR-15).
- [ ] Folders set to polling mode are included in the scheduler's periodic sync; folders in push mode are excluded.

## Blocked by

- Blocked by 1-account-poll-interval-setting-and-scheduler

## User stories addressed

- US-6 (configure push vs. polling per folder)
- US-7 (system folders default to polling)
- US-9 (informational note about server push limits)
