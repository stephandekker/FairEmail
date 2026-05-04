# User Story: Provider Documentation and Guidance

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **user setting up an account with a provider that has specific requirements**, I want to see provider-specific setup instructions, links to documentation, and actionable guidance (e.g. "you need to enable IMAP access" or "you need an app-specific password"), so that I can resolve issues without contacting support or troubleshooting blind.

This slice delivers all provider documentation and guidance surfaces:
- Display external documentation link from a provider entry (FR-33, US-14).
- Display inline setup documentation as formatted text, with locale-specific variants — show the user's locale variant, falling back to default (FR-34, US-15, NFR-4).
- Display registration/sign-up URL when present (FR-35, US-19).
- When a provider is flagged as requiring manual IMAP enablement, display a notification at setup time (US-12, AC-11).
- When a provider is flagged as requiring an app-specific password, display guidance with a link to the provider's app-password page (US-11, AC-10).

## Acceptance Criteria
- [ ] When a matched or selected provider has a documentation URL, it is displayed to the user (e.g. as a clickable link).
- [ ] When a matched or selected provider has inline setup documentation, it is displayed as formatted text.
- [ ] Inline documentation is shown in the user's locale when a translation is available; falls back to the default language otherwise (AC-14).
- [ ] When a provider has a registration/sign-up URL, it is surfaced in the provider's detail view or selection UI.
- [ ] When a provider is flagged as requiring an app password, the setup flow displays guidance about app passwords, ideally with a link to the provider's app-password management page (AC-10).
- [ ] When a provider is flagged as requiring manual IMAP/SMTP enablement, the setup flow displays a notification about this requirement (AC-11).

## Blocked by
`4-browsable-provider-list`, `9-behavioural-overrides`

## HITL / AFK
**HITL** — The wording and placement of guidance messages (app-password prompts, IMAP-enablement notices) affects user experience. Implementation can proceed with reasonable defaults, but the copy should be reviewed.

## Notes
- The existing Android app stores inline documentation as HTML fragments within `<documentation>` elements in `providers.xml`, with `lang` attributes for locale variants. The desktop app should support equivalent locale-specific documentation, though the storage format may differ.
