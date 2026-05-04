# Grace Period Before Daemon Shutdown

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want the daemon to wait briefly before stopping when the last account becomes ineligible, so that rapid configuration changes (toggling accounts on and off) do not cause disruptive start/stop cycling.

## Blocked by
- `1-daemon-process-lifecycle`
- `7-account-level-monitoring-conditions`

## Acceptance Criteria
- When the daemon has no enabled accounts to monitor and no pending operations, it waits a brief grace period (on the order of seconds) before stopping.
- If a new account becomes eligible or a new pending operation is enqueued during the grace period, the daemon cancels the shutdown and continues running.
- After the grace period expires with no reason to stay alive, the daemon stops and the tray icon disappears or switches to inactive.
- The grace period prevents rapid stop/start cycling during account configuration changes.

## Mapping to Epic
- FR-4
- N-4 (grace period rationale — source app uses ~10 seconds)
- AC-14 (last account removed triggers shutdown after grace period)

## HITL / AFK
AFK — behavior and rationale are clearly specified in the epic (N-4).

## Notes
- The source application uses approximately 10 seconds. The exact duration is an implementation detail but should be in that range.
