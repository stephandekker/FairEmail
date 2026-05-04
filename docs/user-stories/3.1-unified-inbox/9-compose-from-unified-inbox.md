# Compose New Message from Unified Inbox

## Parent Feature

#3.1 Unified Inbox

## What to build

When the user starts composing a new message from the Unified Inbox (no message context), pre-select the user-designated primary account as the "From" identity (FR-23). The draft should be stored in the primary account's Drafts folder, and the sent message in the primary account's Sent folder (US-18). The user must always be able to override the chosen account before sending (FR-25).

## Acceptance criteria

- [ ] Composing a new message from the Unified Inbox opens a draft using the primary account's identity (AC-8).
- [ ] The draft is saved in the primary account's Drafts folder (AC-8).
- [ ] The sent copy is filed in the primary account's Sent folder (AC-8).
- [ ] The user can change the "From" account/identity before sending (FR-25).

## Blocked by

- Blocked by `4-basic-unified-message-list`

## User stories addressed

- US-17 (primary account pre-selected for new compose)
- US-18 (draft and sent in chosen account's folders)
