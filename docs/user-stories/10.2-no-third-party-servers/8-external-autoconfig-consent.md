# External Autoconfiguration Consent Prompt

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, if my provider is not in the bundled database and the application offers to query an external autoconfiguration service, I want to be informed before that query is made and told that my email domain will be sent to the service, so that I can choose to enter settings manually instead.

## Blocked by
- `7-bundled-provider-database-privacy` (the bundled lookup must be attempted first; this story handles the fallback)

## Acceptance Criteria
- When the bundled provider database does not contain the user's provider, and the application supports external autoconfiguration lookup, the user is presented with a consent prompt before the lookup executes.
- The prompt discloses that the user's email domain will be sent to the autoconfiguration service and identifies the service.
- The user can decline and choose manual configuration instead.
- If the user consents, the lookup proceeds; if the user declines, no external call is made.
- Manual server configuration is always available as the alternative path.

## Mapping to Epic
- US-12
- FR-16, FR-17
- AC-5

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- OQ-1 in the epic asks whether the act of using the quick-setup wizard itself constitutes sufficient consent for the autoconfiguration lookup. The conservative interpretation (FR-16) requires an explicit interstitial consent prompt. This story implements the conservative interpretation; if the decision is later revised, the consent prompt can be removed.
- The source FairEmail application queries the Thunderbird/Mozilla autoconfiguration service without a separate consent step. This story represents a deliberate tightening of the privacy posture for the desktop application.
