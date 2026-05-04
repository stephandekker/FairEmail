# Create IMAP Account with Basic Connection Settings

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to add a new IMAP mail account by providing server connection details and a display name, so that I can begin receiving and sending mail through that account.

## Blocked by
*(none — this is the foundational slice)*

## Acceptance Criteria
- User can open an "Add account" flow and select IMAP as the protocol.
- User can enter: server host, port, encryption mode (SSL/TLS, STARTTLS, or none), authentication method, username, and password or token.
- User can enter a display name for the account.
- On save, the account is persisted with a globally-unique, stable identifier (FR-2).
- The new account appears in the account list and the navigation pane.
- No application-imposed limit prevents adding the account (FR-1).
- The operation is atomic — either all account data is persisted or none is (NFR-3).
- Account creation works while offline; connection-dependent steps (folder discovery) fail gracefully with a clear message (NFR-6).
- All controls are keyboard-accessible with screen-reader labels (NFR-7).

## Mapping to Epic
- US-1, US-2 (IMAP path), US-3
- FR-1, FR-2, FR-3, FR-16
- NFR-3, NFR-6, NFR-7
- AC-1 (partially — connection test is a separate story)

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story deliberately excludes connection testing (story 2), POP3 (story 4), color/avatar (stories 6–7), and all optional/advanced fields. It establishes the minimal account entity and persistence path.
- The epic specifies "IMAP or POP3" but this story only covers the IMAP path to keep the slice thin. POP3 follows in story 4.
