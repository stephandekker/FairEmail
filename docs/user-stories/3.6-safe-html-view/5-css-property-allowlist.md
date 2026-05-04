# CSS Property Allowlist for Inline Styles

## Parent Feature
#3.6 Safe HTML View

## Blocked by
2-core-sanitization-pipeline

## Description
Implement CSS property filtering for inline `style` attributes. Only properties on a defined allowlist are preserved; all others are silently removed. Additionally, strip all `!important` flags from property values. The allowlist covers: color, background-color/background, font-size, font-family, text-align, font-weight, font-style, text-decoration (limited to line-through and underline), text-transform, display/visibility (limited values), margin/padding (with value validation), border (left/right only), list-style-type, and white-space (limited to pre/pre-wrap).

## Motivation
CSS is a powerful attack surface — properties like `position`, `z-index`, `overflow`, and many others can overlay content, leak information, or disrupt the UI. An allowlist approach ensures only known-safe properties pass through, and `!important` stripping prevents message styles from overriding application UI.

## Acceptance Criteria
- [ ] Inline `style` attributes are parsed and only allowlisted CSS properties are preserved.
- [ ] Non-allowlisted properties (e.g. `position: absolute`, `z-index: 9999`, `overflow: hidden`) are silently removed.
- [ ] `!important` flags are stripped from all preserved property values.
- [ ] `text-decoration` values are limited to `line-through` and `underline`; other values are removed.
- [ ] `display` and `visibility` values are validated against a limited set.
- [ ] `white-space` is limited to `pre` and `pre-wrap`.
- [ ] Margin/padding values are validated (no negative values or extreme values that could cause layout issues).
- [ ] Border properties are limited to left and right borders only.
- [ ] A test message with a mix of safe and unsafe CSS properties renders with only safe properties applied.

## HITL/AFK Classification
AFK — comprehensive test suite with various CSS property combinations.

## Notes
- FR-16 through FR-18 govern this story.
- OQ-1 (allowlist completeness) is an open question in the epic — the initial allowlist should match what's specified, with a note that it may be expanded after review.
- The toggleable nature of some properties (color, background, font-size, font-family, text-align) is handled in story 9 (selective-styling-controls), not here. This story implements the base filtering mechanism.
