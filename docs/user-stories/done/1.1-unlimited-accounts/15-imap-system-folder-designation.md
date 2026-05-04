# IMAP System Folder Designation

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As an IMAP user, I want to designate which server folder serves as my Drafts, Sent, Archive, Trash, and Junk folder, so that the application files messages into the correct folders for each account.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- For each IMAP account, the user can designate a server folder for each system role: Drafts, Sent, Archive, Trash, Junk (FR-35).
- The application auto-detects system folders from server metadata (e.g. SPECIAL-USE) when available (FR-36).
- The user can override auto-detected assignments at any time (FR-36).
- POP3 accounts do not show this setting (they have fixed local folders per story 4).

## Mapping to Epic
- US-36
- FR-35, FR-36

## HITL / AFK
AFK

## Notes
- This story establishes per-account folder role mapping. Actual folder sync behavior is out of scope for this epic (NG5, covered by feature 2.x).
