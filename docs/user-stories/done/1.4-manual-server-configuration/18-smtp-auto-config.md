# Domain-Based Auto-Config for SMTP

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add a domain input field and "Auto-config" button to the SMTP identity configuration screen, functioning identically to the inbound auto-config (story 14) but discovering SMTP settings. On success, SMTP hostname, port, and encryption mode are populated. All auto-filled fields remain editable. On failure, a clear error message is shown and no fields are modified.

Covers epic sections: FR-49.

## Acceptance criteria

- [ ] A domain input field and "Auto-config" button are present on the SMTP identity configuration screen
- [ ] Auto-config discovers SMTP server settings by domain
- [ ] On success, SMTP hostname, port, and encryption mode are populated
- [ ] All auto-filled fields remain fully editable
- [ ] On failure, a clear error message is shown and no fields are modified
- [ ] Progress indicator and disabled controls during the operation (consistent with inbound auto-config)

## Blocked by

- Blocked by `16-smtp-identity-config`
- Blocked by `14-auto-config-inbound`

## User stories addressed

- US-26 (SMTP auto-config works the same as inbound auto-config)
