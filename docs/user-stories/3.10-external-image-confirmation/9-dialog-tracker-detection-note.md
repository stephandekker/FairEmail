## Parent Feature

#3.10 External-Image Confirmation

## What to build

Add a note to the confirmation dialog informing the user that images identified as tracking images will remain blocked even if they proceed. This note must be visible only when tracker detection (feature 3.11) is enabled, and hidden when it is disabled. The note's visibility must react to the current state of the tracker detection setting.

This slice covers only the dialog UI note and its conditional visibility. The actual tracker-detection blocking behavior is handled in slice 14.

## Acceptance criteria

- [ ] The confirmation dialog displays a tracking-image note when tracker detection is enabled (AC-13, FR-10)
- [ ] The tracking-image note is hidden when tracker detection is disabled (AC-13, FR-11)
- [ ] The note text is clear to a non-technical user

## Blocked by

- Blocked by `3-confirmation-dialog-basic`

## User stories addressed

- US-8 (tracking-image note in confirmation dialog)
- US-25 (note visible only when tracker detection is enabled)

## Notes

- This slice depends on being able to read the tracker detection enabled/disabled state from feature 3.11's settings. If feature 3.11 has not been implemented yet, the note should be driven by a setting/flag that 3.11 will later control, defaulting to disabled.

## Type

AFK
