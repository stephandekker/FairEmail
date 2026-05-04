## Parent Feature

#4.14 Attachments

## What to build

When forwarding a message, automatically copy all non-encrypted attachments from the original message to the new draft. When replying, copy only inline images (those referenced by Content-ID in the body). If any attachments to be carried over have not yet been downloaded, prompt the user to download them before including them. Prevent sending if required attachments are incomplete.

Covers epic sections: US-13, US-14, FR-9, FR-10, FR-11, FR-33, AC-8, N-6.

## Acceptance criteria

- [ ] Forwarding a message copies all non-encrypted attachments to the new draft (AC-8).
- [ ] Replying to a message copies only inline images (those referenced by CID in the body) to the reply draft (AC-8).
- [ ] If attachments have not been downloaded, the user is prompted to download them before inclusion (FR-11).
- [ ] A clear indicator is shown for attachments not yet fully downloaded (FR-33).
- [ ] Sending is prevented until incomplete attachment downloads finish or the user removes incomplete items (FR-33).

## Blocked by

- Blocked by `2-compose-attachment-list`

## User stories addressed

- US-13
- US-14
