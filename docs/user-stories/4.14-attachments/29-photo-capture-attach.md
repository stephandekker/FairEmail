## Parent Feature

#4.14 Attachments

## What to build

Provide a mechanism in the compose window to capture a photo directly (via webcam or system camera interface) and have it attached immediately to the current draft. This avoids the need to capture a photo externally and then attach it via the file picker.

Covers epic sections: US-15, FR-3.

## Acceptance criteria

- [ ] The compose window offers an action to capture a photo directly.
- [ ] The captured photo is immediately attached to the current draft.
- [ ] If the image options dialog is enabled, it is presented for the captured photo (inline/attach, resize, etc.).

## Blocked by

- Blocked by `5-image-inline-vs-attach`

## User stories addressed

- US-15

## Notes

- On a Linux desktop, this will likely use the system's webcam interface. The exact mechanism may differ significantly from the Android source's camera intent. The epic does not prescribe the capture mechanism — only that it is available from the compose window.
