# Auto-Restart on Unexpected Termination

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, if the daemon process is terminated unexpectedly (e.g. by the OS reclaiming memory, by a crash, or by a signal), I want it to restart automatically and resume monitoring without my intervention.

## Blocked by
- `1-daemon-process-lifecycle`
- `10-autostart-on-login`

## Acceptance Criteria
- If the daemon process is killed externally (e.g. via SIGKILL), it is restarted by the session manager or equivalent mechanism.
- After an unexpected restart, the daemon returns to full operational state without requiring user interaction.
- The tray icon reappears after restart.
- No stale lock files or orphan state prevent the restart.
- The restart mechanism does not enter a rapid restart loop if the daemon crashes repeatedly (some form of back-off or crash limit).

## Mapping to Epic
- US-6
- FR-5, FR-6
- NFR-3 (resilience), NFR-4 (session integration)
- AC-12

## HITL / AFK
AFK — standard service supervision behavior.

## Notes
- N-5 (sticky restart semantics) documents the source app's approach. The desktop equivalent is "restart on failure" in a service unit or equivalent supervision.
- OQ-6 (graceful shutdown on session end) is tangentially relevant. On normal session end (logout/shutdown), the daemon should terminate cleanly rather than being restarted. The restart mechanism should distinguish between "killed unexpectedly during a live session" and "terminated because the session is ending."
- OQ-4 (multiple-instance prevention) is relevant here. If the restart mechanism launches a new instance while a previous one is still shutting down, there should be single-instance enforcement to prevent conflicts.
