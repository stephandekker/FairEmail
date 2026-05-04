# Per-Account Swipe and Move Defaults

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to configure default swipe actions and a default "move-to" folder per account, so that quick-triage gestures target the right destination for each account.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- Each account has configurable swipe-left and swipe-right target folders or actions (FR-37).
- Each account has a configurable default "move-to" folder (FR-38).
- These defaults are used by the message list when performing swipe or move actions on messages belonging to this account.
- Defaults can be changed at any time via account settings.

## Mapping to Epic
- US-37
- FR-37, FR-38

## HITL / AFK
AFK

## Notes
- On a Linux desktop, "swipe" may translate to keyboard shortcuts or button actions rather than touch gestures. The epic describes the *concept* of quick-triage actions with per-account defaults; the exact interaction mechanism is a design decision.
