# Edit Existing Account

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to edit the settings of an existing account, so that I can update server details, credentials, or other configuration without deleting and re-creating the account.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- User can open an existing account's settings from the account list.
- All fields that were set at creation time are editable (host, port, encryption, auth method, credentials, display name).
- Changes are persisted atomically — either all changes save or none do (NFR-3).
- The account's unique identifier does not change on edit.
- Editing works while offline for non-connection-dependent fields (NFR-6).
- After saving, updated values are reflected immediately in the account list and navigation pane.

## Mapping to Epic
- Implied by FR-3, FR-5, FR-6, FR-16 (all state "shall store" / "shall be user-editable at any time")
- NFR-3, NFR-6

## HITL / AFK
AFK

## Notes
- This story covers the basic edit flow. Editing of color, avatar, category, and advanced settings are covered in their respective stories but all share this same edit surface.
