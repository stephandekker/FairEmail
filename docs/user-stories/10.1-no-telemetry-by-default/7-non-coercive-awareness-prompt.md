# Non-Coercive One-Time Awareness Prompt

## Parent Feature
#10.1 No Telemetry by Default

## User Story
As any user, when I encounter an error and error reporting is not enabled, I want the application to briefly inform me that error reporting exists and could help improve the application, without being coercive or repetitive, so that I am aware of the option but not pressured into it.

## Acceptance Criteria
- [ ] When the application encounters an error while error reporting is disabled, it may display a non-modal, dismissible notification informing the user that error reporting is available.
- [ ] The notification includes a way to access more information (what data is sent, how it is anonymized, how long it is retained) and a way to enable reporting directly.
- [ ] The notification never blocks the user's workflow or requires interaction before the user can continue.
- [ ] If the user dismisses or declines the notification, the application records this and never displays the notification again.
- [ ] After declining, the prompt does not appear again regardless of how many subsequent errors occur.

## Complexity Estimate
Medium

## Blocked by
2-local-crash-error-logging
3-error-reporting-preference-toggle

## Notes
- Epic open question OQ-3 asks whether to prompt on any error or only specific classes of errors. This story assumes prompting on the first qualifying error only (since the prompt is shown at most once, the trigger condition matters only for timing). The implementer should choose a trigger that is likely to occur naturally (e.g. a visible error in the main UI) rather than an obscure edge case the user might never hit.
- Per epic design note N-3: single ask, then silence. This is deliberate — avoid the "nag screen" pattern.
- The informational content shown when the user requests "more information" should cover: what data is sent, how it is anonymized, and the retention period — satisfying US-7.
- This story covers FR-8, FR-9, FR-10, FR-11, US-5, US-6, US-7, AC-5.
