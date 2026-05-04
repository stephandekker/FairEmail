# Chip Rendering for Selected Recipients

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, I want each selected recipient in a recipient field to be rendered as a compact visual chip showing the contact's name (or email) and avatar, so that I can see at a glance who I am addressing and easily remove recipients.

## Blocked by
- `2-basic-autocomplete-from-sent-contacts`

## Acceptance Criteria
- When chip display is enabled (the default), each recipient in a recipient field is rendered as a compact visual chip.
- Each chip displays the contact's display name or email address (if no name is known).
- Each chip displays an avatar (circular or rounded, following the application's global avatar-shape setting) if one is available.
- Chips are deletable by pressing backspace when the cursor is immediately after the chip, or by a direct removal gesture (e.g. close button) on the chip.
- Chip text direction follows the directionality of the contact's name (LTR or RTL).
- The chip display mode is toggleable by the user (chips on/off via a setting); when off, recipients are displayed as plain comma-separated text.
- Chips have appropriate screen-reader labels.
- Selecting a suggestion from the autocomplete dropdown inserts the recipient and renders it as a chip (when chips are enabled).

## HITL/AFK Classification
**HITL** — chip visual design, avatar sizing, RTL rendering, and the toggle UX should be reviewed by a human.

## Notes
- The encryption-status indicator on chips is delivered in story 5; this story covers the base chip without encryption info.
- FR-31 (chip text direction) requires bidi-aware rendering. This should be tested with Arabic and Hebrew display names.
