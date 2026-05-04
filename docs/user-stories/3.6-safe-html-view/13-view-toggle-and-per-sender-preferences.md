# View Toggle and Per-Sender/Domain Preferences

## Parent Feature
#3.6 Safe HTML View

## Blocked by
2-core-sanitization-pipeline

## Description
Provide a clearly labeled toggle control on each message to switch between the safe reformatted view and the original HTML view (and back). When switching to the original view, show a confirmation prompt (dismissible, with "remember for this sender/domain" option). Implement persistent per-sender and per-domain preferences for "always show original" and "always show images" that are applied automatically on future messages.

## Motivation
The safe view is the default but not always sufficient — complex newsletters or trusted senders may need the original rendering. The toggle with confirmation ensures users don't accidentally expose themselves, while per-sender memory eliminates repeated toggling for trusted senders.

## Acceptance Criteria
- [ ] Each message displays a clearly labeled toggle button to switch between reformatted and original views.
- [ ] Switching to the original view triggers a confirmation prompt (when the confirmation preference is enabled, which is the default).
- [ ] The confirmation prompt offers a "remember for this sender" and "remember for this domain" option.
- [ ] When the user confirms and chooses to remember, the preference is stored persistently.
- [ ] Future messages from a sender/domain with "always show original" preference open directly in the original view without prompting.
- [ ] Per-sender/domain "always show images" preference causes images to load automatically for that sender/domain.
- [ ] Switching back from original to reformatted view does not require confirmation.
- [ ] The toggle is discoverable (not hidden in a menu; visible in the message chrome).
- [ ] Per-sender memory is opt-in only — the application never automatically learns preferences from behavior (Design Note N-5).

## HITL/AFK Classification
HITL — UI placement and confirmation flow benefit from UX review to ensure discoverability and clarity.

## Notes
- FR-45 through FR-47 govern this story.
- Design Note N-5 explains why per-sender memory is explicit opt-in, not behavioral.
- The original view itself is feature 3.7 (NG3) — this story only provides the toggle mechanism, not the original-view rendering implementation.
