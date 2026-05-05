## Parent Feature

#1.3 Quick Setup Wizard

## What to build

Support loading an additional, user-supplied provider file that augments or overrides the bundled provider database (FR-16, NFR-5).

**Behavior:**
- The application supports a documented provider-database format (NFR-5).
- A user-supplied provider file can add new provider entries or override existing bundled entries.
- User-supplied entries participate in the same domain-matching and score-based ranking as bundled entries.
- This allows administrators or power users to add custom providers (e.g. corporate mail servers) without modifying the application binary.

## Acceptance criteria

- [ ] The provider database format is documented (NFR-5)
- [ ] A user-supplied provider file can be loaded alongside the bundled database (FR-16)
- [ ] User-supplied entries can add new providers not in the bundled database
- [ ] User-supplied entries can override existing bundled entries
- [ ] User-supplied entries participate in domain matching and score-based ranking
- [ ] The application works correctly when no user-supplied file is present (default behavior)

## Blocked by

- Blocked by 2-bundled-provider-database

## User stories addressed

- (No direct user story in section 6 of the epic; addresses FR-16 and NFR-5 extensibility requirements)

## Notes

- The epic does not specify the file format or location for user-supplied provider files. This is a design decision for implementation. Common choices include a JSON/XML file in a well-known config directory.
