# Connection State and Diagnostics

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to see each account's current connection state and be able to view its connection log, so that I can diagnose connectivity problems.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- Each account displays its current connection state via a status indicator in the account list: connected, connecting, disconnected, closing, backing off (FR-44, AC-18).
- The status updates in near-real-time (AC-18).
- When an account is in an error state, the error detail is displayed in the account settings view (FR-45).
- The user can view a per-account connection log for diagnostic purposes (FR-46, US-42).
- A connection failure on one account does not prevent other accounts from functioning (NFR-2, AC-16).

## Mapping to Epic
- US-41, US-42
- FR-44, FR-45, FR-46
- NFR-2
- AC-16, AC-18

## HITL / AFK
AFK
