# Quota Display

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to see my mailbox quota usage (used / limit) for each account when the server reports it, with a visual warning when usage is high, so that I can manage my storage proactively.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- When the server reports quota information (e.g. via IMAP QUOTA extension), the account settings view displays current usage and limit (FR-42, AC-17).
- When quota usage exceeds a high threshold (e.g. 95%), a visual warning is displayed (FR-43, AC-17).
- When the server does not report quota, no quota information is shown (no error, no placeholder).
- Quota information updates when the account syncs.

## Mapping to Epic
- US-40
- FR-42, FR-43
- AC-17

## HITL / AFK
AFK
