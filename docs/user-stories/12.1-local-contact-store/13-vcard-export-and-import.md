## Parent Feature

#12.1 Local Contact Store

## What to build

Implement export and import of local contacts using the vCard format.

**Export** (FR-33): The user can export the local contact list to a vCard file. The contact type (sent-to, received-from, junk, no-junk) is preserved using an application-specific extension field in the vCard (FR-35). Group labels are included in the export (FR-29).

**Import** (FR-34): The user can import contacts from a vCard file. The import creates new contacts or updates existing ones (matched by account + type + email). Group assignments from the vCard are preserved (FR-29).

Round-tripping (export then import) must produce the same set of contacts with the same type, name, email, and group.

## Acceptance criteria

- [ ] The user can export local contacts to a vCard file (FR-33)
- [ ] The user can import contacts from a vCard file (FR-34)
- [ ] Contact type is preserved via an application-specific extension field (FR-35)
- [ ] Group labels are preserved across export and import (FR-29)
- [ ] Exporting and re-importing produces the same contacts with the same type, name, email, and group (AC-16)
- [ ] Importing updates existing contacts rather than creating duplicates when account + type + email match

## Blocked by

- Blocked by 1-contact-record-and-learn-on-send (requires the contact store)
- Blocked by 8-contact-groups (groups must be supported for round-trip preservation)

## User stories addressed

- US-33 (export to vCard)
- US-34 (import from vCard, preserving groups)

## Notes

Open question OQ-7 asks whether the desktop app should adopt the same vCard extension field as the source Android app, define its own, or use a standard vCard field. This needs a team decision before implementation — the choice affects interoperability with FairEmail Android exports and with third-party tools.
