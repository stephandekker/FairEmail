# Splash Screen Theme Consistency

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user, I want the splash or loading screen shown at application startup to respect my last-known brightness variant, so that dark-mode users are not greeted by a white flash and light-mode users are not greeted by a dark flash.

## Blocked by
- `1-theme-preference-persistence`
- `3-dark-variant-chrome`

## Acceptance Criteria
- The splash or loading screen uses the user's last-known brightness variant (FR-30).
- Launching the application after selecting a dark or black theme does not show a light-colored splash screen (AC-8).
- Launching the application after selecting a light theme does not show a dark splash screen.
- On first launch (no stored preference), the splash uses the default variant (light).

## Mapping to Epic
- FR-30
- AC-8

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Design Note N-7 emphasizes this is a small but meaningful polish point. Implementation requires reading the stored brightness variant before rendering the splash — this is why story 1 (persistence) must store preferences in a location accessible very early in startup.
