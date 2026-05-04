## Parent Feature

#3.10 External-Image Confirmation

## What to build

Add three image-related toggles to the privacy settings screen:

1. **Master toggle** for blocking remote images by default (on = block, off = allow). This is the primary control (FR-26a).
2. **Confirmation toggle** for showing the confirmation dialog before loading images. This toggle is only configurable when the master toggle is enabled; if the master toggle is off, the confirmation toggle becomes moot/disabled (FR-26b, FR-27).
3. **Original-view auto-show toggle** for automatically showing images when the user switches to original/full HTML rendering (FR-26c).

All three toggles must be persisted and must control the corresponding behaviors implemented in other slices.

## Acceptance criteria

- [ ] The privacy settings screen exposes all three image-related toggles (AC-19, FR-26)
- [ ] The confirmation toggle is only configurable when the master block toggle is enabled (AC-19, FR-27)
- [ ] Disabling the master toggle makes the confirmation toggle non-configurable
- [ ] All toggle states persist across application restarts
- [ ] Toggle defaults match the epic: master block = on, confirmation = on, original-view auto-show = off
- [ ] Settings are accessible via keyboard and respect the application-wide theme (NFR-6)

## Blocked by

- Blocked by `1-block-remote-images-by-default`

## User stories addressed

- US-19 (master toggle for image blocking)
- US-20 (confirmation toggle independent of blocking)
- US-21 (original-view auto-show setting — toggle only; behavior in slice 13)

## Type

AFK
