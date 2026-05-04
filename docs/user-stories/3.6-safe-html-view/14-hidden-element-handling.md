# Hidden Element Handling

## Parent Feature
#3.6 Safe HTML View

## Blocked by
5-css-property-allowlist

## Description
Implement handling of hidden elements in the sanitization pipeline. By default, elements with `display: none`, `visibility: hidden`, or equivalent styling remain hidden in the rendered output. A user preference (default: off) reveals hidden elements with a visual distinction (e.g. strikethrough, muted styling, or a border) so users can inspect concealed content such as preheader text, hidden tracking markup, or concealed phishing content.

## Motivation
Senders use hidden elements for legitimate purposes (preheader text) and malicious ones (hidden tracking/phishing content). The default preserves sender intent, while the reveal preference empowers privacy-conscious users to inspect what's being hidden.

## Acceptance Criteria
- [ ] Elements with `display: none` remain hidden by default in the rendered output.
- [ ] Elements with `visibility: hidden` remain hidden by default.
- [ ] A user preference (default: off) controls hidden-element revelation.
- [ ] When the preference is enabled, hidden elements are rendered visibly with a distinct visual treatment that differentiates them from normal content.
- [ ] The visual distinction is clear enough that users can tell which content was originally hidden.
- [ ] A test message with hidden preheader text hides it by default and reveals it (with distinction) when the preference is on.

## HITL/AFK Classification
AFK — testable with HTML containing hidden elements and verifying visibility behavior per preference state.

## Notes
- FR-38 and FR-39 govern this story.
- OQ-5 flags the open question about exact allowed values of `display` and `visibility` and their interaction with this preference — document any decisions made during implementation.
