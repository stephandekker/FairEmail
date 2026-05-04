# Insecure Connections Toggle

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add an "Allow insecure connections" toggle to the inbound configuration screen. Default is off. When enabled, the application accepts server certificates that would otherwise fail validation (self-signed, expired, hostname mismatch). Enabling this toggle automatically disables the DANE toggle (story 12), since DANE verification is incompatible with insecure certificate acceptance.

This toggle is account-scoped, not global (design note N-5). When enabled, the validation rules for username and password are relaxed per FR-18 and FR-19.

Covers epic sections: FR-11, FR-12, FR-18, FR-19 (partial).

## Acceptance criteria

- [ ] An "Allow insecure connections" toggle is present on the inbound configuration screen
- [ ] The toggle defaults to off
- [ ] When enabled, the connection test succeeds against servers with self-signed, expired, or hostname-mismatched certificates
- [ ] Enabling the toggle automatically disables the DANE toggle (if present)
- [ ] When enabled, the username field is no longer required per FR-18
- [ ] When enabled, the password field is no longer required per FR-19
- [ ] The toggle applies only to this account, not globally

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-3 (allow insecure connections for self-signed certificates)
