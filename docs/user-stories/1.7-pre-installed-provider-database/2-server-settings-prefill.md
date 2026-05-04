# User Story: Server Settings Pre-fill from Matched Provider

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **mainstream user**, when the application matches my email domain to a provider in the bundled catalogue, I want all IMAP/SMTP server settings (host, port, encryption mode) to be automatically pre-filled into my account configuration, so that I never have to look up or type server hostnames, port numbers, or encryption modes.

This slice connects the domain-matching result from story 1 to the account setup flow. It delivers:
- When a provider match is found, extract IMAP (or POP3) and SMTP server configurations from the provider entry (FR-15).
- Pre-fill these settings into the account configuration fields without requiring user modification (FR-17).
- Support for providers that define multiple server entries of the same type (FR-16) — use the first/primary entry by default.

## Acceptance Criteria
- [ ] When a provider is matched by domain, the IMAP host, port, and encryption mode are pre-filled into the account configuration.
- [ ] When a provider is matched by domain, the SMTP host, port, and encryption mode are pre-filled into the account configuration.
- [ ] Each server entry specifies exactly one of: SSL/TLS-on-connect or STARTTLS upgrade (FR-15).
- [ ] When a provider defines multiple IMAP or SMTP server entries, a deterministic selection is applied (e.g. first entry wins) and the selected entry is pre-filled.
- [ ] The user does not need to modify any server setting for a matched provider under normal circumstances.
- [ ] Entering a Gmail address matches Gmail and pre-fills `imap.gmail.com:993/SSL` and `smtp.gmail.com:465/SSL` (AC-1).
- [ ] Entering a Gmail address while fully offline still pre-fills settings (connection testing may fail, but detection and pre-fill succeed) (AC-2).

## Blocked by
`1-provider-data-model-and-domain-matching`

## HITL / AFK
**AFK** — Straightforward mapping from matched provider record to account configuration fields.
