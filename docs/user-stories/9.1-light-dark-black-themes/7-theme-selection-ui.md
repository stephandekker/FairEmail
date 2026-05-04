# Theme Selection Interface

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user, I want to open a theme-selection interface from the application's settings or main menu, so that I can browse and choose my preferred color scheme, brightness variant, and related options.

## Blocked by
- `5-color-scheme-catalog`
- `6-color-scheme-reversal`

## Acceptance Criteria
- A theme-selection interface is accessible from the main menu or settings (FR-23).
- The interface allows independent selection of: (a) color scheme, (b) brightness variant (light/dark/black/auto), (c) color reversal option, (d) auto-mode dark variant preference (dark vs. black), (e) content-area overrides (force light for messages, force light for composer), (f) beige preference (FR-24).
- The interface is fully navigable via keyboard (NFR-6).
- Theme names, variant labels, and toggle states are announced to screen readers (NFR-7).
- Active theme state is programmatically determinable for assistive technology (NFR-7).

## Mapping to Epic
- US-1, US-3
- FR-23, FR-24
- NFR-6, NFR-7
- AC-1 (partial — interface exists)

## HITL / AFK
HITL — the layout, grouping, and visual presentation of theme options requires UX design review.

## Notes
- This story covers the UI for selecting themes. The immediate-application behavior (no restart needed) is covered in story 8.
- Whether the interface shows live previews or requires a "confirm" action is an implementation/design choice; the epic allows either (US-2: "preview or immediate application").
