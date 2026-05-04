## Parent Feature

#3.10 External-Image Confirmation

## What to build

When a message is opened and its sender address matches an entry in the image whitelist, load remote images automatically without showing the confirmation dialog. This completes the per-sender whitelisting flow end-to-end: the user whitelists a sender via the dialog (slice 4), and subsequent messages from that sender auto-load images.

## Acceptance criteria

- [ ] Opening a message from a whitelisted sender address loads images automatically (AC-5, FR-23)
- [ ] No confirmation dialog is shown for whitelisted senders
- [ ] The show-images toggle reflects that images are loaded (FR-6)
- [ ] Messages from non-whitelisted senders still require manual action
- [ ] Auto-load works correctly after application restart (FR-22, AC-18)

## Blocked by

- Blocked by `4-per-sender-whitelist-in-dialog`

## User stories addressed

- US-15 (whitelisted sender auto-loads images)

## Type

AFK
