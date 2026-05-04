## Parent Feature

#14.6 System mailto: Handler

## What to build

Extend the warm-start compose path (slice 3) to pre-populate all remaining RFC 6068 fields: Subject, Body (with line-break preservation), CC, BCC, and multiple comma-separated To recipients. After this slice, every standard `mailto:` parameter is wired end-to-end from URI to compose window.

Body text must be rendered with line breaks preserved (`%0D%0A` / `%0A` -> line breaks) and treated as plain text (FR-7). All pre-filled fields must be editable.

Covers epic sections: FR-5, FR-7, FR-9, FR-11 (full), FR-12; AC-4, AC-5, AC-6.

## Acceptance criteria

- [ ] A `mailto:` link with `?subject=Test&body=Hello%0AWorld` opens compose with Subject "Test" and Body containing "Hello" and "World" on separate lines
- [ ] A `mailto:` link with `?cc=a@x.com&bcc=b@x.com` populates the CC and BCC fields correctly
- [ ] A `mailto:` link with multiple comma-separated To addresses populates all of them in the To field
- [ ] A `mailto:?subject=Test` (empty path) opens compose with empty To and Subject "Test"
- [ ] All pre-filled fields (Subject, Body, CC, BCC) are editable before sending
- [ ] Unrecognized parameters in the URI do not prevent the compose window from opening with the fields that were successfully parsed

## Blocked by

- Blocked by `3-basic-warm-start-compose`

## User stories addressed

- US-5 (subject and body pre-filled)
- US-6 (CC and BCC pre-populated)
- US-7 (multiple To recipients)
- US-12 (graceful handling of unrecognized parameters, compose-window side)

## Type

AFK
