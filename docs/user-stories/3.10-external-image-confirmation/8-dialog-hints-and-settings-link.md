## Parent Feature

#3.10 External-Image Confirmation

## What to build

Add two elements to the confirmation dialog:

1. A hint that the user's choices can be reversed in the privacy settings screen (FR-16), reassuring users that experimenting with the options is safe.
2. A button or link that navigates directly to the privacy settings screen (FR-17), so the user can adjust related settings without hunting for them.

## Acceptance criteria

- [ ] The confirmation dialog displays a hint that choices can be undone in settings (FR-16)
- [ ] The dialog offers a shortcut (button or link) to the privacy settings screen (FR-17, AC-20)
- [ ] Activating the settings shortcut navigates to the privacy settings screen
- [ ] The hint text is understandable to a non-technical user (NFR-5)

## Blocked by

- Blocked by `3-confirmation-dialog-basic`
- Blocked by `11-privacy-settings-screen`

## User stories addressed

- US-13 (hint that choices can be reversed)
- US-14 (shortcut to privacy settings)

## Type

AFK
