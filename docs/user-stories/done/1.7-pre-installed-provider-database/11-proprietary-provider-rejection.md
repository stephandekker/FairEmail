# User Story: Proprietary Provider Rejection

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **user who enters an email address from a provider that does not support standard protocols** (e.g. a provider using only proprietary encryption without IMAP/SMTP), I want the application to tell me clearly and immediately that this provider is not supported and why, so that I do not waste time troubleshooting a connection that can never work.

This slice delivers proprietary-provider rejection:
- Maintain a list of known email domains that use proprietary protocols and do not support standard IMAP/POP3/SMTP access (FR-36).
- When the user enters a matching email address, display a clear, non-technical error message explaining that this provider does not support standard email protocols (FR-37).
- The rejection occurs early — before any connection attempt — so the user receives immediate feedback (FR-38).

## Acceptance Criteria
- [ ] The application maintains a list of known proprietary-only email domains.
- [ ] Entering an email address from a known proprietary-only provider displays a clear rejection message before any connection is attempted (AC-12).
- [ ] The rejection message explains *why* the provider is unsupported (no standard IMAP/SMTP access) in non-technical language.
- [ ] The rejection does not prevent the user from going back and entering a different email address.
- [ ] The proprietary domain list can be updated as part of routine catalogue maintenance (data change, not code change).

## Blocked by
`1-provider-data-model-and-domain-matching`

## HITL / AFK
**HITL** — The rejection message copy should be reviewed for tone and clarity. It must be non-technical and non-blaming.

## Notes
- The existing Android app hardcodes proprietary providers (ProtonMail, Tutanota, Skiff, Ctemplar, Criptext, etc.) in `EmailProvider.java`. The epic's design note N-5 emphasises that this should be an explicit, honest rejection rather than a silent failure.
- Uncertainty: the epic does not specify whether the proprietary domain list should live inside the provider catalogue (as entries with a special flag) or as a separate blocklist. Either approach satisfies the requirements. Using catalogue entries with a "proprietary" flag would be more consistent with the data-driven philosophy (Design Note N-3). Using a separate list keeps the catalogue focused on supported providers. The implementer should choose and document the approach.
