# Connection Keep-Alive Mechanism

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, I want the application to automatically keep IDLE connections alive by sending periodic NOOP signals, so that firewalls and NATs do not silently drop my connections and I do not miss new mail.

## Blocked by
- `2-single-folder-idle-session`

## Acceptance Criteria
- For each active IDLE connection, the application sends a keep-alive signal (NOOP or equivalent) at a configurable interval (FR-14).
- The default keep-alive interval is 15 minutes (FR-15).
- The minimum keep-alive interval is no less than 9 minutes (FR-16).
- Keep-alive timing is driven by a system-level scheduling mechanism (alarm or timer), not an in-process sleep loop (FR-18, Design Note N-2).
- The application verifies the connection is alive by checking for a server response to the keep-alive signal; a missing response is treated as a connection failure (FR-17).
- When a keep-alive signal fails or times out, the connection is flagged for reconnection (feeds into story 7).
- Poll-only folders on push-capable accounts are checked opportunistically during each keep-alive cycle (FR-13).

## Mapping to Epic
- US-7, US-8
- FR-13, FR-14, FR-15, FR-16, FR-17, FR-18
- AC-7 (prerequisite)

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story establishes the keep-alive mechanism with a fixed default interval. Automatic interval tuning is story 6. User-configurable interval override is story 16.
- The existing codebase uses `AlarmManager` to schedule keep-alive events. The Linux desktop implementation should use an equivalent system timer facility.
- RFC 2177 recommends clients not hold an IDLE longer than 29 minutes without a refresh. The 15-minute default provides a comfortable margin. Open Question OQ-3 asks whether users should be allowed to set values above 29 minutes.
