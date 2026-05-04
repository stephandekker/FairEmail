## Parent Feature

#4.5 Signature Management

## What to build

Add a boolean setting to automatically remove recognized signatures from quoted text when composing a reply (FR-35). Defaults to disabled (FR-36). The removal recognizes signatures from at least: this application, Gmail, and Outlook (FR-37). Removal is best-effort — unrecognized formats are left in the quoted text (FR-38).

This slice is independent of the signature insertion logic and operates on the quoted/forwarded content at reply composition time.

Covers epic sections: §6.7 (US-22), §7.10 (FR-35 – FR-38).

## Acceptance criteria

- [ ] A boolean "remove signatures from quoted text" setting exists, defaulting to disabled
- [ ] AC-13: With the setting enabled, replying to a message containing a recognized signature (from this app, Gmail, or Outlook) strips the signature from the quoted text
- [ ] Unrecognized signature formats are left in the quoted text (best-effort, FR-38)
- [ ] The setting has no effect on the user's own signature insertion — only on quoted content from the original message

## Blocked by

- Blocked by `1-basic-signature-storage-and-insertion`

## User stories addressed

- US-22

## Notes

- OQ-4 from the epic asks whether the set of recognized third-party signature formats should be expanded beyond Gmail and Outlook. This slice implements the minimum set specified in the epic; additional formats can be added incrementally.
