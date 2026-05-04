## Parent Feature

#4.14 Attachments

## What to build

Extend the image options dialog to offer privacy-sensitive metadata removal. When the user enables this option, the application strips: GPS coordinates (latitude, longitude, altitude, speed), timestamps, image descriptions, artist information, camera/lens serial numbers, XMP data, and user comments. The original filename is replaced with a generic sequential identifier. The strip is opt-in per image (N-2).

Covers epic sections: US-7, FR-12 (privacy-strip part), FR-16, AC-2 (privacy-strip part), AC-4, NFR-3, N-2.

## Acceptance criteria

- [ ] The image options dialog offers a "remove privacy-sensitive data" option.
- [ ] When enabled, GPS coordinates, timestamps, serial numbers, artist info, XMP data, image descriptions, and user comments are removed (AC-4).
- [ ] The original filename is replaced with a generic sequential identifier (FR-16).
- [ ] No residual GPS, serial number, or XMP data survives the strip operation (NFR-3).
- [ ] The stripped filename reveals no information about the original file path (NFR-3).
- [ ] The option is opt-in per image, not automatic (N-2).

## Blocked by

- Blocked by `5-image-inline-vs-attach`

## User stories addressed

- US-7
