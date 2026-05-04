# Add POP3 Account

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As a user whose mail provider only offers POP3, I want to add a POP3 account, so that I can receive mail through that provider with appropriate POP3-specific behavior and folder structure.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- User can select POP3 as the protocol when adding a new account.
- POP3 accounts present a fixed set of local-only folders: Inbox, Drafts, Sent, Trash (FR-10).
- POP3 accounts are visually distinguished from IMAP accounts in the account list (e.g. different text color or label) (FR-11).
- The user is informed of POP3 limitations compared to IMAP: no server-side folders, no server-side search, no remote flag synchronization, local-only sent/drafts/trash (FR-10, US-35).
- Connection testing works for POP3 (reuses story 2's test-connection flow).
- The account is persisted with protocol = POP3 and the same unique identifier scheme as IMAP accounts.

## Mapping to Epic
- US-2, US-35
- FR-8, FR-9, FR-10, FR-11
- AC-2

## HITL / AFK
AFK

## Notes
- POP3-specific behavioral settings (leave on server, delete behavior, download cap) are in story 5. This story covers the creation path, folder structure, and visual distinction.
- The epic's OQ-5 notes that the optimal UX for POP3 limitations disclosure is a design decision. This story should include *some* disclosure but the exact format (inline warning, help panel, pre-setup dialog) can be decided during design.
