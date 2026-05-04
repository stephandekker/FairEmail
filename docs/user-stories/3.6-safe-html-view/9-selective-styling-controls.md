# Selective Styling Controls (User Preferences)

## Parent Feature
#3.6 Safe HTML View

## Blocked by
5-css-property-allowlist, 7-embedded-style-block-processing

## Description
Implement independent on/off user preferences for each of the following styling categories: text colors, background colors, font sizes, font families, and text alignment. When a category is disabled, the corresponding CSS properties are stripped from the sanitized output regardless of whether they would otherwise pass the allowlist. When all categories are disabled, every message renders with the application's default font, size, color, and alignment.

## Motivation
Users have different tolerance for sender-controlled styling. A privacy maximalist wants uniform appearance; a visual fidelity user wants everything preserved. These toggles give fine-grained control over the trade-off between fidelity and uniformity.

## Acceptance Criteria
- [ ] A user preference exists for each of: text colors (default: on), background colors (default: off), font sizes (default: on), font families (default: on), text alignment (default: on).
- [ ] When text colors are disabled, `color` properties are stripped from sanitized output.
- [ ] When background colors are disabled, `background-color` and `background` properties are stripped.
- [ ] When font sizes are disabled, `font-size` properties are stripped.
- [ ] When font families are disabled, `font-family` properties are stripped.
- [ ] When text alignment is disabled, `text-align` properties are stripped.
- [ ] When all optional styling categories are disabled, messages render with the application's default typography — uniform across all messages.
- [ ] When all optional styling categories are enabled, the sanitized view preserves as much of the sender's visual design as the allowlist permits.
- [ ] Preferences persist across sessions.

## HITL/AFK Classification
AFK — testable by toggling preferences and verifying sanitized output changes accordingly.

## Notes
- FR-17 governs the default values for each toggle.
- Design Note N-3 explains why background colors default to off.
- Properties that are "always allowed" (font-weight, font-style, text-decoration, text-transform, etc.) are not affected by these toggles.
