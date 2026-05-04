## Parent Feature

#3.10 External-Image Confirmation

## What to build

When images are shown for a message (by any means: user action, whitelist match, or global setting) and tracker detection (feature 3.11) is enabled, images identified as tracking pixels by the tracker-detection subsystem must remain blocked and be replaced with a visual indicator. The tracker-detection gate operates independently of the image-confirmation gate: disabling image confirmation does not disable tracker detection, and vice versa (FR-33).

This slice defines the integration boundary between this epic (3.10) and the tracker-detection epic (3.11). The actual detection logic belongs to 3.11; this slice ensures that 3.10's image-loading respects 3.11's verdicts.

## Acceptance criteria

- [ ] When images are shown and tracker detection is enabled, tracking pixels are blocked and replaced with a visual indicator (AC-12, FR-32)
- [ ] Non-tracking remote images load normally when the user shows images
- [ ] Disabling image confirmation does not disable tracker detection (FR-33)
- [ ] Disabling tracker detection allows all images to load when the user shows images
- [ ] The visual indicator for blocked tracking images is distinguishable from the indicator for all-images-blocked state

## Blocked by

- Blocked by `2-show-images-toggle`

## User stories addressed

- US-24 (tracking pixels remain blocked with visual indicator)

## Notes

- This slice depends on feature 3.11 (Tracker-image detection) providing an API or mechanism to query whether a given image is a tracking pixel. If 3.11 is not yet implemented, this slice should define and code against an interface/contract that 3.11 will fulfill, with a default "no tracking images detected" stub.

## Type

AFK
