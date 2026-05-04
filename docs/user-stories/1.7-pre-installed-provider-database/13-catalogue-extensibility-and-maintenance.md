# User Story: Catalogue Extensibility and Maintenance

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **developer or catalogue maintainer**, I want to add, modify, or remove provider entries by changing only the catalogue data file — without code changes for routine maintenance — and I want the data model to accommodate new optional fields without breaking existing entries, so that the catalogue can evolve safely across releases.

This slice ensures the catalogue is maintainable as a pure data concern:
- Adding, modifying, or removing a provider entry requires only a change to the catalogue data file and a new application release — no code changes for routine maintenance (FR-43).
- The catalogue is updated as part of the regular release cycle; no over-the-air update mechanism (FR-42).
- The data model accommodates new optional fields (e.g. new quirk types, new OAuth parameters) without breaking existing entries or requiring format migration (NFR-3).

## Acceptance Criteria
- [ ] A new provider can be added to the catalogue by editing only the data file — no source code changes required.
- [ ] An existing provider's settings can be modified by editing only the data file.
- [ ] A provider can be removed by editing only the data file.
- [ ] Adding a new optional field to the data model does not break parsing of existing entries that lack the field.
- [ ] The catalogue is versioned and released with the application — there is no separate update mechanism.

## Blocked by
`1-provider-data-model-and-domain-matching`

## HITL / AFK
**AFK** — These are structural/architectural properties verified by adding a test provider entry and confirming no code changes are needed.

## Notes
- FR-43 includes the caveat "unless a new quirk type is introduced" — adding a *new kind* of behavioural override will require code changes to interpret the new field. This is expected and acceptable. The point is that adding a *new provider* with existing quirk types is data-only.
- This story is partly a quality gate on the architecture established in story 1. If story 1's data model and parsing are well-designed, this story's acceptance criteria should be trivially satisfied. It is listed separately to make the maintainability requirement explicit and testable.
