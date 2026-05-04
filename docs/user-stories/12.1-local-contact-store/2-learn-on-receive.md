## Parent Feature

#12.1 Local Contact Store

## What to build

Extend automatic contact learning to cover incoming messages. When the application processes a newly received message, the sender's address (preferring Reply-To if present, otherwise From) is recorded as a received-from contact in the context of the receiving account.

The same create-or-update logic from story 1 applies: new contacts are initialised with times-contacted = 1 and matching timestamps; existing contacts have their counter incremented and last-contacted updated (FR-5, FR-6, FR-7, FR-8).

Self-sent messages — where the sender matches one of the account's own identities — must be handled correctly: recipients are recorded as sent-to contacts but the sender is not recorded as a received-from contact (FR-11).

The same exclusions apply: skip non-personal addresses (FR-10) and skip messages in Drafts, Trash, or Spam folders (FR-9).

## Acceptance criteria

- [ ] After receiving a message from a new address, a received-from contact appears with times-contacted = 1 (AC-2)
- [ ] Receiving multiple messages from the same sender increments times-contacted and updates last-contacted
- [ ] Reply-To is preferred over From when present (FR-5)
- [ ] Self-sent messages do not create a received-from contact for the sender (FR-11)
- [ ] Non-personal addresses are skipped (AC-10)
- [ ] Messages in Drafts, Trash, or Spam do not create or update contacts (AC-11)
- [ ] Contacts are scoped to the receiving account

## Blocked by

- Blocked by 1-contact-record-and-learn-on-send (depends on contact storage and create-or-update logic)

## User stories addressed

- US-2 (auto-record sender on receive)
- US-3 (contacts scoped to account)
- US-4 (update existing, no duplicates)
- US-5 (skip non-personal)
- US-6 (skip system folders)
