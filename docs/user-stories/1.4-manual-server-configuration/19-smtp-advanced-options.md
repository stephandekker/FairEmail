# SMTP Advanced Options

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add two advanced options to the SMTP identity configuration screen:

1. **"Use IP address in EHLO"** toggle (default on). When enabled, the application uses the device's IP address in the SMTP EHLO command. When disabled, a custom EHLO hostname field is shown.
2. **Custom EHLO hostname** field — used when "Use IP" is disabled and a value is provided.
3. **"Login before send"** toggle — when enabled, the application verifies the inbound (IMAP/POP3) account is accessible before each SMTP send attempt.

Covers epic sections: FR-52, FR-53, FR-54.

## Acceptance criteria

- [ ] A "Use IP address in EHLO" toggle is present, defaulting to on
- [ ] When "Use IP" is enabled, the EHLO command uses the device's IP address
- [ ] When "Use IP" is disabled, a custom EHLO hostname field is shown
- [ ] The custom EHLO hostname is used in the SMTP greeting when provided
- [ ] A "Login before send" toggle is present
- [ ] When "Login before send" is enabled, the inbound account is verified before each SMTP send
- [ ] All settings are persisted with the identity

## Blocked by

- Blocked by `16-smtp-identity-config`

## User stories addressed

- US-39 (EHLO IP/hostname control)
- US-40 (login before send)
