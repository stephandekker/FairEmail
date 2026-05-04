# Password-Protected Content

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a privacy-conscious sender, I want to select text and encrypt it with a password, replacing the visible text with a decryption link, so that only the intended recipient who knows the password can read it.

## Blocked by
`1-wysiwyg-editor-surface`

## Acceptance Criteria
- A "protect with password" action is accessible from the toolbar or a menu.
- The user selects text, invokes the action, and is prompted to enter a password.
- On confirmation, the selected content is replaced with an encrypted payload or link that the recipient can decrypt by entering the correct password.
- Decryption is available to any recipient without requiring a paid license.
- If the selected content exceeds approximately 1,500 characters (including formatting markup), the user is warned before proceeding.
- The encrypted payload persists after saving as a draft and reopening.

## Mapping to Epic
- US-39, US-40
- FR-61, FR-62, FR-63
- AC-22, AC-23
- OQ-3, N-7

## HITL / AFK
HITL — the encryption mechanism, payload format, and size limit (OQ-3) need design decisions. N-7 clarifies this is *not* end-to-end encryption but a convenience feature for embedding a single sensitive datum.

## Notes
- OQ-3 flags a discrepancy in the source application: the size limit is approximately 1,500 characters in some code paths and approximately 5,000 in others. A single authoritative limit must be established before implementation.
- N-7 clarifies that this is not a substitute for PGP/S/MIME. It exists for the use case of sending a single sensitive piece of information (a password, an account number) within an otherwise normal message.
- The encryption/decryption mechanism is not specified by the epic. Design must decide on a standard approach (e.g. AES with a key derived from the password) and a delivery mechanism (e.g. a self-contained HTML snippet or a link to a decryption page).
