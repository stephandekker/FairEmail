## Parent Feature

#4.5 Signature Management

## What to build

Replace the plain-text signature editor with a visual rich text editing surface that supports formatting (bold, italic, underline, links, colors, fonts). The editor re-uses the application's standard rich text editing component (per NG-2 — the toolbar itself is defined in epic 4.1). This slice also adds a full-screen preview of the rendered signature (FR-8).

The signature content is now stored as HTML (FR-1). When inserted into a compose message, the HTML is rendered faithfully in the message body (NFR-2).

Covers epic sections: §6.1 (US-2, US-4), §7.2 (FR-5, FR-8).

## Acceptance criteria

- [ ] The signature editor provides a visual rich text editing mode with formatting controls (bold, italic, underline, links, colors, fonts)
- [ ] Signature content is stored as HTML
- [ ] A full-screen preview shows the rendered signature as it will appear in messages
- [ ] A signature with formatting (e.g. bold text, a hyperlink) is faithfully included in the outgoing message HTML (AC-1, partially)

## Blocked by

- Blocked by `1-basic-signature-storage-and-insertion`

## User stories addressed

- US-2
- US-4
