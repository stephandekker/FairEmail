# Inbound Test Connection — Error Diagnostics

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Enhance the inbound test connection (from story 5) to provide actionable, categorized error messages on failure. At minimum the following failure categories must be distinguished with specific guidance:

- DNS resolution failure (unknown host)
- Connection refused / timeout
- TLS/SSL handshake failure (certificate error, protocol mismatch)
- Authentication failure (wrong credentials)
- Protocol mismatch (e.g. POP3 port used for IMAP)
- Untrusted certificate (fingerprint display and trust option are in story 10)

When the account is connected to a provider that has documentation or a help link in the bundled database, the error display includes a link to that provider's setup documentation.

Covers epic sections: FR-36, FR-37, NFR-6.

## Acceptance criteria

- [ ] DNS resolution failure shows a specific "unknown host" error with the hostname
- [ ] Connection refused or timeout shows a specific error mentioning the host and port
- [ ] TLS/SSL handshake failure shows a certificate or protocol error with guidance
- [ ] Authentication failure shows a "wrong credentials" error
- [ ] Protocol mismatch (e.g. connecting IMAP to a POP3 port) shows a protocol mismatch hint
- [ ] Error messages are specific enough for the user to diagnose and correct without external help
- [ ] When the provider has a help link in the bundled database, the error includes a link to provider setup docs

## Blocked by

- Blocked by `5-inbound-test-connection`

## User stories addressed

- US-16 (actionable error message identifying the cause)
