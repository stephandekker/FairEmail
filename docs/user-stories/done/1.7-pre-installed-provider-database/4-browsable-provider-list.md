# User Story: Browsable Provider List

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **user who prefers to select my provider explicitly**, I want to browse a sorted list of all known providers — popular ones first, then alphabetical — so that I can find my provider visually rather than relying on automatic domain detection.

This slice delivers the browsable provider list UI:
- Present all enabled providers sorted by provider-defined priority (popular first), with ties broken by locale-aware alphabetical name comparison (FR-10, NFR-4).
- Exclude disabled providers from the list and from automatic matching (FR-11).
- Exclude debug-only providers unless the application is in debug/development mode (FR-12).
- Visually distinguish or group alternative variants with their primary variant (FR-13).
- Include a fallback "Other provider" entry that routes to manual server configuration (FR-14).

## Acceptance Criteria
- [ ] The provider list is visible and browsable in the account setup flow.
- [ ] Popular providers (e.g. Gmail, Outlook, Yahoo) appear near the top of the list, ahead of less common providers (AC-5).
- [ ] After priority-sorted providers, remaining providers are sorted alphabetically using locale-aware collation rules.
- [ ] Disabled providers do not appear in the list (AC-5).
- [ ] Debug-only providers do not appear in the list when the application is in normal mode; they do appear when debug mode is active (AC-5).
- [ ] Alternative variants of a provider are visually distinguished or grouped with their primary variant.
- [ ] An "Other provider" (or equivalent) fallback entry is present, routing the user to manual server configuration (US-6).
- [ ] Selecting a provider from the list pre-fills all server settings, username format, and behavioural overrides into the account configuration (AC-6).

## Blocked by
`1-provider-data-model-and-domain-matching`

## HITL / AFK
**AFK** — Sorting and filtering logic is well-specified. The UI layout may benefit from design review, but the story's acceptance criteria are testable without it.

## Notes
- The epic says providers are sorted by "provider-defined priority" then "locale-aware alphabetical name comparison". The existing Android app uses an `order` integer attribute for priority. The desktop app should honour the same semantic (lower order = higher priority) regardless of storage format.
