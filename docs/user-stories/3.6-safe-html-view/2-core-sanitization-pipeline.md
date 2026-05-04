# Core Sanitization Pipeline (Tag & Attribute Allowlist)

## Parent Feature
#3.6 Safe HTML View

## Blocked by
1-rendering-engine-lockdown

## Description
Implement the foundational sanitization pipeline that parses raw HTML email into a DOM, walks the tree, and filters it against an allowlist of permitted tags and permitted attributes per tag. Tags not on the allowlist are removed (preserving inner text where appropriate). Attributes not on the allowlist are silently removed. The pipeline outputs a clean HTML document ready for further processing stages.

## Motivation
This is the backbone of the safe HTML view. All subsequent sanitization stories (script removal, form neutralization, CSS filtering, etc.) layer on top of this core allowlist-based filtering. Without it, no other sanitization can occur.

## Acceptance Criteria
- [ ] Raw HTML email is parsed into a DOM structure (not processed as raw text/regex).
- [ ] Only tags on a defined allowlist survive the pipeline; all others are removed.
- [ ] When a non-allowed tag is removed, its text content is preserved (e.g. `<blink>hello</blink>` becomes `hello`).
- [ ] Only attributes on a per-tag allowlist survive; all others are silently stripped.
- [ ] The pipeline produces valid, well-formed HTML output.
- [ ] Malformed/encoding-tricked HTML is handled correctly by the DOM parser (no bypass via unclosed tags, null bytes, etc.).
- [ ] The sanitized output is what reaches the rendering engine in the default (reformatted) view — no unsanitized HTML is ever passed to the renderer in this mode.
- [ ] Semantic structure (headings, paragraphs, lists, links, alt text on images) is preserved through sanitization.

## HITL/AFK Classification
AFK — automated tests can verify allowlist behavior with a comprehensive set of test HTML inputs.

## Notes
- FR-1 through FR-4 are the governing requirements.
- NFR-1 (performance < 1s for typical messages) and NFR-2 (bounded resource use) apply here.
- The specific allowlist contents will be refined as subsequent stories (CSS, forms, scripts) are implemented, but the mechanism must be in place here.
