# WYSIWYG Editor Surface

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a casual composer, I want the compose window to provide a rich text editing surface that renders formatting in real time (bold appears bold, colored text appears colored, etc.), so that I can see exactly how my message will look while I write it.

## Blocked by
*(none — this is the foundational slice)*

## Acceptance Criteria
- The compose window defaults to a rich text editing surface for new compositions, replies, and forwards.
- The editing surface renders formatting in real time (WYSIWYG): bold text displays as bold, colored text displays in color, lists display with bullets/numbers, etc.
- The editing surface supports free-form text entry, text selection (mouse and keyboard), and cursor navigation.
- The surface is responsive — text entry and cursor movement show no perceptible lag on messages up to 10,000 words with mixed formatting.
- The editing surface does not crash or corrupt content on edge cases: empty messages, very large messages, or complex formatting.
- All controls are keyboard-accessible with screen-reader labels.

## Mapping to Epic
- FR-1, FR-2, FR-3
- NFR-1, NFR-2, NFR-5, NFR-8

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story establishes the minimal editing surface only. The formatting toolbar, individual formatting actions, and HTML serialization are separate stories.
- The surface must be designed to host formatting actions that will be added in subsequent stories, but this story only requires that the surface can *render* formatting, not that the user can *apply* it yet.
