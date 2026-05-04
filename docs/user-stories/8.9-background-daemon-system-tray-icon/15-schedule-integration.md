# Schedule Integration

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user who has configured a synchronization schedule, I want the daemon to respect that schedule — pausing synchronization outside the configured window and resuming when the window opens — and I want the tray icon to reflect the scheduled pause.

## Blocked by
- `1-daemon-process-lifecycle`
- `2-system-tray-icon-presence`

## Acceptance Criteria
- When a synchronization schedule is configured and the current time is outside the scheduled window, the daemon pauses synchronization.
- When the scheduled window opens, the daemon resumes synchronization automatically.
- During a schedule-imposed pause, the tray icon indicates the paused/scheduled state so the user understands why synchronization is not active.
- When the scheduled window is active, the tray icon returns to its normal monitoring state.

## Mapping to Epic
- US-18
- FR-33, FR-34
- AC-16

## HITL / AFK
AFK — behavior is well-specified by the epic.

## Notes
- The synchronization schedule itself is defined by epic 7.3 (NG5). This story covers only the daemon's obedience to an externally configured schedule and the tray icon's reflection of schedule-imposed pauses. It does not define the schedule configuration UI or rules.
