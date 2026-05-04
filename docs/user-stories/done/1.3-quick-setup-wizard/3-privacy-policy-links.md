## Parent Feature

#1.3 Quick Setup Wizard

## What to build

Display privacy policy links on the wizard's initial screen before the user initiates any action. This ensures the user is informed about data sharing before they enter credentials or trigger detection.

**Required links (FR-37):**
- The application's own privacy policy.
- The privacy policy of any third-party autoconfig service used during detection (e.g. Thunderbird ISPDB / Mozilla Privacy Policy).

**Security guarantee (FR-38):**
- The wizard never transmits the user's password to any third-party service. Passwords are sent only to the user's own mail server during the connectivity check. This guarantee should be documented/visible in the privacy information.

## Acceptance criteria

- [ ] Privacy policy links are visible on the wizard's initial screen before the user initiates any action (AC-17, FR-37)
- [ ] A link to the application's privacy policy is present (FR-37)
- [ ] A link to the third-party autoconfig service's privacy policy is present (FR-37)
- [ ] Links are accessible via keyboard and screen reader (NFR-8)

## Blocked by

- Blocked by 1-wizard-ui-with-input-validation

## User stories addressed

- US-27 (privacy policy links displayed before user begins)
