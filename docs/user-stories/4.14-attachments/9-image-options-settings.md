## Parent Feature

#4.14 Attachments

## What to build

Add settings to control the image options dialog behavior: (1) a global setting to suppress/re-enable the image options dialog ("don't show again" / re-enable from settings), and (2) a setting to automatically resize images in replied/quoted text without per-message intervention.

Covers epic sections: US-9, US-10, FR-13, FR-18.

## Acceptance criteria

- [ ] A global setting controls whether the image options dialog appears each time an image is added (US-9).
- [ ] The dialog can be suppressed via a "don't show again" option, and re-enabled from settings (FR-13).
- [ ] A setting enables automatic resizing of images in replied/quoted text (US-10, FR-18).
- [ ] When auto-resize is enabled, images in quoted text are resized without user intervention per message.

## Blocked by

- Blocked by `6-image-resize`
- Blocked by `7-image-metadata-strip`

## User stories addressed

- US-9
- US-10

## Notes

- OQ-2 from the epic asks whether auto-resize of images in reply text should default to on or off. This needs a design decision before implementation.
