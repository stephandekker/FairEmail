# Common-Reference Threading

## Parent Feature
#3.2 Conversation Threading

## User Story
As a power user, I want the application to use the root message of a References chain as a shared thread anchor, so that messages sharing a common ancestor are grouped together even when intermediate messages in the chain are missing from my mailbox.

## Blocked by
1-rfc-header-thread-computation

## Acceptance Criteria
- [ ] When enabled, common-reference threading uses the first entry in the `References` header (typically the root message) as the thread identifier, linking messages that share the same root ancestor (FR-17, AC-19).
- [ ] Common-reference threading is enabled by default (FR-18).
- [ ] The message's own `Message-ID` is never used as the common reference to avoid self-linking (FR-19).
- [ ] Common-reference threading is independently enableable/disableable via a user-facing setting (FR-2).
- [ ] Messages sharing a root ancestor are grouped even when intermediate messages are missing from the local store (AC-19).

## HITL / AFK
AFK — a well-defined threading strategy with clear rules.

## Notes
- OQ-7 in the epic asks whether the default should be enabled (non-app-store) or disabled (app-store). This story follows the epic's FR-18 which states "enabled by default." If the decision changes, update this story.
