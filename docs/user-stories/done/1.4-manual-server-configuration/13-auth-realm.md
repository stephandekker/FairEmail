# Authentication Realm Field

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add an optional authentication realm field to the inbound configuration screen. The field defaults to empty. When provided, the value is passed to the authentication mechanism as the SASL realm or NTLM domain parameter during connection and test operations.

Covers epic sections: FR-10.

## Acceptance criteria

- [ ] An optional "Realm" field is present on the inbound configuration screen
- [ ] The field defaults to empty
- [ ] When provided, the realm is used as the SASL realm or NTLM domain during authentication
- [ ] The realm value is persisted with the account settings
- [ ] The realm is used during connection tests

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-5 (optional authentication realm for SASL/NTLM domain)
