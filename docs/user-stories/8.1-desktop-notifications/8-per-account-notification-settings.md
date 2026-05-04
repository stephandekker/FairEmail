## Parent Feature

#8.1 Desktop Notifications

## What to build

Add a per-account notification enable/disable toggle that overrides the global default for all folders in that account. The toggle is accessible from the account's settings context. When an account has notifications disabled, no new-mail notifications are produced for any folder in that account, regardless of the global or per-folder settings (unless a higher-precedence override exists).

Covers epic sections: §7.7 (FR-26, FR-30 — account context).

## Acceptance criteria

- [ ] Each account has an independent notification enable/disable toggle
- [ ] Disabling notifications for an account suppresses all new-mail notifications for that account's folders
- [ ] The per-account toggle overrides the global setting
- [ ] The per-account toggle is accessible from the account's settings/context menu
- [ ] Enabling the per-account toggle restores notification behavior per the folder-level defaults

## Blocked by

- Blocked by `7-global-notification-configuration`

## User stories addressed

- US-21 (per-account notification settings overriding global defaults)
