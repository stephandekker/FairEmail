# Tray Icon Status Tooltip

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want the tray icon's tooltip to show aggregate status information (monitored accounts, pending operations, network state), so that I can tell at a glance what the daemon is doing.

## Blocked by
- `2-system-tray-icon-presence`

## Acceptance Criteria
- The tray icon tooltip displays the number of accounts currently being monitored (e.g. "Monitoring 3 account(s)").
- When pending operations exist, the tooltip displays the count (e.g. "2 operation(s) pending"), and the count decreases as operations complete.
- When the network is unsuitable for synchronization, the tooltip indicates the daemon is waiting for a suitable connection.
- When zero accounts are being monitored but the daemon is still running (e.g. only processing pending operations), the tooltip indicates this distinct state.
- The tooltip updates within seconds when the underlying state changes (account connects/disconnects, operation completes, network changes).
- Tooltip text is accessible to screen readers.

## Mapping to Epic
- US-8, US-9, US-10
- FR-11, FR-12, FR-13
- NFR-7 (accessibility)
- AC-7, AC-8, AC-9 (partially — tray tooltip reflects state)

## HITL / AFK
AFK — the content of the tooltip is well-specified by the epic (N-6: aggregate status, not per-account detail).

## Notes
- OQ-2 (unread count on tray icon) is relevant. The epic notes that the source application does NOT show aggregate unread count in the foreground notification — only monitored account count and pending operation count. Whether to add an unread count badge is an open design question. This story implements only what the epic specifies (account count + pending ops + network state). Unread count can be added in a follow-up if the design decision resolves in its favor.
