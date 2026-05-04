## Parent Feature

#4.14 Attachments

## What to build

Ensure that when a message is signed and/or encrypted (PGP or S/MIME), all attachments are included within the signed/encrypted container — not sent separately. Encryption-related attachments (signature files, encrypted containers) are tracked internally and do not appear as user-visible regular attachments.

Covers epic sections: US-29, US-30, FR-34, FR-35, AC-16.

## Acceptance criteria

- [ ] Encrypting a message with PGP that has two attachments produces a single encrypted container; decrypting it reveals both attachments intact (AC-16).
- [ ] Signing a message includes all attachments in the cryptographic signature (US-30).
- [ ] Encryption-related files (signature files, encrypted containers) do not appear in the user-visible attachment list (FR-35).

## Blocked by

- Blocked by `1-basic-file-attachment`

## User stories addressed

- US-29
- US-30
