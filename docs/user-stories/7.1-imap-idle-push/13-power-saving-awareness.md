# System Power-Saving Mode Awareness

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As a battery-conscious laptop user, I want the application to respect system power-saving modes by allowing IDLE connections to lapse gracefully when power saving is active, and resuming push when normal power returns, so that background email checking does not drain my battery unnecessarily.

## Blocked by
- `11-network-change-handling`

## Acceptance Criteria
- The application monitors system power-saving state (e.g. laptop on battery with power saver active, system suspend) (FR-33).
- When the system enters power-saving mode, IDLE connections lapse gracefully rather than being aggressively maintained (FR-33, AC-16).
- Exiting power-saving mode causes reconnection and push resumption within one keep-alive cycle (AC-16).
- The user can exempt specific accounts from power-saving restrictions, allowing push to remain active even in power-saving mode for high-priority accounts (FR-36).
- System suspend/hibernate causes IDLE connections to die gracefully; on wake, the application reconnects and performs a full folder check (Design Note, OQ-4).

## Mapping to Epic
- US-18
- FR-33, FR-36
- AC-16

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Open Question OQ-4: should the application schedule a system wake-up timer during suspend to perform periodic checks, or is "check on wake" sufficient? The epic does not resolve this. For the initial implementation, "check on wake" is likely sufficient — document this decision and revisit if users report missed mail during long suspend periods.
- On Linux desktop, power-saving detection can use UPower D-Bus signals, systemd inhibitors, or logind suspend/resume signals. The specific mechanism is an implementation choice.
- The per-account exemption setting UI is part of story 16.
