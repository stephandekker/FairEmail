# Script and Executable Content Removal

## Parent Feature
#3.6 Safe HTML View

## Blocked by
2-core-sanitization-pipeline

## Description
Extend the sanitization pipeline to completely remove all forms of executable content: `<script>` elements (tag and contents), all `on*` event-handler attributes on every element, and `javascript:` protocol references in any attribute value (notably `href` and `src`).

## Motivation
Script execution is the single most dangerous capability in HTML email. Removing it at the sanitization layer (in addition to the rendering engine lockdown) ensures defense in depth — the content is structurally safe before it reaches any renderer.

## Acceptance Criteria
- [ ] All `<script>` elements are removed entirely (tag and contents, not just the tag).
- [ ] All event-handler attributes (`onclick`, `onload`, `onerror`, `onmouseover`, and every other `on*` attribute) are removed from every element.
- [ ] All `javascript:` protocol references in `href`, `src`, or any other attribute are removed or the attribute is cleared.
- [ ] Obfuscated variants (e.g. `java&#x73;cript:`, mixed-case `JaVaScRiPt:`, whitespace injection) are caught and removed.
- [ ] A test message with script tags, event handlers, and javascript: links renders with all executable content stripped and no script execution occurs.

## HITL/AFK Classification
AFK — fully testable with crafted HTML inputs containing various script injection patterns.

## Notes
- FR-5 through FR-8 govern this story.
- The rendering-engine-level script disable (story 1) is the second line of defense; this story is the primary line.
