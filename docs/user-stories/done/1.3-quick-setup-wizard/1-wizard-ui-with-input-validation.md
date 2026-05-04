## Parent Feature

#1.3 Quick Setup Wizard

## What to build

The foundational wizard screen that a user sees when setting up an account. This slice delivers the complete input UI and all client-side validation, wired into both entry points (first-launch and add-account).

**Entry points:**
- On first launch with no configured accounts, the application presents the wizard as the default and only initial screen (FR-1).
- When the user initiates "Add account" from settings or navigation, the wizard is the default path (FR-2).

**Input fields (single screen):**
- Display name, email address, and password (FR-3).
- Password field supports show/hide toggle (FR-4).

**Validation (before any network operation):**
- Display name is non-empty (FR-5a).
- Email address is non-empty and matches a standard email pattern (FR-5b).
- Password is non-empty (FR-5c).
- If password contains leading/trailing whitespace or non-printable characters, display a warning (not a blocking error) (FR-6).

**Network pre-check:**
- The wizard requires network connectivity before starting the check and displays a clear message if the network is unavailable (FR-7).

This slice does NOT include provider detection, connectivity checks, or account creation. The "Check" button exists but is wired only to validation and the network-availability gate.

## Acceptance criteria

- [ ] On first launch with zero accounts, the wizard screen is shown automatically (AC-14, FR-1)
- [ ] "Add account" from settings/navigation opens the wizard as the default path (FR-2)
- [ ] Three input fields (name, email, password) are present on a single screen (FR-3)
- [ ] Password field has a working show/hide toggle (FR-4)
- [ ] Submitting with an empty name produces an immediate, field-specific error (AC-14)
- [ ] Submitting with an invalid email produces an immediate, field-specific error (AC-14)
- [ ] Submitting with an empty password produces an immediate, field-specific error (AC-14)
- [ ] A password with leading whitespace triggers a visible warning (AC-15, FR-6)
- [ ] A password with trailing whitespace triggers a visible warning (FR-6)
- [ ] A password with non-printable characters triggers a visible warning (FR-6)
- [ ] The warning for whitespace/non-printable characters is non-blocking (user can proceed) (FR-6)
- [ ] If the network is unavailable when the user clicks Check, a clear message is shown (FR-7)
- [ ] All wizard elements are keyboard-navigable and screen-reader accessible (NFR-8)

## Blocked by

None - can start immediately

## User stories addressed

- US-1 (first launch presents wizard)
- US-2 (add account reaches wizard)
- US-3 (name, email, password on single screen)
- US-4 (password show/hide toggle)
- US-5 (whitespace/control character warning)
- US-6 (field validation before network check)
