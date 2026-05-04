## Parent Feature

#4.5 Signature Management

## What to build

Add a raw HTML editing mode to the signature editor, allowing users to directly author or paste arbitrary HTML source. The editor must support toggling between visual (rich text) and raw HTML modes. When in raw HTML mode, content is displayed in a monospaced font (FR-11). Switching between modes preserves the underlying HTML content, with a warning that the visual editor may not render all HTML constructs faithfully (FR-7). An HTML syntax validation function reports parsing errors (FR-10).

Covers epic sections: §6.1 (US-3, US-5, US-7), §7.2 (FR-6, FR-7, FR-10, FR-11).

## Acceptance criteria

- [ ] The signature editor offers a toggle to switch between visual rich text mode and raw HTML mode
- [ ] In raw HTML mode, the content is displayed in a monospaced font (FR-11)
- [ ] Switching from raw HTML to visual mode preserves the HTML content
- [ ] A warning is shown when switching between modes about potential formatting differences (FR-7)
- [ ] An HTML validation function reports syntax/parsing errors before saving (FR-10)
- [ ] Arbitrary HTML pasted into raw mode is preserved faithfully in the outgoing message (AC-12)

## Blocked by

- Blocked by `2-rich-text-signature-editor`

## User stories addressed

- US-3
- US-5
- US-7
