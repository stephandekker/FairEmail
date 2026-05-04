# Structured Prefix Search Expressions

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As a power user, I want to type prefix-based search expressions directly in the query field (e.g. `from:alice@example.com`, `to:bob`, `cc:carol`, `bcc:dave`, `keyword:important`), so that I can target specific message fields without opening the advanced options panel.

## Blocked by
3-search-field-toggles

## Acceptance Criteria
- The following prefixes are recognized in the query text: `from:`, `to:`, `cc:`, `bcc:`, `keyword:`.
- Each prefix is translated into the corresponding targeted IMAP SEARCH criterion (FROM, TO, CC, BCC, KEYWORD).
- A `from:` prefix targets only the sender field on the server, regardless of which checkboxes are enabled in the advanced options.
- Prefix expressions are treated as AND conditions combined with any remaining free-text query and any criteria selected in the advanced options.
- Multiple prefixes in the same query are supported (e.g. `from:alice to:bob`).
- Prefix parsing works for both local and server search paths.

## Mapping to Epic
- US-13, US-14 (prefix portion only, not raw:)
- FR-11, FR-12
- AC-7

## Notes
- The `raw:` prefix for Gmail is handled separately in slice #8 because it follows a completely different code path (X-GM-RAW extension) and has different scoping rules.
- Uncertainty: the epic does not specify how to handle malformed prefix expressions (e.g. `from:` with no value, or `from:` mid-word). The recommendation is to treat a prefix as recognized only when it appears at the start of a whitespace-delimited token, and to treat `from:` with no value as a no-op for that criterion.
