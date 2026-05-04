# Import Account Configurations

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to import accounts from a previously exported file, with duplicate detection, so that I can restore or migrate my setup without creating duplicates.

## Blocked by
24-export-accounts

## Acceptance Criteria
- User can import accounts from a previously-exported file (FR-49).
- Duplicate detection matches by the account's unique identifier. If an account with the same identifier already exists, the import handles it gracefully (skip or update) (FR-49, N-8).
- The user can selectively choose which accounts and which data categories to import (FR-50, US-45).
- Imported accounts appear in the account list with all their settings, identities, folder mappings, rules, and contacts intact (AC-15).
- If the file is password-protected, the user is prompted for the password.
- The import flow provides clear feedback on success, skipped duplicates, and failures.

## Mapping to Epic
- US-44, US-45
- FR-49, FR-50
- AC-15 (import portion)

## HITL / AFK
AFK
