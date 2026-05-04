# Revision Indicator and Dirty Marker

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As a careful composer, I want to see which revision number I am currently viewing and whether my latest changes have been saved, so that I know where I am in the history and whether my work is captured.

## Blocked by
6-revision-history-storage

## Acceptance Criteria
- The compose window displays the current revision number in a visible location (e.g. title bar or status area). (FR-19, AC-8)
- When the draft body has been modified since the last save (dirty state), the revision indicator shows a distinguishing mark (e.g. an asterisk) alongside the revision number. (FR-20, AC-8)
- Navigating via undo or redo immediately updates the displayed revision number. (FR-17)
- The indicator is visible without requiring user interaction (no hover or click to reveal).

## Mapping to Epic
- FR-19, FR-20
- US-16, US-17
- AC-8

## HITL / AFK
AFK — small UI element with straightforward data binding.

## Estimation
Small — one text label bound to revision number and dirty flag.
