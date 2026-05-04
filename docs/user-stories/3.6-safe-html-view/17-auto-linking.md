# Auto-Linking of URLs and Email Addresses

## Parent Feature
#3.6 Safe HTML View

## Blocked by
2-core-sanitization-pipeline

## Description
Implement automatic conversion of plain-text URLs and email addresses in the sanitized output into clickable links. Rate-limit auto-linking per element to prevent denial-of-service from messages containing thousands of URL-like strings.

## Motivation
Many emails contain plain-text URLs or email addresses that are not wrapped in `<a>` tags. Auto-linking makes these actionable for the user. The rate limit prevents abuse where a malicious message could include thousands of URL-like strings to slow down processing.

## Acceptance Criteria
- [ ] Plain-text URLs (http://, https://) in sanitized message output are converted to clickable `<a>` links.
- [ ] Plain-text email addresses are converted to clickable `mailto:` links.
- [ ] Auto-linking is rate-limited per element — after a threshold number of conversions within a single element, remaining URL-like strings are left as plain text.
- [ ] Already-linked URLs (inside existing `<a>` tags) are not double-wrapped.
- [ ] A test message with plain-text URLs renders them as clickable links.
- [ ] A test message with thousands of URL-like strings in one element does not cause excessive processing time.

## HITL/AFK Classification
AFK — testable with crafted HTML inputs containing various URL patterns and stress-test inputs.

## Notes
- FR-52 and FR-53 govern this story.
- OQ-7 flags the open question about auto-linking false positives in technical messages (log dumps, code snippets). For now, implement straightforward URL/email detection with the rate limit; per-message suppression can be added later if needed.
- Link-click behavior (confirmation, tracking-parameter stripping) is out of scope — that's feature 3.12 (NG2).
