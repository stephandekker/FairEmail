# Account Color

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to assign, change, or clear a color for each account, so that messages, folders, and notifications from that account are visually distinguishable from those of other accounts.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- User can set a color for any account via a color picker in account settings (FR-12).
- User can change or clear the color at any time (FR-12, US-16).
- The account color is displayed consistently on: message-list color stripe/badge, navigation pane folder icons, account settings list, compose window, notifications, and desktop widgets (FR-14).
- When multiple color levels exist (account, folder, identity), the most specific wins: identity > folder > account (FR-15).
- The color renders identically across all surfaces and themes (light, dark, high-contrast) (NFR-8).
- The color picker is keyboard-accessible with screen-reader labels (NFR-7).

## Mapping to Epic
- US-13, US-14, US-16
- FR-5 (color property), FR-12, FR-14, FR-15
- NFR-7, NFR-8
- AC-5

## HITL / AFK
AFK

## Notes
- The epic's OQ-3 flags that account color was a paid "Pro" feature in the source application. The decision on whether to gate this is a product decision to be resolved before implementation.
