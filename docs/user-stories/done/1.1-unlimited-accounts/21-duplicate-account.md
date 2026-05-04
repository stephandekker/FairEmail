# Duplicate Account

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to duplicate an existing account's configuration into a new account, so that I can set up a similarly-configured account without re-entering all settings.

## Blocked by
1-create-imap-account, 3-edit-existing-account

## Acceptance Criteria
- User can initiate duplication from the account list or account settings.
- The duplicate opens in the account editor, pre-filled with the source account's settings (FR-31, AC-10).
- The user can modify any field before saving (FR-31).
- Saving creates a new, independent account with its own unique identifier (AC-10).
- The duplicate does not inherit the source's primary designation.
- The duplicate does not share any mutable state (messages, folders, sync state) with the source.

## Mapping to Epic
- US-5
- FR-31
- AC-10

## HITL / AFK
AFK
