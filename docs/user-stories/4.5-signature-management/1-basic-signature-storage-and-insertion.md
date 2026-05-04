## Parent Feature

#4.5 Signature Management

## What to build

The foundational tracer bullet: allow each identity to store a plain-text signature and automatically insert it into new compose messages. This slice wires up the full vertical path — identity data model (signature field), a minimal plain-text signature editor accessible from identity settings, and the compose-window logic that inserts the signature into the message body when composing a new message.

The signature is per-identity (FR-1, FR-3) and editable from identity settings (FR-2). On this slice the editor is plain-text only; rich text and HTML editing come in later slices. The signature is inserted at the default "below the text" position (FR-21) with no placement options exposed yet.

Covers epic sections: §6.1 (US-1, partially), §7.1 (FR-1 – FR-3), §7.5 (FR-21 default only).

## Acceptance criteria

- [ ] Each identity has a signature field that persists across application restarts
- [ ] The identity settings screen includes a way to open a signature editor
- [ ] The signature editor allows entering and saving plain-text content
- [ ] Composing a new message with an identity that has a signature inserts that signature into the message body below the user's text area
- [ ] An identity with an empty signature produces a compose message with no signature block
- [ ] Changing one identity's signature does not affect any other identity's signature (FR-3)

## Blocked by

None — can start immediately.

## User stories addressed

- US-1 (partially — plain text only, rich text in later slices)
