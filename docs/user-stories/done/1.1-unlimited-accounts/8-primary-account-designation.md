# Primary Account Designation

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to designate one account as the primary account, so that the application uses it by default for new compositions and other situations where no specific account is implied.

## Blocked by
1-create-imap-account, 9-enable-disable-sync

## Acceptance Criteria
- User can set any synchronized account as primary (FR-24, FR-25).
- Only one account can be primary at a time. Setting a new primary automatically demotes the previous one (FR-26, AC-4).
- The primary account is visually indicated in the account list (e.g. star icon) (FR-27, AC-4).
- When the first account is added and no primary exists, it is designated primary by default (FR-28).
- Only accounts with synchronization enabled are eligible for primary designation (FR-25, US-24).
- If synchronization is disabled on the primary account, its primary designation is automatically revoked (FR-32).

## Mapping to Epic
- US-22, US-23, US-24, US-25
- FR-24, FR-25, FR-26, FR-27, FR-28, FR-32
- AC-4

## HITL / AFK
AFK

## Notes
- This story depends on story 9 (enable/disable sync) because primary eligibility is gated on synchronization status. However, the two can be developed in parallel if the sync-enabled flag is available on the account entity from story 1 — the dependency is logical, not strictly sequential.
