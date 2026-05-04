# Delete Account

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to delete an account permanently, with a confirmation prompt, so that I can remove accounts I no longer use — understanding that all associated data will be removed.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- User can initiate deletion from the account list or account settings.
- A confirmation dialog names the account and warns that all associated data will be removed (FR-29, AC-9).
- Upon confirmation, the account and all associated data are deleted: folders, messages, identities, pending operations, rules, and contacts (FR-30, AC-9).
- The account's notification channel is removed (FR-41).
- If the deleted account was primary, the primary designation is cleared (no automatic reassignment — the user must choose a new primary).
- Deletion is permanent — there is no undo or archive state (N-7).
- The account list updates immediately after deletion.

## Mapping to Epic
- US-4
- FR-29, FR-30, FR-41
- AC-9

## HITL / AFK
AFK

## Notes
- The epic does not specify whether primary is automatically reassigned to another account after deleting the primary. The source application appears to simply clear primary status. If this creates a "no primary" state, the user must set a new one manually.
