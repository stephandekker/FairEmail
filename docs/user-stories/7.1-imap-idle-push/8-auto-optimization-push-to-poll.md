# Auto-Optimization: Push-to-Poll Fallback

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, when my server's IDLE behavior indicates poor support (frequent "still here" messages or repeated connection drops despite tuning), I want the application to automatically switch that account to polling and inform me, so that I get reliable mail delivery even from poorly-behaved servers.

## Blocked by
- `6-keep-alive-auto-tuning`
- `3-poll-mode-fallback`

## Acceptance Criteria
- If the server sends unsolicited "still here" notifications within 3 minutes of entering IDLE, the application treats this as evidence of poor IDLE support (FR-23, Design Note N-4).
- If IDLE repeatedly fails to maintain a stable connection despite keep-alive tuning, or the keep-alive interval has been reduced below a viability threshold (e.g. 12 minutes), the application switches the account to polling (FR-24).
- When auto-optimization triggers, the application: (a) switches the account to polling at a 15-minute default interval, (b) logs the reason, and (c) surfaces a notification or status indicator to the user (FR-25, AC-8).
- Auto-optimization is a per-account decision — folders on other accounts are unaffected (FR-27).
- On an account where the server sends "still here" within 3 minutes of IDLE start, auto-optimization triggers within one connection cycle (AC-8).
- The user can override auto-optimization and force push mode for any account (FR-26, AC-9).
- When forced, the application respects the override and does not re-trigger auto-optimization unless the user removes the override (AC-9).

## Mapping to Epic
- US-15, US-16, US-17
- FR-23, FR-24, FR-25, FR-26, FR-27
- AC-8, AC-9

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- The 3-minute "still here" threshold and 12-minute viability threshold are empirically chosen values from the source application (Design Note N-4). These should be constants that can be adjusted.
- Open Question OQ-6 (Outlook IDLE bug): the epic asks whether a server-behavior quirks database should be maintained. This story implements the generic auto-optimization path; provider-specific pre-emptive defaults are story 17.
