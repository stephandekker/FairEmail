# Viewport Normalization

## Parent Feature
#3.6 Safe HTML View

## Blocked by
2-core-sanitization-pipeline

## Description
Extend the sanitization pipeline to normalize or override viewport meta tags embedded in message HTML. Remove or override restrictive viewport attributes (minimum/maximum scale locks, `user-scalable=no`) so the user can always zoom and scroll freely regardless of what the sender specified.

## Motivation
Some HTML emails include viewport meta tags that lock zoom or disable scrolling — originally intended for mobile webviews. On a desktop email client, this would prevent the user from zooming into small text or scrolling through content. The pipeline must guarantee user control.

## Acceptance Criteria
- [ ] Viewport meta tags in message HTML are normalized to permit user zooming and scrolling.
- [ ] `user-scalable=no` is removed or overridden to allow scaling.
- [ ] Minimum-scale and maximum-scale locks are removed or overridden.
- [ ] After sanitization, the user can always zoom and scroll the rendered message freely.
- [ ] A test message with `<meta name="viewport" content="user-scalable=no, maximum-scale=1">` renders with zooming enabled.

## HITL/AFK Classification
AFK — testable by verifying the sanitized HTML output and confirming zoom/scroll behavior.

## Notes
- FR-43 and FR-44 govern this story.
