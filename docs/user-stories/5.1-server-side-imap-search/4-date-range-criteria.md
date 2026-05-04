# Date Range Search Criteria

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, I want to specify a date range (after date, before date, or both) in the search dialog, so that I can narrow search results to a specific time period on both local and server searches.

## Blocked by
2-server-side-search-single-folder

## Acceptance Criteria
- The advanced options section includes a date-range selector with a start date and an end date.
- The user can specify a start date only, an end date only, or both.
- Dates are selectable via a date picker control.
- For server search, the dates are translated to IMAP SINCE and BEFORE criteria.
- Specifying both dates returns only messages whose received date falls within that range.
- Date criteria combine with text criteria using AND logic.

## Mapping to Epic
- US-9
- FR-7 (Date — on or after, Date — on or before), FR-9
- AC-2

## Notes
- IMAP date-based search uses date-only granularity (no time component). This is an IMAP protocol limitation, not an application choice. The date picker should reflect this by offering date-only selection.
