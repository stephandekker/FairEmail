## Parent Feature

#14.6 System mailto: Handler

## What to build

Sanitize all content extracted from `mailto:` URIs before it reaches the compose window:

1. **Address sanitization (FR-20):** Strip leading/trailing whitespace and invisible Unicode characters (zero-width spaces, zero-width joiners, etc.) from all email addresses (To, CC, BCC). This defends against copy-paste artifacts and visually-deceptive addresses (design note N-3).

2. **Body sanitization (FR-21):** Treat body text as plain text only. Any HTML tags, script content, or markup in the body parameter must be escaped and rendered as literal text, not interpreted or executed. This prevents injection of formatted content or scripts via malicious `mailto:` links (design note N-4).

These sanitizations apply to the output of the URI parser before it is passed to the compose window. They cut across the full path: parse -> sanitize -> display.

Covers epic sections: FR-20, FR-21; AC-11, AC-12; NFR-4.

## Acceptance criteria

- [ ] Email addresses containing invisible Unicode characters (zero-width spaces, etc.) have those characters stripped; the displayed address is clean
- [ ] Leading and trailing whitespace is removed from all email addresses
- [ ] Body text containing HTML tags (e.g. `<script>alert('x')</script>`) displays those tags as literal text, not as rendered or executed markup
- [ ] Body text containing benign formatting tags (e.g. `<b>bold</b>`) is displayed literally, not formatted
- [ ] Sanitization does not corrupt legitimate addresses or body content
- [ ] All BCC recipients are clearly visible for user review before sending (preventing hidden-recipient surprises)

## Blocked by

- Blocked by `4-full-field-prepopulation`

## User stories addressed

- US-14 (address sanitization against deceptive addresses)
- US-15 (body treated as plain text, not executable markup)
- US-16 (all pre-filled fields clearly displayed for review, especially BCC)

## Type

AFK
