## Parent Feature

#4.5 Signature Management

## What to build

Support variable placeholders in signatures that are expanded when the signature is inserted into a message (FR-26). The supported placeholders are: `$name$` (full display name), `$firstname$` (first word of display name), `$lastname$` (last word of display name), `$email$` (identity email address), `$date$` (current date in system long date format), `$weekday$` (current day of week). Date-related placeholders respect the user's locale (FR-27). Unrecognized placeholders are left as-is (FR-28). Expansion occurs locally at composition time (NFR-5).

Covers epic sections: §6.5 (US-17, US-18), §7.7 (FR-26 – FR-28).

## Acceptance criteria

- [ ] Signatures can contain placeholder tokens: `$name$`, `$firstname$`, `$lastname$`, `$email$`, `$date$`, `$weekday$`
- [ ] AC-9: A signature with `$name$` and `$date$`, sent from identity "Alice Smith", produces "Alice Smith" and the current locale-formatted date in the message
- [ ] `$firstname$` expands to "Alice" and `$lastname$` expands to "Smith" for display name "Alice Smith"
- [ ] `$email$` expands to the identity's email address
- [ ] `$weekday$` expands to the current day of the week in the user's locale
- [ ] Date-related placeholders respect the user's locale (FR-27)
- [ ] Unrecognized placeholders (e.g. `$unknown$`) are left as-is in the output (FR-28)
- [ ] Expansion happens locally at compose time, no data sent to external services (NFR-5)

## Blocked by

- Blocked by `1-basic-signature-storage-and-insertion`

## User stories addressed

- US-17
- US-18

## Notes

- OQ-1 from the epic flags uncertainty about whether the Android source actually supports variable expansion in signatures (vs. only in reply templates). The epic resolves this in favor of supporting variables in signatures, consistent with the high-level feature description.
- OQ-2 asks whether users should be able to customize the date format within the placeholder syntax. This slice implements system-default formatting only; custom format support could be a future enhancement.
