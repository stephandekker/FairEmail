## Parent Feature

#4.14 Attachments

## What to build

Image attachments in the compose list offer an "edit image" action for basic adjustments. This opens the image in an editor (system or built-in) for modifications before sending.

Covers epic sections: FR-28.

## Acceptance criteria

- [ ] Image attachments in the compose list offer an "edit image" context action.
- [ ] The action opens the image for basic editing/adjustments.
- [ ] Edits are applied to the attachment that will be sent (the original is replaced with the edited version).

## Blocked by

- Blocked by `5-image-inline-vs-attach`

## User stories addressed

- (FR-28 — no corresponding numbered user story in the epic; this is a functional requirement without an explicit US.)

## Notes

- The epic does not specify what "basic adjustments" encompasses (crop, rotate, brightness, etc.) or whether this uses a built-in editor or delegates to the system image editor. This is a design decision to be made before implementation.
