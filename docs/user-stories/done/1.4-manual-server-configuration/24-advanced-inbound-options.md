# Advanced Inbound Options

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add three advanced options to the inbound account configuration screen:

1. **Polling interval** — configurable interval for periodic synchronization, allowing the user to balance responsiveness against resource usage.
2. **Network restrictions** — options to restrict synchronization to unmetered networks only and/or VPN-only connections.
3. **UTF-8 / internationalized email support** — toggle to enable or disable UTF-8 support for this connection, controlling whether the application uses non-ASCII in protocol commands.

Covers epic sections: FR (implicit from user stories; no dedicated FR numbers for these).

## Acceptance criteria

- [ ] A polling interval setting is available on the inbound configuration screen
- [ ] The user can restrict synchronization to unmetered networks only
- [ ] The user can restrict synchronization to VPN-only connections
- [ ] A UTF-8 / internationalized email toggle is available
- [ ] All advanced options are persisted with the account settings
- [ ] The polling interval is respected by the synchronization engine

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-36 (configure polling interval)
- US-37 (restrict sync to unmetered/VPN-only networks)
- US-38 (enable/disable UTF-8 support)
