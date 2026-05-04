# Enable / Disable Synchronization

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to enable or disable synchronization for each account independently, so that I can temporarily or permanently stop syncing an account without deleting it.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- Each account has a synchronization enabled/disabled toggle (FR-6).
- Disabling synchronization stops all syncing for that account (AC-11).
- Disabling synchronization clears any active error or warning state for that account (FR-32, AC-11).
- Disabling synchronization on the primary account revokes its primary designation (FR-32, AC-11).
- Toggling synchronization does not affect the account's notification settings — they survive the toggle (AC-19).
- The sync state is visible in the account list.

## Mapping to Epic
- US-26
- FR-6 (sync enabled flag), FR-32
- AC-11, AC-19 (notification independence)

## HITL / AFK
AFK
