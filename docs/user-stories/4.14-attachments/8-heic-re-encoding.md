## Parent Feature

#4.14 Attachments

## What to build

When a user attaches a HEIC/HEIF image and chooses to resize it, the application re-encodes the output into a universally supported format (JPEG or PNG) so that recipients on platforms that do not support HEIC can view the image.

Covers epic sections: US-8, FR-15, AC-3.

## Acceptance criteria

- [ ] Attaching a HEIC/HEIF image with resize enabled produces a JPEG or PNG output file (AC-3).
- [ ] The re-encoded file is correctly typed with the new MIME type and appropriate file extension.
- [ ] The re-encoding preserves visual quality at a level comparable to the resize target.

## Blocked by

- Blocked by `6-image-resize`

## User stories addressed

- US-8

## Notes

- OQ-3 from the epic asks whether HEIC images should always be re-encoded on attach (even without resize). This story implements the epic's current specification: re-encoding only when resize is requested. If the answer to OQ-3 changes, this story's scope would expand.
