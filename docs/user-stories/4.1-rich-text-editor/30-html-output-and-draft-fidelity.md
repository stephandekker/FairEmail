# HTML Output Serialization and Draft Round-Trip Fidelity

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, I want the editor to produce clean, standards-compliant HTML with inline styles when I send or save a draft, and I want a saved draft to look exactly the same when I reopen it, so that my formatting is faithfully preserved for both recipients and myself.

## Blocked by
`3-bold-italic-underline-strikethrough`, `10-paragraph-alignment`, `11-bullet-lists`, `19-hyperlink-insert-and-edit`

## Acceptance Criteria
- When the message is sent or saved as a draft, the editor's content is serialized to well-formed HTML suitable for email transmission.
- The HTML output uses inline styles rather than external stylesheets, for maximum compatibility with email clients that strip `<style>` blocks.
- The HTML output correctly represents all supported formatting: character styles as appropriate tags or inline CSS, lists as `<ul>`/`<ol>`/`<li>`, block quotes as `<blockquote>`, alignment as CSS text-align, headings as heading tags, etc.
- The HTML output handles bidirectional text correctly, applying appropriate directionality attributes when the content includes right-to-left text.
- Saving a draft with complex formatting (nested lists, colors, fonts, alignment, block quotes) and reopening it reproduces the same visual appearance (round-trip fidelity).
- The HTML output renders acceptably in at least the top five email clients by market share.

## Mapping to Epic
- FR-64, FR-65, FR-66, FR-67
- AC-24
- NFR-3, NFR-4

## HITL / AFK
HITL — HTML output quality and cross-client rendering should be tested against major email clients. This is the story most likely to require iterative QA.

## Notes
- This story is listed last because it integrates the output of all formatting stories. However, in practice, HTML serialization should be built incrementally alongside each formatting feature — not deferred to the end.
- The blocked-by list includes representative formatting stories; in practice, this story's implementation grows as each formatting capability is added.
- NFR-4 (interoperability) is critical: the HTML must render acceptably in Gmail, Outlook, Apple Mail, Yahoo Mail, and Thunderbird at minimum.
- NG-1 clarifies that MIME structure and transmission encoding are outside this epic's scope — this story only covers producing the HTML content.
