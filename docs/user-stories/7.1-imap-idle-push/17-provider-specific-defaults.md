# Provider-Specific Push/Poll Defaults

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As a user of a provider known to have unreliable or absent IDLE support, I want the application to default my account (or specific folder types) to polling without attempting IDLE first, so that initial setup is smooth and I do not experience unnecessary connection churn.

## Blocked by
- `9-per-folder-push-poll-control`
- `8-auto-optimization-push-to-poll`

## Acceptance Criteria
- For providers known to lack IDLE support (e.g. certain Yahoo, AOL configurations), the application defaults those accounts to poll mode without attempting IDLE first (US-25).
- For providers where user-created folders do not reliably support IDLE (e.g. certain implementations where only Inbox supports it), the application defaults non-Inbox folders to poll while still using IDLE on the Inbox (US-26, FR-39).
- Provider-specific defaults can be overridden by the user — the user can switch any defaulted folder to push mode (FR-39).
- Provider identification uses the existing provider database (from epic 1.7).

## Mapping to Epic
- US-25, US-26
- FR-39

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Open Question OQ-2: the canonical list of providers requiring poll defaults, and whether this list is user-extensible, is not fully documented. The initial implementation should cover the providers mentioned in the epic (Yahoo, AOL, Gmail label folders) and document the mechanism for adding more.
- Open Question OQ-6: the epic asks whether a server-behavior quirks database should handle known IDLE bugs (e.g. Outlook). This story covers provider defaults; auto-optimization (story 8) handles the generic detection path. A quirks database could be a future enhancement.
