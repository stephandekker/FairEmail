## Parent Feature

#3.10 External-Image Confirmation

## What to build

When a message resides in a folder identified as Junk or Spam, the show-images action must load images immediately without presenting the confirmation dialog, regardless of the global confirmation setting. This removes unnecessary friction when the user is already inspecting suspicious content (design note N-4).

## Acceptance criteria

- [ ] In a Junk/Spam folder, the show-images action loads images without a confirmation dialog (AC-11, FR-29)
- [ ] This behavior applies regardless of whether confirmation is globally enabled
- [ ] The show-images toggle still functions as a toggle (can re-block) in Junk/Spam folders
- [ ] Messages in non-Junk/Spam folders are unaffected by this exception

## Blocked by

- Blocked by `2-show-images-toggle`

## User stories addressed

- US-23 (junk/spam folder bypasses confirmation)

## Notes

- This slice requires the ability to determine whether a message's folder is identified as Junk or Spam. This depends on the folder type detection from feature 2.2 (Special-folder auto-detection). If that is not yet available, folder identification may need a stub.

## Type

AFK
