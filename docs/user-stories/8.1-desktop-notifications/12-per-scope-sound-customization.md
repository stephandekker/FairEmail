## Parent Feature

#8.1 Desktop Notifications

## What to build

Allow the notification sound to be configured at each level of the precedence chain: globally, per-account, per-folder, and per-sender, with the most specific setting taking precedence. When a per-sender custom sound is configured, notifications from that sender use the custom sound regardless of folder or account defaults.

Covers epic sections: §7.4 (FR-17).

## Acceptance criteria

- [ ] Notification sound can be configured globally (already exists from story 7), per-account, per-folder, and per-sender
- [ ] The most specific sound setting takes precedence (sender > folder > account > global)
- [ ] Configuring a custom sound for a specific sender causes notifications from that sender to use the custom sound (AC-15)
- [ ] Removing a per-scope sound override causes fallback to the next level in the chain

## Blocked by

- Blocked by `6-notification-sound-and-rate-limiting`
- Blocked by `11-notification-precedence-resolution`

## User stories addressed

- (AC-15 — per-sender sound override)

## Notes

- OQ-8 (sound file format and location) is relevant: this story needs to define how custom sound files are referenced and what formats are supported.
