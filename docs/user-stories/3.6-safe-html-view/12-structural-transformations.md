# Structural HTML Transformations

## Parent Feature
#3.6 Safe HTML View

## Blocked by
2-core-sanitization-pipeline

## Description
Implement structural transformations in the sanitization pipeline: convert HTML tables into linear block layouts so table-based email designs reflow at any display width; convert deprecated formatting elements (`<font>`, `<center>`, `<big>`, etc.) to semantically neutral elements with equivalent inline styling; convert address-type elements to actionable links where appropriate.

## Motivation
The vast majority of HTML emails use tables for layout, not data. Converting them to blocks ensures correct reflow without horizontal scrolling. Deprecated elements need modern equivalents to render correctly while remaining on the allowlist.

## Acceptance Criteria
- [ ] HTML tables used for layout are transformed into linear block-level elements that reflow correctly at any display width.
- [ ] `<font>` elements are converted to `<span>` (or equivalent) with corresponding inline styles for color, size, and face.
- [ ] `<center>` elements are converted to a block element with `text-align: center`.
- [ ] `<big>` and similar deprecated sizing elements are converted to spans with appropriate font-size styling.
- [ ] Address-type elements are converted to actionable links where appropriate.
- [ ] A test newsletter with table-based layout renders in a linear, reflowed layout in the reformatted view.
- [ ] A test message with `<font color="red" size="4">` renders with equivalent inline styling.
- [ ] Semantic structure (headings, lists, links) is preserved through transformations.

## HITL/AFK Classification
HITL — table-to-block conversion quality benefits from visual review on real-world newsletter emails to ensure graceful degradation rather than garbled output.

## Notes
- FR-35 through FR-37 govern this story.
- Design Note N-7 explains the table-to-block rationale and the trade-off with genuine data tables.
- OQ-4 flags the open question about detecting data tables vs layout tables — for now, convert all tables and rely on the original view for cases where tabular structure is needed.
- NFR-5 (graceful degradation) is critical here — degraded but readable, not garbled.
