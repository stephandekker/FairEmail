## Parent Feature

#4.14 Attachments

## What to build

Allow the user to preview the contents list of a zipped attachment without extracting it. The preview shows the list of files contained in the archive.

Covers epic sections: US-41, FR-45, AC-20.

## Acceptance criteria

- [ ] A received .zip attachment offers a "view contents" action (AC-20).
- [ ] The action shows the list of files inside the archive without extracting them.
- [ ] The preview is available after the zip file has been downloaded.

## Blocked by

- Blocked by `21-download-attachments`

## User stories addressed

- US-41

## Notes

- OQ-8 from the epic asks whether nested zips (zip within zip) should also be previewable. This story implements top-level preview only per the current specification. The open question should be resolved before expanding scope.
