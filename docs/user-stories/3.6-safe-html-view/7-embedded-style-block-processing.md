# Embedded Style Block Processing

## Parent Feature
#3.6 Safe HTML View

## Blocked by
5-css-property-allowlist, 6-remote-font-and-stylesheet-blocking

## Description
Implement parsing and application of embedded `<style>` blocks. When enabled (default: on), parse each CSS rule in embedded style blocks, validate each property against the same CSS allowlist used for inline styles, and apply the surviving rules to matching elements as inline styles. When disabled via user preference, remove embedded `<style>` blocks entirely and apply no class-based styling.

## Motivation
Many modern HTML emails rely entirely on class-based styling in `<style>` blocks rather than inline styles. Without this processing, most newsletter and transactional emails would be unreadable. Safety comes from the property-level allowlist, not from ignoring classes.

## Acceptance Criteria
- [ ] Embedded `<style>` blocks are parsed and CSS rules are resolved against the property allowlist.
- [ ] Only allowlisted properties survive in the resolved rules.
- [ ] Resolved rules are applied to matching elements (by class, tag, or other selectors) as inline styles.
- [ ] When the class-based CSS parsing preference is disabled, `<style>` blocks are removed entirely and no class-based styling is applied.
- [ ] `@import` and `@font-face` rules within style blocks are stripped (per story 6) before processing.
- [ ] A test newsletter email with class-based styling renders correctly with styles applied when the preference is on.
- [ ] The same email renders as unstyled (but readable) content when the preference is off.

## HITL/AFK Classification
AFK — testable with HTML emails that use class-based styling.

## Notes
- FR-19 and FR-20 govern this story.
- Design Note N-6 explains why class parsing is on by default.
- Complex selectors (e.g. descendant combinators, pseudo-classes) may need to be handled or explicitly excluded — the epic does not specify selector complexity limits. Flag in implementation if certain selector types are problematic.
