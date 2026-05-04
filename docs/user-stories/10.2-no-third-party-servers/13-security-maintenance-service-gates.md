# Security and Maintenance Service Opt-In Gates (File Scanning, Breach Check, Blocklist Updates, App Updates, Safe Browsing)

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, I want file/attachment scanning, password breach checking, blocklist/filter-list updates, application update checks, and cloud-based safe browsing to each be independently toggleable and disabled by default, so that no security or maintenance feature contacts a third party without my explicit consent.

## Blocked by
- `9-optional-service-gating-framework` (these services use the generic gating framework)

## Acceptance Criteria
- File/attachment scanning services (transmitting file hashes or contents to a cloud scanner) are disabled by default with opt-in disclosure.
- Password breach checking (transmitting a partial hash to a breach-checking service) is disabled by default; activated only by an explicit user action, not automatically.
- Blocklist/filter-list updates from an external source are disabled by default (or bundled with application updates). If fetched separately, opt-in is required.
- Application update checks against external servers are disabled by default (or absent in builds where the system package manager handles updates).
- Cloud-based safe browsing in the built-in message renderer is disabled by default. Enabling requires an explicit setting change with disclosure that URL data will be sent to a third party.
- Each service has its own independent toggle.
- Each toggle appears in the consolidated optional-services view.
- Disabling any of these services takes effect immediately.

## Mapping to Epic
- US-18, US-19
- FR-30, FR-31, FR-32, FR-33, FR-34
- AC-7, AC-13

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- OQ-2 asks whether blocklist updates should be offered as automatic background updates (opt-in) or delivered only through application upgrades. The epic supports both, gated behind a setting. This story implements the gated opt-in approach.
- FR-33 notes that update checks may be "absent entirely in builds where the system package manager handles updates." On Linux desktop, this is the likely path for most distribution channels.
- Password breach checking (FR-31) uses a per-action consent model similar to geolocation (FR-35) — the user triggers it explicitly rather than enabling a persistent background service.
