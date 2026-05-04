# Metered Network Awareness

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, I want the application to distinguish between metered and unmetered network connections and allow me to configure whether push is active on metered connections, so that I can avoid unexpected data usage.

## Blocked by
- `11-network-change-handling`

## Acceptance Criteria
- The application distinguishes between metered and unmetered network connections (FR-34).
- The user can configure per account whether push (or polling) is permitted on metered connections (FR-34).
- On a metered network with "no push on metered" configured, IDLE sessions are not established (AC-10).
- Switching from a metered to an unmetered network causes push to activate automatically (AC-10).
- Switching from unmetered to metered causes IDLE sessions to be torn down (if configured to do so) and replaced with polling or no sync, per the user's configuration.
- When the active network's metered status changes, the application re-evaluates all active connections (FR-35).

## Mapping to Epic
- US-19, US-20
- FR-34, FR-35
- AC-10

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- On Linux desktop, metered status detection depends on the network management layer (e.g. NetworkManager exposes a metered property per connection). The specific detection mechanism is an implementation choice.
- The per-account configuration UI is part of story 16 (global/per-account settings). This story covers the underlying logic that respects the setting.
