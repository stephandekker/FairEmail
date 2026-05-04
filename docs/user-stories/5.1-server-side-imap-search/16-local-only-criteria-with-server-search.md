# Local-Only Criteria Interaction with Server Search

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, when I have enabled both server-searchable criteria and local-only criteria (such as attachment presence, encryption status, local notes, or attachment filenames), I want the application to handle this gracefully — either by post-filtering server results using local-only criteria or by clearly indicating that some criteria apply only to local results — so that I understand the scope of my search.

## Blocked by
3-search-field-toggles, 5-flag-size-filters-quick-buttons

## Acceptance Criteria
- The following criteria are available for local (device) search only and are not sent to the server: local notes, attachment filenames, encryption status, attachment presence, hidden/snoozed state, message headers (debug mode), raw HTML content (debug mode).
- When the user has enabled any local-only criterion and also requests a server search, the application either applies the local-only filter as a post-filter on server results or clearly indicates that the criterion applies only to local results.
- The user is never silently misled about which criteria were applied to the server vs. locally.

## Mapping to Epic
- FR-29, FR-30

## Notes
- Open question OQ-6 in the epic acknowledges that the interaction between local-only criteria and server search is ambiguous. This story requires that the application handle it explicitly (post-filter or inform), but does not prescribe which approach. The implementation should choose based on feasibility and UX clarity.
- Uncertainty: post-filtering server results by local-only criteria (e.g. "has attachment") requires that the messages be at least partially downloaded to evaluate the local criterion. This may conflict with the purpose of server search (finding messages not in local cache). The alternative — informing the user that certain criteria only apply locally — may be the more honest UX. This decision should be made during design/implementation.
