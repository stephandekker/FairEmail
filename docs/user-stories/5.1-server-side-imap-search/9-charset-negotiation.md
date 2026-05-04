# Charset Negotiation and Transliteration Fallback

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, I want the application to handle non-ASCII characters in my search query by requesting UTF-8 encoding from the server and falling back to transliterated ASCII if the server does not support UTF-8, so that searches for names with accents or non-Latin characters degrade gracefully rather than failing.

## Blocked by
2-server-side-search-single-folder

## Acceptance Criteria
- When constructing a server search command, the application requests UTF-8 charset encoding if the server advertises support.
- If the server rejects the UTF-8-encoded search, the application retries with ASCII encoding, transliterating non-ASCII characters to their closest ASCII equivalents (e.g. ß -> ss, ø -> o, accented characters -> base characters).
- If both UTF-8 and ASCII attempts fail, the search fails with an error message identifying the server and the nature of the failure.
- A search for "Munchen" (transliterated from "München") produces partial results rather than a failure on a server that does not support UTF-8.

## Mapping to Epic
- US-18
- FR-16, FR-17, FR-18
- AC-9

## Notes
- The transliteration is intentionally lossy (see epic Design Note N-4). A search for "Müller" matching "Muller" is more useful than an error.
- The two-stage approach (try full query, then retry with reduced charset) follows the same "try and fall back" philosophy as capability degradation (slice #10).
