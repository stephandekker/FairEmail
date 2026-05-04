# Curated Color Scheme Catalog

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user, I want to choose from at least six curated color schemes (primary/accent pairings), so that I can personalize the application's appearance beyond brightness alone.

## Blocked by
- `2-light-variant-chrome`
- `3-dark-variant-chrome`
- `4-true-black-variant-chrome`

## Acceptance Criteria
- At least six curated color schemes are available, each defining a primary color and an accent color (FR-1).
- Each color scheme is available in all three brightness variants: light, dark, and true-black (FR-2).
- A monochrome/neutral scheme (e.g. Black & White, Grey) is included for users who prefer no color (FR-4).
- A warm-toned scheme (e.g. Solarized) is included for users who prefer reduced blue-light exposure (FR-5).
- All schemes pass WCAG AA contrast requirements for body text in all three brightness variants (NFR-2, AC-16).
- Switching between color schemes changes the primary and accent colors used across application chrome.

## Mapping to Epic
- US-3 (partial — independent scheme selection)
- FR-1, FR-2, FR-4, FR-5
- NFR-2
- AC-16

## HITL / AFK
HITL — the specific color pairings and their names require design review to ensure aesthetic quality, brand coherence, and accessibility compliance.

## Notes
- Open Question OQ-2 asks whether the desktop version should ship with the same ~8 schemes as the source application, a subset, or an expanded set. This story requires at least six per FR-1; the exact set needs design input.
- Each scheme must be tested for WCAG AA contrast in all three variants. This is a design/QA gate, not just a code change.
