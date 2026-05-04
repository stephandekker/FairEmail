# Account Avatar

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to assign, change, or clear an avatar image for each account, so that the account is visually identifiable in the account list.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- User can set an avatar image for any account via an image picker in account settings (FR-13).
- User can change or clear the avatar at any time (US-16).
- The avatar is stored as a persistent reference (FR-5).
- The avatar is displayed alongside the account name in the account list (US-15, AC-6).
- The image picker is keyboard-accessible with screen-reader labels (NFR-7).

## Mapping to Epic
- US-15, US-16
- FR-5 (avatar property), FR-13
- NFR-7
- AC-6

## HITL / AFK
AFK

## Notes
- The epic's OQ-2 flags an open question about how account avatars interact with per-message contact photos (Gravatar, BIMI, identicons). This story covers the account-level avatar only; the interaction with contact photos should be clarified during design and may be addressed in a separate story or epic.
- OQ-3 also applies here — avatar was a Pro feature in the source app.
