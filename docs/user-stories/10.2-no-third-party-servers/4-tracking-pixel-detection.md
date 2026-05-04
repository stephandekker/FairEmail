# Tracking Pixel Detection and Flagging

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As a privacy-conscious user, I want the application to detect and flag likely tracking pixels (very small or invisible images) separately from regular remote images, so that I can make an informed decision about what to load.

## Blocked by
- `2-remote-content-blocking` (tracking pixels are a subset of remote content; blocking must exist first)

## Acceptance Criteria
- The application detects likely tracking pixels using a configurable size threshold (e.g. images below N×N pixels) and/or known tracker patterns from a bundled blocklist.
- Tracking pixels are reported to the user as distinct from regular remote images (e.g. a separate count or label in the blocked-content indicator).
- The detection uses bundled data only — no network call to any external service is required to identify tracking pixels.
- The user can still choose to load tracking pixels if they wish, but the distinction is clear.

## Mapping to Epic
- US-7
- FR-8

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- The bundled tracker blocklist referenced here overlaps with the blocklist update mechanism in story 13. The initial implementation should ship a static bundled list; optional updates are covered separately.
- The "configurable size threshold" is mentioned in FR-8. Whether this is a user-facing setting or a developer-configured default is not specified in the epic; default to a sensible built-in threshold with optional user override.
