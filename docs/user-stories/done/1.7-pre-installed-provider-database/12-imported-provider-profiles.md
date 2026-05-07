# User Story: Imported Provider Profiles

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As an **IT administrator**, I want to import a custom provider profile from a file so that users in my organisation get the same zero-configuration experience as users of mainstream providers, and I want imported profiles to coexist with the bundled catalogue so that mainstream providers continue to work alongside my custom entries.

This slice delivers imported/custom profile support:
- Import provider profiles from a user-supplied file using the same data model as the bundled catalogue (FR-39).
- Merge imported profiles with the bundled catalogue at load time — both are available for matching and browsing (FR-40).
- Define deterministic, documented conflict-resolution behaviour when an imported and a bundled profile match the same domain (FR-41).

## Acceptance Criteria
- [ ] The application supports importing provider profiles from a user-supplied file.
- [ ] Imported profiles use the same data model as bundled catalogue entries.
- [ ] After import, the imported provider appears in both the browsable list and automatic domain matching (AC-13).
- [ ] Importing a custom profile does not remove or disable any bundled provider — both coexist.
- [ ] When an imported profile and a bundled profile have overlapping domain patterns, the conflict is resolved deterministically (e.g. imported wins, bundled wins, or user chooses).
- [ ] The conflict-resolution behaviour is documented and predictable.

## Blocked by
`1-provider-data-model-and-domain-matching`, `4-browsable-provider-list`

## HITL / AFK
**HITL** — The conflict-resolution policy (OQ-1 in the epic) is an open question. The implementer should choose a reasonable default (e.g. imported profiles take priority, since an admin importing a profile presumably wants it to override the bundled entry) and document it, but this decision should be reviewed.

## Notes
- OQ-1 in the epic explicitly flags conflict resolution as an open question. The recommended default is "imported wins" (an admin who imports a profile for a domain that already has a bundled entry presumably wants the imported one to take effect). This should be documented and reviewable.
- The epic does not specify the import file format. It should use the same format as the bundled catalogue for consistency and to minimise tooling requirements for administrators.
- The existing Android app loads imported providers from the app cache directory. The desktop app should define an appropriate import location and mechanism (e.g. file picker, config directory).
