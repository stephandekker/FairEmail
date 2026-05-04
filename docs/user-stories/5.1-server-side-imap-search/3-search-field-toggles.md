# Advanced Search Field Toggles

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, I want to expand an advanced options section in the search dialog where I can individually enable or disable search fields (senders, recipients, subject, keywords, message body), so that I can control exactly which parts of the message are searched on the server.

## Blocked by
2-server-side-search-single-folder

## Acceptance Criteria
- The search dialog has an expandable "advanced options" section.
- The advanced options section contains individually toggleable checkboxes for: senders, recipients, subject, keywords, and message body.
- Each enabled text field is translated into the corresponding IMAP SEARCH criterion (FROM, TO/CC/BCC, SUBJECT, KEYWORD, BODY).
- Multiple enabled fields are combined with AND logic.
- Disabling all text fields and submitting only a text query produces no server results (or the application handles this gracefully).
- Toggling fields works for both local and server search paths.

## Mapping to Epic
- US-8
- FR-7, FR-8
- AC-3, AC-4, AC-5 (field-specific matching)

## Notes
- The body search toggle is included here, but the capability degradation behaviour (auto-retry without body when server rejects it) is handled in slice #10.
- Keywords toggle is included here, but keyword capability degradation is also in slice #10.
