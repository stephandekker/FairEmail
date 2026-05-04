## Parent Feature

#3.10 External-Image Confirmation

## What to build

Add a global "Do not ask this again" checkbox to the confirmation dialog. When checked and confirmed, the confirmation dialog is suppressed for all future show-images actions — the toggle loads images immediately without a dialog.

Additionally, when the global checkbox is checked, a nested "Show images by default" checkbox becomes enabled. If both are checked and confirmed, remote images load automatically on all messages without any user action, effectively disabling the blocking default globally.

## Acceptance criteria

- [ ] The confirmation dialog offers a "Do not ask this again" (global) checkbox (FR-14)
- [ ] After checking global option and confirming, subsequent show-images actions on any message proceed without a dialog (AC-8)
- [ ] When the global checkbox is checked, a nested "Show images by default" checkbox becomes enabled (FR-15)
- [ ] After checking "Show images by default" and confirming, messages display remote images automatically (AC-9)
- [ ] Cancelling the dialog does not change any global preference (FR-19)
- [ ] Both preferences are persisted across restarts

## Blocked by

- Blocked by `3-confirmation-dialog-basic`

## User stories addressed

- US-11 (global "do not ask again" option)
- US-12 (nested "show images by default" option)

## Type

AFK
