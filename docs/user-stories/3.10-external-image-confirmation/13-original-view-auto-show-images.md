## Parent Feature

#3.10 External-Image Confirmation

## What to build

When the "always show images in original view" setting is enabled (slice 11 provides the toggle) and the user switches to the original/full HTML rendering of a message, images load automatically for that message without a separate confirmation dialog. The show-images toggle must reflect that images are displayed. If the user switches back to the safe/reformatted view, the image-display state reverts to whatever it was before the original-view switch.

## Acceptance criteria

- [ ] With the original-view auto-show setting enabled, switching to original/full HTML loads images automatically (AC-14, FR-30)
- [ ] No confirmation dialog is shown for the auto-loaded images
- [ ] The show-images toggle reflects that images are currently displayed (US-27)
- [ ] The user can re-block images while in original view (US-27)
- [ ] Switching back to safe/reformatted view reverts image state to its prior value (FR-31)
- [ ] With the setting disabled, switching to original view does not auto-load images

## Blocked by

- Blocked by `2-show-images-toggle`
- Blocked by `11-privacy-settings-screen`

## User stories addressed

- US-21 (setting for auto-show in original view — behavior)
- US-26 (images auto-load on switching to original view)
- US-27 (toggle reflects auto-shown state; can re-block)

## Type

AFK
