# Domain-Based Auto-Config for Inbound Server

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add a domain input field and an "Auto-config" button to the inbound configuration screen. When activated, auto-config attempts to discover server settings in the following order: (1) bundled provider database by domain match, (2) DNS NS record lookup, (3) DNS SRV records per RFC 6186, (4) MX record lookup, (5) well-known auto-configuration XML endpoints, (6) port scanning as a last resort.

On success, hostname, port, and encryption mode are populated with discovered values. All other fields remain unchanged, and all auto-filled fields remain fully editable. On failure, a clear error message is displayed and no fields are modified.

Auto-config runs in the background with a progress indicator; the auto-config button and domain field are disabled during the operation.

Covers epic sections: FR-23 through FR-28.

## Acceptance criteria

- [ ] A domain input field and "Auto-config" button are present on the inbound configuration screen
- [ ] Auto-config attempts discovery in the specified order (provider DB, DNS NS, SRV, MX, autoconfig XML, port scan)
- [ ] On success, hostname, port, and encryption mode are populated with discovered values
- [ ] Username, password, certificate, and security options are NOT modified by auto-config
- [ ] All auto-filled fields remain fully editable after population
- [ ] On failure, a clear error message is shown and no fields are modified
- [ ] A progress indicator is shown during the operation
- [ ] The auto-config button and domain field are disabled during the operation

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-8 (auto-config pre-fills host, port, encryption from domain)
- US-9 (all auto-filled fields remain editable)
- US-10 (auto-config failure shows clear message)

## Notes

Open question OQ-2 from the epic: the total timeout and whether to show stage-by-stage progress is undecided. The source application shows only a spinner with no stage indication. This is a design decision to flag for future review.
