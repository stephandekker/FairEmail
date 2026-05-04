# Provider Dropdown Selection

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add a provider dropdown to the inbound configuration screen containing known providers from the bundled database, plus a "Custom" option. Selecting a provider pre-fills hostname, port, and encryption mode with that provider's recommended settings. Selecting "Custom" leaves all fields as-is.

When a provider is selected that has provider-specific guidance (e.g. "Use an app-specific password"), that guidance is displayed as contextual help text.

Covers epic sections: FR-29, FR-30, FR-31.

## Acceptance criteria

- [ ] A provider dropdown is present on the inbound configuration screen
- [ ] The dropdown contains known providers from the bundled database plus a "Custom" option
- [ ] Selecting a provider pre-fills hostname, port, and encryption mode
- [ ] Selecting "Custom" leaves all fields as-is (empty for new, unchanged for existing)
- [ ] Pre-filled fields remain editable after provider selection
- [ ] Provider-specific guidance (e.g. "Use an app-specific password") is displayed as contextual help text when applicable

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-11 (provider dropdown with "Custom" option)
- US-12 (provider selection pre-fills port and encryption, still overridable)

## Notes

Open question OQ-7 from the epic: whether the set of providers with special warnings should be enumerable and documented, or treated as an opaque database, is undecided.
