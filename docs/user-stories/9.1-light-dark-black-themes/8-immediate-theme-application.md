# Immediate Theme Application Without Restart

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user, I want theme changes to take effect immediately upon selection without requiring a manual application restart, preserving my navigation state, so that trying different themes is frictionless.

## Blocked by
- `7-theme-selection-ui`

## Acceptance Criteria
- Theme changes take effect immediately upon confirmation; no manual restart is required (FR-25).
- If the interface must reload to apply the theme, this happens automatically (FR-26).
- After a theme change, the user's navigation state (current view, scroll position) is preserved as closely as possible (FR-26).
- Theme switching completes within two seconds with no visible rendering artifacts or half-applied states (NFR-1).
- Theme changes do not break keyboard focus or tab order (NFR-6).

## Mapping to Epic
- US-2
- FR-25, FR-26
- NFR-1, NFR-6
- AC-1, AC-15

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Design Note N-4 explains that the source application achieves this by tearing down and recreating the current screen. The desktop version may use a different mechanism (live style re-application, CSS variable swap, etc.) — the requirement is instantaneity and state preservation.
