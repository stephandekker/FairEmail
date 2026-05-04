## Parent Feature

#4.5 Signature Management

## What to build

Ensure signatures are included in the application's settings export/import mechanism, so that they can be backed up and restored alongside other identity configuration (FR-4, NFR-3). All signature content — including embedded image references — must survive a round-trip export and import on a fresh installation.

Covers epic sections: §7.1 (FR-4), NFR-3.

## Acceptance criteria

- [ ] AC-14: Exporting settings and importing them on a fresh installation restores all identity signatures
- [ ] Embedded image references in signatures remain valid after export/import
- [ ] Signatures with rich text formatting are preserved faithfully through export/import
- [ ] Signatures with variable placeholders are preserved as templates (unexpanded) through export/import

## Blocked by

- Blocked by `4-image-insertion-in-signatures`

## User stories addressed

- (No specific user story in §6; this covers FR-4 and NFR-3 which are cross-cutting requirements)

## Notes

- This slice depends on the image insertion slice because the export/import mechanism must handle whatever image storage approach is chosen there. If image handling is deferred, this slice can still be implemented for text/HTML-only signatures, with image support added later.
