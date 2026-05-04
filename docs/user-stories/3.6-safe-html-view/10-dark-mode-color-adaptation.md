# Dark-Mode Color Adaptation

## Parent Feature
#3.6 Safe HTML View

## Blocked by
5-css-property-allowlist, 9-selective-styling-controls

## Description
Extend the sanitization pipeline to adapt text and background colors for the active application theme. In dark mode, evaluate luminance of each color and adjust for adequate contrast against the dark background. When a text color is present without a background color (or vice versa), infer the missing color to avoid contrast failures. In high-contrast/monochrome themes, suppress message background colors. Provide a per-message "force light background" toggle.

## Motivation
Many emails are designed for light backgrounds. Without color adaptation, dark-mode users would frequently see dark text on dark backgrounds or other unreadable combinations. The pipeline must proactively fix contrast issues while the force-light toggle provides an escape hatch for color-dependent content.

## Acceptance Criteria
- [ ] In dark mode, text colors are adjusted to be readable against the dark background.
- [ ] In dark mode, background colors are adjusted to maintain adequate contrast with adjusted text colors.
- [ ] When only a text color is specified (no background), the pipeline infers the background and adjusts the text color if needed.
- [ ] When only a background color is specified (no text color), the pipeline infers the text color and adjusts if needed.
- [ ] In high-contrast or monochrome themes, message background colors are suppressed.
- [ ] A per-message "force light background" toggle is available in the message UI.
- [ ] When force-light is enabled in dark mode, the message renders against a white background with original (light-theme) color adjustments.
- [ ] Color adaptations never produce unreadable contrast ratios.
- [ ] The same message with the same preferences produces the same color output deterministically.

## HITL/AFK Classification
HITL — color adaptation quality benefits from visual review to ensure readability across a range of real-world emails. Automated contrast-ratio checks can cover the basics, but visual spot-checking is valuable.

## Notes
- FR-21 through FR-24 govern this story.
- OQ-8 flags potential conflict between algorithmic darkening and the pipeline's own color adjustments — this should be tested during implementation.
- NFR-7 (theme consistency) is the governing non-functional requirement.
