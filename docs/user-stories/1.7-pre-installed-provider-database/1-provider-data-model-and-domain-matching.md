# User Story: Provider Data Model and Domain Matching

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **developer building the account setup flow**, I want a bundled provider catalogue with a well-defined data model that can be parsed at startup and matched against an email domain using regex patterns, so that all downstream features (pre-fill, OAuth, quirks, UI list) have a single, reliable source of provider data to consume.

This is the foundational slice. It delivers:
- The provider data model (all fields specified in FR-2 and FR-3).
- A bundled catalogue file containing at least 150 provider entries (FR-1).
- A parser that loads the catalogue into memory at application startup (FR-4).
- Domain-matching logic: given an email address, extract the domain and match it against provider domain patterns using case-insensitive regular expressions (FR-5, FR-6, FR-7).
- The match result is a provider record (or "no match"). No settings are pre-filled yet — that is the next slice.

## Acceptance Criteria
- [ ] The application ships with a bundled catalogue file containing at least 150 provider entries.
- [ ] Each provider entry contains at minimum: a display name and one or more server configurations (IMAP and/or POP3, SMTP) with host, port, and encryption mode.
- [ ] Each provider entry may optionally contain: domain patterns, MX patterns, unique identifier, subtitle, sort-order priority, documentation URL, registration URL, inline documentation (with locale variants), OAuth profile, Graph profile, and behavioural overrides.
- [ ] The catalogue is loaded entirely from the local application bundle with no network dependency.
- [ ] Given an email address, the application extracts the domain and matches it against all enabled provider domain patterns using regular-expression matching.
- [ ] Domain matching is case-insensitive (e.g. `Alice@Gmail.COM` matches the Gmail provider).
- [ ] Domain patterns support wildcards and alternation sufficient for multi-TLD providers (e.g. `yahoo\..*` matches `yahoo.com`, `yahoo.co.uk`, `yahoo.fr`).
- [ ] Matching completes in well under one second with 200+ provider entries on a modern desktop (NFR-2).
- [ ] When no domain pattern matches, the result is "no match" (no error, no crash — the caller can fall back to other discovery methods).

## Blocked by
_(none — this is the first story)_

## HITL / AFK
**AFK** — This story is pure data-model and matching logic with clear, testable acceptance criteria. No human judgement needed during implementation.

## Notes
- The epic deliberately does not prescribe the storage format (XML, JSON, TOML, etc.) or the parsing/indexing strategy (FR-3 non-goal NG3). The implementation should choose a format that is easy to maintain and extend.
- The existing Android codebase uses `providers.xml` with an `XmlPullParser`. The Linux desktop app may choose a different format, but the data model contract from the epic must be fulfilled regardless.
- The epic's data model (FR-3) is extensive. This story requires the *model* to support all optional fields, but the matching logic in this slice only needs to use domain patterns. Other fields are consumed by later stories.
