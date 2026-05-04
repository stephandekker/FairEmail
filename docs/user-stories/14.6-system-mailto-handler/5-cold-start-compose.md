## Parent Feature

#14.6 System mailto: Handler

## What to build

When the application is not running and the user clicks a `mailto:` link, the operating system launches the application. The application must detect that it was launched with a `mailto:` URI, skip the normal "land on inbox" flow, and open directly into a compose window with the URI fields pre-populated. The user should not need to navigate through the main interface first (design note N-2).

The compose window must appear within the application's normal startup time plus one second (NFR-1). This slice reuses the URI parsing and field pre-population already built in slices 1, 3, and 4.

Covers epic sections: FR-10 (cold start path); AC-10; NFR-1 (cold start latency).

## Acceptance criteria

- [ ] Clicking a `mailto:` link when the application is not running launches the application directly into a compose window
- [ ] The compose window is pre-populated with all fields from the URI (To, Subject, Body, CC, BCC)
- [ ] The user does not need to navigate through the main UI to reach the compose window
- [ ] The compose window appears within the application's normal startup time plus one second
- [ ] After dismissing or sending the compose, the user can access the main application normally

## Blocked by

- Blocked by `4-full-field-prepopulation`

## User stories addressed

- US-13 (seamless cold-start compose)

## Type

AFK

## Notes

- The cold-start path is identified in design note N-2 as essential for the feature to feel integrated. This likely requires a dedicated application entry point or startup-mode flag. The exact mechanism is an implementation detail not prescribed by the epic.
