## Parent Feature

#4.5 Signature Management

## What to build

Enable inserting images (e.g. a company logo) into signatures via the rich text editor. Image references must remain valid when the signature is used in messages and across application restarts (FR-9). This slice must address the image persistence challenge noted in OQ-5 — the desktop application needs a mechanism (different from Android's persistent URI permissions) to ensure embedded images remain accessible.

Covers epic sections: §6.1 (US-6), §7.2 (FR-9).

## Acceptance criteria

- [ ] The signature editor allows inserting images (e.g. from a file picker)
- [ ] Inserted images render correctly in the signature preview
- [ ] Images in the signature are included faithfully in outgoing messages (AC-1)
- [ ] Image references remain valid across application restarts
- [ ] Images survive settings export/import (this will be fully verified in the export/import slice, but the storage mechanism must support it)

## Blocked by

- Blocked by `2-rich-text-signature-editor`

## User stories addressed

- US-6

## Notes

- OQ-5 from the epic flags that the desktop application needs a different image persistence mechanism than the Android source (which uses persistent URI permissions). The design decision for image storage/referencing should be made during implementation of this slice. Consider embedding images as data URIs or copying them to an application-managed directory.
