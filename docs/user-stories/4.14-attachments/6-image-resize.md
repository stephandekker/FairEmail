## Parent Feature

#4.14 Attachments

## What to build

Extend the image options dialog to offer resizing at attach time. The user can select a target maximum pixel dimension from a configurable set of options. A "limit width only" toggle constrains only the horizontal dimension while preserving aspect ratio. Resizing uses integer scaling factors to preserve sharpness (N-3). Supported input formats: JPEG, PNG, WebP, and HEIC/HEIF (FR-14). A global setting controls the available resize target dimensions with a sensible default (e.g. 1440 px).

Progress feedback is shown if the resize operation exceeds one second (NFR-1).

Covers epic sections: US-5, US-6, FR-12 (resize parts), FR-14, FR-17, FR-19, AC-2 (resize part), N-3.

## Acceptance criteria

- [ ] The image options dialog offers a resize option with selectable target pixel dimensions.
- [ ] A "limit width only" toggle is available that constrains only horizontal dimension, preserving aspect ratio (US-6).
- [ ] Resizing uses integer scaling factors and preserves aspect ratio (N-3).
- [ ] JPEG, PNG, WebP, and HEIC/HEIF input formats are supported for resize (FR-14).
- [ ] Progress feedback is shown for resize operations exceeding 1 second (NFR-1).
- [ ] The set of resize target dimensions is configurable in send settings with a sensible default (FR-19).

## Blocked by

- Blocked by `5-image-inline-vs-attach`

## User stories addressed

- US-5
- US-6
