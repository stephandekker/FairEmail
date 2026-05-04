## Parent Feature

#4.14 Attachments

## What to build

Before sending, scan the message body for locale-specific keywords that suggest the user intended to attach a file (e.g. "attached", "attachment", "enclosed" in English; "pièce jointe" in French; "Anhang" in German). If any keyword is found and no attachments have been added, warn the user. The keyword list is configurable per locale. Adding an attachment suppresses the warning.

Covers epic sections: US-25, FR-29, AC-13, AC-22, N-5.

## Acceptance criteria

- [ ] A message containing "attached" with no attachments triggers a warning before send (AC-13).
- [ ] Adding an attachment suppresses the warning (AC-13).
- [ ] The keyword list is locale-sensitive (e.g. "pièce jointe" in French, "Anhang" in German) (AC-22).
- [ ] The keyword list is configurable per locale (FR-29).
- [ ] The warning is easily dismissible (N-5).

## Blocked by

- Blocked by `1-basic-file-attachment`

## User stories addressed

- US-25
