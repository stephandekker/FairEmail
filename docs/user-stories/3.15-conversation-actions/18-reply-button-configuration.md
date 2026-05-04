# Reply Button Configuration

## Parent Feature
#3.15 Conversation Actions

## User Story
As a power user, I want to configure what the primary reply button does on single-click and long-press, and have its icon reflect the configured action, so that my most common actions are always one or two clicks away.

## Blocked by
`1-action-menu-infrastructure`, `2-basic-reply`, `4-reply-all`, `10-basic-forward`

## Acceptance Criteria
- The primary reply button supports two configurable trigger gestures: single-click and long-press, each independently mappable to any conversation action or to "show menu" (FR-3, US-32, US-33).
- Configuring single-click to "forward" causes a single click to open a forward compose (AC-16).
- Configuring single-click to "show menu" opens the action menu on click.
- The reply button icon changes to reflect the configured single-click action (reply icon, reply-all icon, forward icon, etc.) (FR-4, US-34, AC-16).
- The configuration persists across sessions.
- The button is keyboard-accessible and has a descriptive label for screen readers (NFR-6).

## Mapping to Epic
- US-32, US-33, US-34
- FR-3, FR-4
- NFR-6
- AC-16
- Design Note N-1

## HITL / AFK
HITL — the interaction model (single-click vs. long-press vs. keyboard shortcut) is a UX-sensitive decision on desktop, where long-press is less conventional than on mobile. A design review is recommended.

## Notes
- N-1 explains the two-gesture approach: it avoids the trade-off between "reply is always one click" and "I need a menu." Users who always reply can set single-click to reply; users who vary can set single-click to open the menu.
- On a Linux desktop, "long-press" may need to be adapted to a right-click, secondary button, or modifier-click convention. This is a platform adaptation question that should be resolved during design.
