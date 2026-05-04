# Remote Font and External Stylesheet Blocking

## Parent Feature
#3.6 Safe HTML View

## Blocked by
5-css-property-allowlist

## Description
Extend the sanitization pipeline to block all remote font and stylesheet loading: remove `<link rel="stylesheet">` elements entirely, strip `@import` rules from embedded style blocks, and strip `@font-face` rules. Preserve `font-family` declarations that reference locally available fonts (subject to the user's font-family preference toggle).

## Motivation
Remote fonts and stylesheets leak the user's IP address to third-party servers and can be used for fingerprinting. Blocking them at the sanitization layer ensures no network request is made for fonts or external CSS, while still allowing locally installed fonts to be used for visual fidelity.

## Acceptance Criteria
- [ ] `<link rel="stylesheet">` elements are removed entirely from the output.
- [ ] `@import` rules within embedded `<style>` blocks are removed and not followed.
- [ ] `@font-face` rules within embedded `<style>` blocks are removed.
- [ ] `font-family` declarations referencing local fonts are preserved in inline styles.
- [ ] No network request is made to load a remote font or stylesheet when viewing a sanitized message.
- [ ] A test message referencing Google Fonts via `<link>` and `@font-face` renders using local fallback fonts with no remote requests.

## HITL/AFK Classification
AFK — testable by verifying output HTML and monitoring for network requests.

## Notes
- FR-12 through FR-15 govern this story.
- The font-family preference toggle (on/off) is part of story 9 (selective-styling-controls); this story just ensures the blocking mechanism works and preserves local font references by default.
