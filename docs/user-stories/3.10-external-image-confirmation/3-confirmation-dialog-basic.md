## Parent Feature

#3.10 External-Image Confirmation

## What to build

When the user activates the show-images toggle and the confirmation preference is enabled (the default), display a modal confirmation dialog before loading any images. The dialog must include a privacy warning explaining that loading remote images can leak privacy-sensitive information. Confirming the dialog loads images for the current message. Cancelling leaves images blocked and makes no changes to preferences.

This slice introduces the basic dialog with confirm/cancel only — no whitelist checkboxes or global options yet. Those are added in subsequent slices.

## Acceptance criteria

- [ ] Clicking the show-images action presents a modal confirmation dialog (AC-2, FR-8)
- [ ] The dialog displays a privacy warning about remote image loading (FR-9)
- [ ] Confirming the dialog causes remote images to appear in the message (AC-3, FR-18)
- [ ] Cancelling the dialog leaves images blocked; no network request is made (AC-4, FR-19)
- [ ] The dialog is accessible via keyboard and carries appropriate screen-reader labels (NFR-6)

## Blocked by

- Blocked by `2-show-images-toggle`

## User stories addressed

- US-7 (confirmation dialog with privacy warning)

## Type

AFK
