# SMTP Identity Configuration — Basic Fields

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Create the SMTP identity configuration screen with the following fields: hostname, port, encryption mode (SSL/TLS / STARTTLS / None), username, password, client certificate (optional), authentication realm (optional), email address, display name, and associated inbound account (dropdown of existing accounts).

SMTP port auto-fill follows the same convention as inbound but with SMTP-specific defaults: SSL/TLS -> 465, STARTTLS -> 587, None -> 25. Port auto-fill uses the same smart heuristic (only replace known defaults or empty values).

The SMTP username and password default to the associated inbound account's credentials but are independently editable.

Encryption mode, security options (insecure, DANE), client certificate selection, and validation rules follow the same conventions as the inbound configuration.

Covers epic sections: FR-45 through FR-49.

## Acceptance criteria

- [ ] An SMTP identity configuration screen presents fields for: hostname, port, encryption mode, username, password, client certificate, realm, email address, display name, and associated account
- [ ] Encryption mode offers SSL/TLS, STARTTLS, and None
- [ ] Port auto-fills based on encryption: SSL/TLS -> 465, STARTTLS -> 587, None -> 25
- [ ] Port auto-fill uses the smart heuristic (does not overwrite non-default user-entered ports)
- [ ] SMTP username and password default to the associated inbound account's credentials
- [ ] SMTP username and password are independently editable (can differ from inbound)
- [ ] Associated account is selectable from a dropdown of existing inbound accounts
- [ ] The identity can be saved, persisting all fields

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-24 (configure SMTP independently from inbound)
- US-25 (SMTP port defaults)
- US-28 (identity form collects email, display name, account association)
- US-29 (SMTP credentials default to inbound, independently overridable)
