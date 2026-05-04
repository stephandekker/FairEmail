# Auto-Save Settings

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As a power user, I want to configure which auto-save triggers are active, whether revision history is kept, and whether drafts sync to the server, so that the feature adapts to my editing style.

## Blocked by
2-paragraph-break-trigger, 3-punctuation-trigger, 6-revision-history-storage, 9-server-draft-sync

## Acceptance Criteria
- The following four settings are available in the send preferences and are independently configurable: (FR-25, AC-16)

  | Setting | Default |
  |---|---|
  | Auto-save on paragraph break | Enabled |
  | Auto-save on punctuation | Disabled |
  | Save revision history | Enabled |
  | Save drafts to server | Enabled |

- All four settings persist across application restarts. (AC-16)
- Changes to these settings take effect immediately for all open and future compose sessions. (FR-26)
- Disabling "auto-save on paragraph break" prevents newlines from triggering saves, but loss-of-focus saves still occur. (AC-12)
- Disabling "save revision history" causes each save to overwrite the previous snapshot; undo/redo controls are hidden. (AC-9)

## Mapping to Epic
- FR-25, FR-26
- US-3, US-6, US-14
- AC-9, AC-12, AC-16

## HITL / AFK
AFK — standard settings UI with four toggles.

## Estimation
Small-to-medium — four toggle controls in a settings panel, wired to the existing trigger and storage logic.
