## Parent Feature

#4.14 Attachments

## What to build

The foundational compose-side attachment capability: a user clicks an "attach" action in the compose window, a system file picker opens, and one or more selected files are added to the draft. Each attached file is stored with its detected MIME type, original filename (sanitized against path-traversal), and file size. This slice also covers the core MIME-type detection and filename sanitization logic (FR-49) that all subsequent attachment sources will reuse.

This is the tracer bullet for the entire attachment system — it proves that a file can travel from the user's file system into a draft, be persisted, and be available for later sending.

Covers epic sections: US-1, FR-1, FR-49, NFR-1, NFR-6, AC-1.

## Acceptance criteria

- [ ] The compose window offers an "attach file" action that opens the system file picker.
- [ ] Selecting one or more files via the picker adds them to the draft's attachment list.
- [ ] Each attached file's MIME type is correctly detected; if no extension is present, the type is guessed from content.
- [ ] Filenames are sanitized to prevent path-traversal (no `../`, no absolute paths).
- [ ] Adding a file under 10 MB completes in under 2 seconds (NFR-1).
- [ ] Attachments can be added while offline; only send requires connectivity (NFR-6).

## Blocked by

None - can start immediately.

## User stories addressed

- US-1
