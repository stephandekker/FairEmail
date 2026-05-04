# Reply / Forward from Unified Inbox

## Parent Feature

#3.1 Unified Inbox

## What to build

When the user replies to, replies-all-to, or forwards a message from within the Unified Inbox, pre-select the account that received the original message as the "From" identity (FR-24). If the receiving account is ambiguous, fall back to the primary account. The user must always be able to override the chosen account before sending (FR-25). Draft and sent copies are stored in the selected account's respective folders.

## Acceptance criteria

- [ ] Replying to a message in the Unified Inbox defaults to the receiving account's identity (AC-9).
- [ ] Forwarding a message defaults to the same receiving account.
- [ ] If the receiving account is ambiguous, the primary account is used as fallback.
- [ ] The user can override the "From" identity before sending (FR-25).
- [ ] Draft is stored in the selected account's Drafts; sent copy in its Sent folder.

## Blocked by

- Blocked by `9-compose-from-unified-inbox`

## User stories addressed

- US-19 (reply defaults to receiving account)
