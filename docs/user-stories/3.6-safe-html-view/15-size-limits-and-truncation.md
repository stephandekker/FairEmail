# Size Limits and Truncation

## Parent Feature
#3.6 Safe HTML View

## Blocked by
2-core-sanitization-pipeline

## Description
Enforce maximum input size limits in the sanitization pipeline. Messages exceeding the reformatted-view threshold are truncated with a visible indicator appended to the output. A separate, larger threshold applies to the original view. This prevents maliciously or accidentally oversized messages from consuming unbounded memory or freezing the display.

## Motivation
Without size limits, a single maliciously crafted message could cause excessive memory consumption or freeze the rendering engine. Bounded resource use is a core non-functional requirement of the pipeline.

## Acceptance Criteria
- [ ] The sanitization pipeline enforces a maximum input size for the reformatted view (approximately 100 KB per OQ-3).
- [ ] Messages exceeding the reformatted-view limit are truncated at that threshold.
- [ ] A visible truncation indicator is appended to the rendered output when truncation occurs, informing the user that content was cut.
- [ ] A separate, larger maximum (approximately 1 MB per OQ-3) applies to the original view.
- [ ] Truncation does not produce malformed HTML (the output remains well-formed after truncation).
- [ ] A test message exceeding the size limit displays truncated content with the indicator visible.

## HITL/AFK Classification
AFK — testable with oversized HTML inputs and verifying truncation behavior.

## Notes
- FR-40 through FR-42 govern this story.
- OQ-3 flags that the exact thresholds (100 KB / 1 MB) and whether they should be user-configurable are unresolved. Implement the specified defaults; configurability can be added later if needed.
- NFR-2 (bounded resource use) is the governing non-functional requirement.
