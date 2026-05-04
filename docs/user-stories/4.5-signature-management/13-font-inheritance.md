## Parent Feature

#4.5 Signature Management

## What to build

Ensure the inserted signature inherits the user's chosen compose font setting, so that the signature and the message body are visually consistent (FR-39). When the compose font is changed, existing and newly inserted signatures should reflect the updated font.

Covers epic sections: §6.8 (US-23), §7.11 (FR-39).

## Acceptance criteria

- [ ] AC-15: The signature inherits the compose font setting and appears visually consistent with the message body
- [ ] If the user changes the compose font, signatures inserted after the change use the new font
- [ ] The font inheritance applies to both new messages and replies/forwards

## Blocked by

- Blocked by `1-basic-signature-storage-and-insertion`

## User stories addressed

- US-23
