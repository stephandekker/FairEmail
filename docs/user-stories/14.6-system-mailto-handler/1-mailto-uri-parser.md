## Parent Feature

#14.6 System mailto: Handler

## What to build

A standalone URI parsing module that accepts a raw `mailto:` URI string and returns a structured result containing all fields defined by RFC 6068: To (one or more addresses), CC, BCC, Subject, and Body. The parser must handle percent-decoding per RFC 3986, comma-separated To addresses, absent or empty path components (e.g. `mailto:?subject=Test`), and line-break sequences (`%0D%0A`, `%0A`) in the body. Unrecognized or unsupported header parameters must be silently discarded without affecting successfully parsed fields. Malformed URIs must not cause errors; the parser returns whatever fields it could extract.

This is the foundational building block for all subsequent mailto: handling slices. It is pure logic with no UI or system-integration dependency, making it independently testable.

Covers epic sections: FR-4 through FR-9.

## Acceptance criteria

- [ ] Given a simple URI `mailto:alice@example.com`, the parser extracts a single To address and no other fields
- [ ] Given a URI with query parameters `?subject=Hello&body=Hi%20there`, the parser extracts Subject and Body with correct percent-decoding
- [ ] Given a URI with `?cc=a@x.com&bcc=b@x.com`, the parser extracts CC and BCC fields
- [ ] Given a URI with multiple comma-separated To addresses, the parser returns all addresses as separate entries
- [ ] Given a URI with `%0D%0A` or `%0A` in the body, the parser preserves them as line breaks
- [ ] Given a URI with unrecognized parameters (e.g. `?foo=bar`), the parser silently ignores them and returns all recognized fields
- [ ] Given an empty path `mailto:?subject=Test`, the parser returns an empty To field and the parsed Subject
- [ ] Given a severely malformed URI, the parser does not throw/crash and returns a best-effort partial result

## Blocked by

None - can start immediately

## User stories addressed

- US-4 (basic recipient extraction)
- US-5 (subject and body extraction)
- US-6 (CC and BCC extraction)
- US-7 (multiple To recipients)
- US-12 (graceful handling of malformed/unrecognized URIs)

## Type

AFK
