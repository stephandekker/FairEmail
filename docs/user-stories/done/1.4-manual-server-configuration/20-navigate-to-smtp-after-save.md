# Navigate to SMTP Identity After Inbound Save

## Parent Feature

#1.4 Manual Server Configuration

## What to build

After saving a new inbound account, guide the user to the SMTP identity configuration screen with the new account pre-selected. This creates the natural sequential flow described in the epic: inbound configuration -> save -> outbound (SMTP) identity creation.

The user should have the option to create an identity at save time (e.g. a checkbox or prompt). If opted in, the application navigates to the identity/SMTP screen with the new account pre-selected in the associated account dropdown.

Covers epic sections: FR-43.

## Acceptance criteria

- [ ] After saving a new inbound account, the user is guided to the SMTP identity configuration screen
- [ ] The new account is pre-selected in the identity's associated account dropdown
- [ ] The user has an option to skip identity creation if they choose
- [ ] The SMTP username and password are pre-filled from the inbound account's credentials

## Blocked by

- Blocked by `1-create-imap-account`
- Blocked by `16-smtp-identity-config`

## User stories addressed

- US-23 (guide to outbound identity configuration after saving inbound account)
