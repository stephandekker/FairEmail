# URL Tracking Parameter Stripping

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, I want the option to have known tracking parameters stripped from URLs before they are opened, so that the destination site receives less information about how I arrived.

## Blocked by
- `5-link-confirmation-dialog` (the link handling flow must exist before parameter stripping can be added to it)

## Acceptance Criteria
- An optional setting exists to strip known tracking parameters from URLs before opening them.
- The setting defaults to disabled (or enabled — the epic says "offer an option" but does not specify the default; see Notes).
- The stripping uses a bundled parameter list and does not require contacting an external server.
- When enabled, the confirmation dialog shows the cleaned URL.
- The bundled parameter list covers common tracking parameters (e.g. utm_source, utm_medium, fbclid, gclid, etc.).

## Mapping to Epic
- US-10
- FR-13

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- The epic says "offer an optional setting" but does not explicitly state whether it defaults to enabled or disabled. Given the overall opt-in philosophy of the epic, defaulting to disabled is the conservative choice — but stripping tracking parameters does not contact a third-party server, so an argument could be made for enabling it by default. This should be confirmed during design.
- The bundled parameter list overlaps with the blocklist update mechanism in story 13. Initially, a static bundled list is sufficient.
