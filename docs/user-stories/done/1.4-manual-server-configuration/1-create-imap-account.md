# Create IMAP Account with Basic Fields and Save

## Parent Feature

#1.4 Manual Server Configuration

## What to build

The foundational vertical slice: a configuration screen where the user selects IMAP as the inbound protocol and enters the minimum required fields — hostname, port, encryption mode (SSL/TLS, STARTTLS, None), username, and password — then saves the account. On save the account is persisted with all connection settings and appears in the application's navigation/account list, ready for synchronization.

This slice establishes the account creation form, the persistence layer for account records, and the basic UI flow. It does NOT include test-connection, auto-config, provider dropdown, security options, display properties, or SMTP identity — those are layered on in subsequent stories.

Covers epic sections: FR-1 (IMAP protocol), FR-4 (inbound fields — host, port, encryption, username, password only), FR-5 (three encryption modes), FR-41 (save persists settings), FR-58 (display name defaults to username/email).

The password field must include a visibility toggle per FR-8. The "None" encryption mode must carry a prominent visual warning per FR-16.

## Acceptance criteria

- [ ] User can open a "new account" screen and select IMAP as the inbound protocol
- [ ] The form presents fields for: hostname, port, encryption mode (SSL/TLS / STARTTLS / None), username, and password
- [ ] Encryption mode defaults to SSL/TLS per NFR-2
- [ ] The password field has a visibility toggle (show/hide) per FR-8
- [ ] Selecting "None" encryption displays a prominent visual warning per FR-16
- [ ] User can press "Save" to persist the account
- [ ] After saving, the account appears in the application's navigation/account list
- [ ] The account's display name defaults to the username or email address per FR-58

## Blocked by

None - can start immediately

## User stories addressed

- US-1 (create account with protocol, host, port, encryption, username, password)
- US-7 (account display name — default only; color/category in a later story)
