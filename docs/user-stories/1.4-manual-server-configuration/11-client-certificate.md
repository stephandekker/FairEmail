# Client Certificate Selection

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add a client certificate selector to the inbound configuration screen. The selector opens the system certificate store and allows the user to choose a certificate for mutual TLS authentication. The selected certificate's name is displayed in the form. A clear action allows removing the selection. The selected certificate is used during connection tests and subsequent connections.

When a client certificate is selected, the password field is no longer required (FR-19).

Covers epic sections: FR-9, FR-19 (partial).

## Acceptance criteria

- [ ] A client certificate selector is present on the inbound configuration form
- [ ] The selector opens the system certificate store for the user to choose a certificate
- [ ] The selected certificate's name is displayed in the form
- [ ] A clear action allows removing the certificate selection
- [ ] The selected certificate is used during connection tests
- [ ] The selected certificate is used during subsequent live connections
- [ ] When a client certificate is selected, the password field is no longer required

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-4 (select client certificate from system store for mutual TLS)
