## Parent Feature

#8.1 Desktop Notifications

## What to build

Play the configured notification sound when a new-mail notification is emitted, subject to rate limiting. Provide a configurable rate-limiting interval: when multiple notifications arrive within the interval, only the first produces a sound and subsequent ones are silent. Provide an "alert once" option: when enabled, only the first notification in a group produces a sound and updates to the group are silent.

This slice covers the global notification sound behavior only. Per-account/per-folder/per-sender sound customization is a separate story (story 12).

Covers epic sections: §7.4 (FR-14, FR-15, FR-16).

## Acceptance criteria

- [ ] A new-mail notification plays the configured notification sound
- [ ] With rate limiting set to 5 seconds, the first notification in a burst plays a sound; subsequent notifications within 5 seconds are silent (AC-5)
- [ ] The rate-limiting interval is user-configurable
- [ ] An "alert once" option is available: when enabled, only the first notification in a group produces a sound; group updates are silent
- [ ] Rate limiting and alert-once are independently configurable (per design note N-4)

## Blocked by

- Blocked by `1-notification-categories-and-basic-new-mail`

## User stories addressed

- US-25 (rate-limiting option for notification sounds)

## Notes

- OQ-8 (sound file format and location) is relevant: this story needs to decide whether to ship a default sound or rely on the desktop environment's notification sound. The epic does not prescribe this.
