## Parent Feature

#1.3 Quick Setup Wizard

## What to build

When the detected provider entry includes an OAuth configuration and OAuth is enabled for that provider, the wizard offers OAuth-based sign-in as an option after detection (FR-39).

**Behavior:**
- After provider detection, if the provider supports OAuth (per FR-15n and FR-40), the wizard presents an OAuth sign-in option alongside or instead of password authentication.
- Known OAuth-supporting providers include at minimum: Gmail, Outlook/Office 365, Yahoo, AOL, Yandex, Mail.ru, and Fastmail (FR-40).
- When OAuth is used, the wizard stores the resulting tokens instead of the user's password and configures the account for token-based authentication (FR-41).
- The OAuth token acquisition flow itself is out of scope for this epic (NG2, epic 1.5). This slice is responsible for triggering the flow and handling the result.

## Acceptance criteria

- [ ] For a provider that supports OAuth and has it enabled, the wizard offers OAuth sign-in after detection (AC-18, FR-39)
- [ ] OAuth is available for Gmail, Outlook/Office 365, Yahoo, AOL, Yandex, Mail.ru, and Fastmail (FR-40)
- [ ] When OAuth is used, tokens are stored instead of the password (FR-41)
- [ ] The account is configured for token-based authentication (FR-41)
- [ ] If the user declines OAuth, password-based authentication remains available

## Blocked by

- Blocked by 13-account-and-identity-creation

## User stories addressed

- US-28 (OAuth sign-in offered for supporting providers)

## Notes

- Open Question OQ-4 in the epic asks about the criteria for promoting OAuth support from debug-only to production. This is unresolved and may affect which providers actually ship with OAuth enabled.
- The OAuth token acquisition, refresh, and storage lifecycle is the concern of epic 1.5. This slice only handles the wizard's role: detecting OAuth availability, triggering the flow, and using the result.
