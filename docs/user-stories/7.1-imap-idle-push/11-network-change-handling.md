# Network Change Detection and Reconnection

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, when my network interface changes (e.g. switching from wired to wireless, or getting a new IP via DHCP renewal), I want the application to detect the change and re-establish all IDLE connections, so that push continues working reliably on the new network path.

## Blocked by
- `7-connection-failure-recovery`

## Acceptance Criteria
- The application monitors system network-change events (interface changes, IP address changes, connectivity restoration) (FR-32, FR-35).
- When the active network changes, the application tears down all IDLE sessions and re-establishes them on the new network path (FR-35, Design Note N-7).
- Recovery is attempted automatically whenever network connectivity is restored after a period of disconnection (FR-32).
- All push/poll behavior functions correctly over both IPv4 and IPv6 connections (AC-18).
- The application survives an IP address change (e.g. DHCP renewal) by reconnecting transparently (AC-18).
- Reconnection after network change includes a full folder check before re-entering IDLE (FR-30).
- Reconnection scheduling avoids thundering-herd effects — all accounts do not reconnect simultaneously (NFR-7).

## Mapping to Epic
- US-11 (partial), US-20
- FR-32, FR-35
- NFR-7
- AC-6 (partial), AC-18

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Design Note N-7 explains the aggressive reconnection strategy: a network change may mean new IP, NAT state, or firewall rules. Probing existing connections is less reliable than a clean teardown and reconnect.
- On Linux desktop, network change detection can use NetworkManager D-Bus signals or equivalent. The specific mechanism is an implementation choice.
- This story is deliberately aggressive about reconnection. Story 7 handles the reconnection mechanics (backoff, full sync); this story adds the network-change trigger.
