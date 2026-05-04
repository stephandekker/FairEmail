## Parent Feature

#1.3 Quick Setup Wizard

## What to build

The offline, bundled provider database that ships with the application and maps email domains to server settings. This is the highest-confidence detection strategy and the foundation for all provider-aware behavior in the wizard.

**Database structure:** each provider entry contains at minimum the fields specified in FR-15:
- Unique identifier and display name (FR-15a)
- Domain-matching patterns (FR-15b)
- MX-matching patterns (FR-15c)
- Incoming server: hostname, port, encryption mode (FR-15d)
- Outgoing server: hostname, port, encryption mode (FR-15e)
- Username type (FR-15f)
- Keep-alive interval (FR-15g)
- NOOP keep-alive flag (FR-15h)
- Partial-fetch support flag (FR-15i)
- Maximum TLS version (FR-15j)
- App-specific password required flag (FR-15k)
- Provider documentation link (FR-15l)
- Localized documentation snippets (FR-15m)
- OAuth configuration (FR-15n)
- Display order / priority (FR-15o)
- Enabled/disabled flag (FR-15p)

**Matching:** given an email address, extract the domain and match against provider entries' domain patterns. Return a candidate with a confidence score. Bundled-database matches always score higher than any network-discovered candidate (FR-11).

**Coverage:** at least 150 provider entries covering major global and regional providers (NFR-4). The top 20 providers by global market share must be present (NFR-3).

**Offline:** the database is usable without any network access (FR-9, NFR-2).

This slice delivers the database, its schema, the domain-matching lookup function, and the score-based candidate model. It does NOT deliver the wizard UI integration or connectivity check — those are separate slices.

## Acceptance criteria

- [ ] The bundled database contains at least 150 provider entries (NFR-4)
- [ ] Each entry supports all fields defined in FR-15 (a through p)
- [ ] Domain-pattern matching correctly resolves well-known domains (e.g. gmail.com, outlook.com, yahoo.com) to their provider entries (NFR-3)
- [ ] Bundled-database lookups complete in well under one second (NFR-1)
- [ ] The database is usable with no network access (FR-9, NFR-2)
- [ ] A bundled-database match produces a candidate with a score that outranks any network-discovered candidate (FR-11)
- [ ] The top 20 email providers by global market share are present in the database (NFR-3)

## Blocked by

None - can start immediately

## User stories addressed

- US-7 (detect provider from email address and fill in server settings)

## Notes

- The epic specifies "at least 150 provider entries" (NFR-4). The FairEmail Android source bundles a provider XML file that may serve as a starting point, but the exact format and content for the Linux desktop version is a design decision. The epic deliberately does not prescribe a file format.
- FR-15n (OAuth configuration) fields are defined here but the OAuth flow itself is handled in a separate slice (and the OAuth token lifecycle is out of scope per NG2).
