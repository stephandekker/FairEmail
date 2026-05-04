## Parent Feature

#3.10 External-Image Confirmation

## What to build

Add a per-sender checkbox to the confirmation dialog: "Do not ask this again for [sender address]". When the user checks this option and confirms the dialog, the sender address is persistently recorded in the image whitelist. The whitelist entry must survive application restarts.

This slice covers the dialog UI addition and the persistence of the sender whitelist entry. The auto-load behavior (skipping the dialog for whitelisted senders) is handled in the next slice.

## Acceptance criteria

- [ ] The confirmation dialog offers a "Do not ask this again for [sender address]" checkbox (FR-12)
- [ ] The checkbox displays the actual sender address of the current message
- [ ] Confirming with the checkbox checked persists the sender address in the whitelist (FR-20)
- [ ] Confirming with the checkbox unchecked does not add a whitelist entry
- [ ] Cancelling the dialog does not persist any whitelist entry regardless of checkbox state (FR-19)
- [ ] The whitelist entry survives application restart (FR-22)

## Blocked by

- Blocked by `3-confirmation-dialog-basic`

## User stories addressed

- US-9 (per-sender "do not ask again" option in dialog)

## Type

AFK
