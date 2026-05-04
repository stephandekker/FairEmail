## Parent Feature

#8.1 Desktop Notifications

## What to build

When a contact photo or avatar is available for the sender of a new message, display it as the notification icon or image so the user can visually identify the sender at a glance.

Covers epic sections: §7.2 (FR-7).

## Acceptance criteria

- [ ] If a contact photo or avatar is available for the sender, the notification displays it as the notification icon or image
- [ ] If no avatar is available, the notification falls back to a default icon (application icon or generated placeholder)
- [ ] Avatar display works within the notification grouping model (individual notifications show the sender's avatar)

## Blocked by

- Blocked by `1-notification-categories-and-basic-new-mail`

## User stories addressed

- US-28 (sender contact photo or avatar in notifications)

## Notes

- The epic's non-goal NG2 references epic 8.6 (sender avatars in notifications) as a separate epic. There may be overlap between this story (FR-7, which is in THIS epic) and epic 8.6. FR-7 clearly belongs to this epic, but the detailed avatar-sourcing behavior (BIMI, Gravatar, generated avatars) may be defined in 8.6. This story covers the notification integration — showing whatever avatar is available — not the avatar-sourcing pipeline itself.
