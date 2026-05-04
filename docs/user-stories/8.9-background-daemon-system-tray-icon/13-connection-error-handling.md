# Connection Error Handling and Backoff

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, when a server connection fails, I want the daemon to retry with increasing back-off intervals and reflect the error state in the tray icon, so that persistent errors do not drain system resources and I can see when something requires attention.

## Blocked by
- `1-daemon-process-lifecycle`
- `6-tray-icon-status-tooltip`

## Acceptance Criteria
- When a server connection fails, the daemon retries with increasing back-off intervals.
- The tray icon or its tooltip indicates when one or more accounts are in an error or back-off state.
- Authentication errors are surfaced via a separate, higher-priority notification (not just the tray tooltip), so that credential problems are not silently ignored.
- The daemon does not crash on transient failures (network drops, server timeouts, DNS failures).
- The daemon recovers automatically when conditions improve (server becomes reachable, credentials are updated).

## Mapping to Epic
- US-11
- FR-27, FR-28, FR-29
- NFR-3 (resilience)
- AC-10

## HITL / AFK
AFK — behavior is well-specified by the epic.

## Notes
- The "separate, higher-priority notification" for authentication errors (FR-29) bridges into the notifications epic (8.1). This story ensures the daemon emits the notification; the notification format and actions are governed by epic 8.1.
- The specific back-off strategy (linear, exponential, jittered) is an implementation detail not prescribed by the epic.
