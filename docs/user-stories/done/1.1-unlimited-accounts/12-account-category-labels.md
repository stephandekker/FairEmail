# Account Category Labels

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As a multi-account user, I want to assign a category label to each account (e.g. "Work", "Personal"), with autocomplete from existing categories, so that I can organize my accounts into logical groups.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- Each account has an optional category label field (free-form text) (FR-17).
- When editing the category field, existing category names are offered as autocomplete suggestions (FR-23, US-18).
- Categories are implicitly created when first assigned and implicitly deleted when no account carries that label (FR-22).
- There is no separate category management screen (N-4).
- Categories are case-sensitive (N-4).
- The category label is persisted with the account and survives restarts.

## Mapping to Epic
- US-17, US-18
- FR-5 (category property), FR-17, FR-22, FR-23

## HITL / AFK
AFK

## Notes
- The epic's OQ-4 asks whether an explicit category management screen should exist. The source application does not provide one, and this story follows that design. If a management screen is added later, it would be a separate story.
