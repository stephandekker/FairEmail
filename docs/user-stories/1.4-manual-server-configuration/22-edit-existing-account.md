# Edit Existing Account

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Allow the user to open any existing account's configuration and change any server parameter (host, port, encryption, credentials, certificate, security options, display properties, POP3-specific options). The same form used for creation is reused for editing, with all fields pre-populated from the existing configuration.

The "Test Connection" and "Save" operations are available when editing, with the same validation and test behavior as at creation time. Changing an existing account's server configuration does not discard previously fetched messages or folder state — only connection parameters are updated.

Covers epic sections: FR-59, FR-60, FR-61.

## Acceptance criteria

- [ ] The user can open any existing account's configuration screen
- [ ] All fields are pre-populated with the account's current settings
- [ ] All fields that are configurable at creation are also editable on an existing account
- [ ] "Test Connection" is available and works the same as at creation time
- [ ] "Save" is available and persists changes
- [ ] Changing server configuration does NOT discard previously fetched messages or folder state
- [ ] Only connection parameters are updated; existing data is preserved

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-34 (edit any existing account's configuration)
