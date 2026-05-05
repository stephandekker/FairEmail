# SMTP Test Connection

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add a "Test Connection" button to the SMTP identity configuration screen. When pressed, the application connects to the SMTP server using the entered parameters, authenticates, queries the server's advertised maximum message size, and reports success or failure.

On success, if the server reports a maximum message size, that value is displayed and optionally stored as the identity's size limit.

The test shows a progress indicator and disables input fields during the operation, consistent with the inbound test behavior. Error diagnostics follow the same categorization as inbound test errors.

Covers epic sections: FR-50, FR-51.

## Acceptance criteria

- [ ] A "Test Connection" button is present on the SMTP identity configuration screen
- [ ] The test connects to the SMTP server, authenticates, and reports success or failure
- [ ] On success, the server's advertised maximum message size is displayed (if available)
- [ ] The maximum message size is optionally stored as the identity's size limit
- [ ] A progress indicator is shown during the test
- [ ] Input fields are disabled during the test
- [ ] On failure, actionable error messages are displayed (same categories as inbound errors)

## Blocked by

- Blocked by `16-smtp-identity-config`

## User stories addressed

- US-27 (SMTP test connection reports max message size)
