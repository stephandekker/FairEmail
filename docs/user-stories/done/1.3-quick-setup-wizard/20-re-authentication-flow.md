## Parent Feature

#1.3 Quick Setup Wizard

## What to build

An option in the wizard to re-authorize an existing account whose credentials have expired or been revoked, rather than creating a duplicate account (FR-32, Design Note N-8).

**Behavior:**
- The wizard offers a visible option to re-authorize an existing account (e.g. "Authorize existing account again") (FR-32).
- When re-authorizing, the wizard matches an existing account by username and incoming-server protocol type (FR-34).
- Only the credentials (password or OAuth token) and the synchronization-enabled flag are updated. Folder structure, sync settings, identities, rules, and all other account properties are preserved (FR-33).

## Acceptance criteria

- [ ] The wizard offers a visible "re-authorize existing account" option (FR-32)
- [ ] Existing accounts are matched by username and incoming-server protocol type (FR-34)
- [ ] Re-authorization updates only the credentials (password or OAuth token) (AC-13, FR-33)
- [ ] The synchronization-enabled flag is updated (FR-33)
- [ ] Folder structure is preserved after re-authorization (AC-13)
- [ ] Rules and identity settings are unchanged after re-authorization (AC-13)
- [ ] Sync settings are preserved after re-authorization (AC-13)

## Blocked by

- Blocked by 13-account-and-identity-creation

## User stories addressed

- US-24 (re-authorize existing account without creating duplicate)
- US-25 (only credentials updated, existing configuration preserved)

## Notes

- Open Question OQ-7 in the epic asks whether re-authorization should prompt for whether to re-enable sync, since FR-33 re-enables sync and a user who previously disabled it may be surprised. This is an unresolved UX decision.
