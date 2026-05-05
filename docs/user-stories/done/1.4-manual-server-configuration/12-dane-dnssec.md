# DANE and DNSSEC Toggles

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add two security toggles to the inbound configuration screen:

1. **DANE** (default off) — when enabled, the application verifies the server's TLS certificate against TLSA records in DNSSEC-signed DNS.
2. **DNSSEC** (default off) — when enabled, the application requires DNSSEC-validated DNS resolution for the server hostname.

When "Allow insecure connections" (story 9) is enabled, the DANE toggle is automatically disabled and cannot be turned on (FR-12).

Covers epic sections: FR-13, FR-14.

## Acceptance criteria

- [ ] A DANE toggle is present on the inbound configuration screen, defaulting to off
- [ ] A DNSSEC toggle is present on the inbound configuration screen, defaulting to off
- [ ] When DANE is enabled, the server's certificate is verified against TLSA records
- [ ] When DNSSEC is enabled, DNS resolution requires DNSSEC validation
- [ ] When "Allow insecure connections" is enabled, the DANE toggle is disabled and cannot be turned on
- [ ] Both toggles are persisted with the account settings

## Blocked by

- Blocked by `9-insecure-connections-toggle`

## User stories addressed

- US-6 (enable DANE and/or DNSSEC verification)
