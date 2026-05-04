# Folder Role Assignment After Successful Test

## Parent Feature

#1.4 Manual Server Configuration

## What to build

After a successful IMAP inbound test, display dropdown selectors for each well-known folder role: Drafts, Sent, Archive, Trash, and Spam. Each selector is pre-populated with the folder that best matches that role based on server-advertised folder attributes and naming conventions. If no match is found for a role, the selector defaults to "not set". The user can override any auto-detected assignment. Folder role assignments are persisted when the account is saved.

The Save button and folder role selectors should be made visible only after a successful test (per design note N-2).

Covers epic sections: FR-38, FR-39, FR-40.

## Acceptance criteria

- [ ] After a successful IMAP test, dropdown selectors appear for Drafts, Sent, Archive, Trash, and Spam roles
- [ ] Each selector is pre-populated with the best-matching folder from the server's folder list
- [ ] If no match is found for a role, the selector defaults to "not set"
- [ ] The user can override any auto-detected assignment by selecting a different folder
- [ ] The Save button is visible only after a successful test
- [ ] Folder role assignments are persisted on save

## Blocked by

- Blocked by `5-inbound-test-connection`

## User stories addressed

- US-19 (folder role selectors pre-populated with auto-detected folders)
- US-20 (no-match defaults to "not set")
