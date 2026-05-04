## Parent Feature

#3.10 External-Image Confirmation

## What to build

Add a discoverable show-images action (button or toolbar control) to the message view. The action functions as a toggle: activating it loads all remote images for the current message; activating it again re-blocks them. The toggle's visual state clearly indicates whether images are currently shown or blocked. Each message's show/block state is independent — showing images on one message does not affect any other message.

This slice wires the toggle directly to loading/blocking without any confirmation dialog. The confirmation dialog is layered on top in a subsequent slice. This allows the toggle mechanism to be built and verified in isolation.

## Acceptance criteria

- [ ] A show-images action is visible on the message view whenever the message contains blocked remote images (FR-4, NFR-5)
- [ ] Activating the action loads remote images for the current message (FR-5)
- [ ] Activating the action again re-blocks remote images for that message (FR-5, AC-15)
- [ ] The toggle's visual state reflects whether images are currently shown or blocked (FR-6)
- [ ] Showing images on one message does not cause images to appear on any other message (FR-7, AC-16)
- [ ] The per-message state does not persist across sessions — reopening resets to blocked (see epic §10, OQ-6)

## Blocked by

- Blocked by `1-block-remote-images-by-default`

## User stories addressed

- US-4 (single discoverable show-images action)
- US-5 (toggle back to re-block)
- US-6 (per-message independence)

## Type

AFK
