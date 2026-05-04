# Test Connection Before Save

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to test the connection to my mail server before saving the account, so that I can verify my settings are correct and get clear feedback on success or failure.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- A "Test connection" action is available in the account creation/edit flow.
- Testing verifies both the incoming server (IMAP or POP3) and the outgoing server (SMTP) when applicable (FR-34).
- On success, the user sees clear positive feedback (e.g. "Connection successful").
- On failure, the user sees a clear error message indicating what went wrong (authentication failure, unreachable host, certificate error, etc.).
- Testing works for both IMAP and POP3 accounts.
- The test does not persist the account — saving is a separate action.
- If the device is offline, the test fails gracefully with an appropriate message (NFR-6).

## Mapping to Epic
- US-11
- FR-33, FR-34
- NFR-6
- AC-1 (test portion)

## HITL / AFK
AFK

## Notes
- This story assumes SMTP configuration exists alongside the account. The epic's non-goals (NG2) exclude identities, but connection testing for SMTP is explicitly in scope (FR-34). The SMTP settings needed for testing may come from a minimal outgoing-server configuration that is part of the account entity, not from identities.
