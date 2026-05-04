# Algorithmic Darkening of HTML Message Content

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user viewing a dark-themed message content area (with no force-light override active), I want the application to algorithmically darken HTML content that was designed for light backgrounds, so that text remains readable and images remain visible.

## Blocked by
- `3-dark-variant-chrome`
- `13-force-light-message-viewer`

## Acceptance Criteria
- When the application is in a dark or black variant and no content override is active, HTML email content designed for light backgrounds is rendered with algorithmically darkened colors (FR-17).
- Text remains legible after darkening (AC-6).
- Images remain visible after darkening (AC-6).
- The darkening does not corrupt the layout or break interactive elements in the email.
- When "force light" is active (globally or per-message), algorithmic darkening is not applied.

## Mapping to Epic
- US-14
- FR-17
- AC-6

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Open Question OQ-5 asks about darkening aggressiveness. Some messages contain brand colors or images that look wrong when inverted. The per-message toggle (story 15) serves as the escape hatch for messages where darkening produces poor results.
- Design Note N-3 explains the underlying tension: many HTML emails assume a light canvas.
