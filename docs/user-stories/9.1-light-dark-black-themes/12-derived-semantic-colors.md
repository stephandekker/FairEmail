# Derived Semantic Colors Adaptation

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user, I want all semantic UI colors (read/unread indicators, verification badges, encryption indicators, warning markers, folder colors, account-color stripes, link colors, icons) to adapt to the active theme, so that the interface is visually harmonious in every variant.

## Blocked by
- `5-color-scheme-catalog`
- `3-dark-variant-chrome`
- `4-true-black-variant-chrome`

## Acceptance Criteria
- All semantic UI colors (read/unread indicators, verification badges, encryption indicators, warning markers, folder-color accents, account-color stripes, separators) are derived from the active theme's color scheme and brightness variant (FR-19).
- Link colors in dark/black variants are adjusted to maintain legibility against dark backgrounds — not the same hue/lightness as light-mode links (FR-20, AC-13).
- Toolbar and status-bar icon colors adapt to the luminance of the toolbar background, ensuring legibility in both light and dark variants (FR-21).
- Links in the message list and content area use a color distinct from the light-mode link color when in dark themes (AC-13).

## Mapping to Epic
- US-21
- FR-19, FR-20, FR-21
- AC-13

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story covers the systematic derivation of semantic colors. Individual elements (read/unread, badges, etc.) should all flow from the same color-derivation logic rather than being hard-coded per-element.
