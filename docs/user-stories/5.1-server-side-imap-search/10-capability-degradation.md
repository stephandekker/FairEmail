# Capability Degradation with User Notification

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, when the IMAP server rejects a search criterion (such as body text or keyword search), I want the application to automatically retry without the unsupported criterion and notify me what was excluded, so that I still get partial results rather than a blank failure.

## Blocked by
3-search-field-toggles

## Acceptance Criteria
- If the server rejects a search that includes a body-text criterion, the application automatically retries with the body criterion removed.
- If the server rejects a search that includes a keyword criterion, the application automatically retries with the keyword criterion removed.
- After each automatic retry, the user is notified which criterion was excluded from the results.
- The notification is visible and clearly explains why results may be broader than expected.
- Every automatic capability fallback produces a user-visible notification (degradation transparency).

## Mapping to Epic
- US-16, US-17
- FR-19, FR-20, FR-21
- NFR-3
- AC-5 (body degradation), AC-10 (keyword degradation)

## Notes
- This slice depends on #3 (field toggles) because degradation is meaningful only when individual fields can be toggled — the retry removes a specific field from the criteria set.
- The epic's Design Note N-3 explains the rationale: "try and fall back" avoids probing server capabilities in advance, works with servers that do not accurately report their capabilities, and ensures the user always gets the broadest results the server can provide.
