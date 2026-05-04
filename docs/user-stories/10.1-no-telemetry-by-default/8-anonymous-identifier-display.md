# Anonymous Identifier Display in UI

## Parent Feature
#10.1 No Telemetry by Default

## User Story
As a user who has opted in, I want to be able to view my anonymous identifier within the application, so that I can reference it if I ever want to request manual deletion of my reports or communicate with the developer about a specific issue.

## Acceptance Criteria
- [ ] When error reporting is enabled, the user's anonymous identifier is visible in a diagnostic or "about" screen.
- [ ] The displayed identifier matches the identifier attached to transmitted error reports.
- [ ] When error reporting is disabled, the identifier need not be displayed (since no reports are being sent).
- [ ] The identifier display is easy to find for a user who is looking for it (e.g. in an "About" or "Debug info" section of settings).

## Complexity Estimate
Small

## Blocked by
4-anonymous-identifier-generation
5-error-report-capture-and-transmission

## Notes
- This story is deliberately separated from identifier generation (story 4) because it touches the UI layer, while generation is a data/storage concern. Together they form the full identifier feature.
- This story covers FR-18, FR-19, US-11, AC-7.
