# Explicit Manual Save

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As any user, I want a "Save" menu item and keyboard shortcut that immediately saves my draft, regardless of whether the draft is dirty, so that I can force a save whenever I choose.

## Blocked by
1-local-draft-persistence

## Acceptance Criteria
- A "Save" action is available via a menu item and a keyboard shortcut.
- The manual save always persists the draft body, even if the dirty flag is not set. (FR-5)
- The save creates a revision snapshot if revision history is enabled (once story 6 is implemented; until then, it simply saves).
- The keyboard shortcut is discoverable and follows platform conventions.

## Mapping to Epic
- FR-5 (explicit user-initiated save, ignores dirty state)
- NFR-7 (keyboard-accessible)

## HITL / AFK
AFK — standard menu/shortcut wiring.

## Estimation
Small — one menu entry, one shortcut binding, delegates to the persistence layer.
