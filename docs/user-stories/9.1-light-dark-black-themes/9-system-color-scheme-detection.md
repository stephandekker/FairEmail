# System Color-Scheme Detection

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As a day/night automatic user, I want to select an "auto" brightness mode that detects my desktop environment's light/dark preference, so that the application matches my system appearance without manual intervention.

## Blocked by
- `1-theme-preference-persistence`
- `3-dark-variant-chrome`

## Acceptance Criteria
- An "auto" / "system" brightness mode is available for selection (FR-10).
- In auto mode, the application detects the desktop environment's color-scheme preference at startup and applies the corresponding brightness variant (FR-10).
- Detection works for at least: GNOME, KDE Plasma, and any freedesktop-portal-compliant compositor (FR-14).
- If the system does not expose a color-scheme preference, auto mode defaults to the light variant (FR-13).
- No error is shown when the system does not support color-scheme reporting (AC-14).
- The user is informed (not via an error, but via UI state or tooltip) that auto-switching is unavailable on unsupported environments (NFR-5).

## Mapping to Epic
- US-5, US-8
- FR-10, FR-13, FR-14
- NFR-5
- AC-14

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Open Question OQ-4 asks about detection priority when multiple signals exist (freedesktop portal AND GNOME-specific setting). This story should document the chosen detection order. A reasonable default: prefer freedesktop portal, fall back to DE-specific settings.
- This story covers initial detection at startup. Runtime monitoring (live switching) is story 10.
