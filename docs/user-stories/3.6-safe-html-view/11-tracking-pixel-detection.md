# Tracking Pixel Detection and Replacement

## Parent Feature
#3.6 Safe HTML View

## Blocked by
8-image-blocking-and-placeholders

## Description
Extend the sanitization pipeline to detect likely tracking pixels using multiple heuristics (zero/near-zero dimensions, tiny computed surface area, CSS max-dimension constraints, known-tracker domain lists, known tracking CSS class names) and replace them with a visible indicator icon and descriptive alt text. Preserve the original image source in a non-rendering attribute for user inspection. Provide a user preference to disable tracking-pixel detection.

## Motivation
Tracking pixels are invisible images that report message opens to senders. Detecting and visibly replacing them (rather than silently removing) informs the user that tracking was attempted, supporting informed trust decisions about senders.

## Acceptance Criteria
- [ ] Images with declared width or height of zero or near-zero (a few pixels or less) are detected as tracking pixels.
- [ ] Images with computed surface area below a small threshold are detected.
- [ ] Images with CSS constraining max dimensions to 1px or less are detected.
- [ ] Images hosted on domains in a known-tracker list are detected.
- [ ] Images with CSS class names associated with known tracking beacons are detected.
- [ ] Detected tracking pixels are replaced with a visible indicator icon and alt text (e.g. "Tracking image").
- [ ] The original image URL is preserved in a non-rendering attribute for user inspection.
- [ ] A user preference controls whether tracking-pixel detection is enabled (default: on).
- [ ] When detection is disabled, tiny images are treated as regular images (subject to normal image blocking rules).
- [ ] Images whose declared dimensions are unreasonably small are removed entirely as likely tracking constructs (even when detection is "disabled", per FR image-size rules from story 8).

## HITL/AFK Classification
AFK — testable with crafted HTML containing various tracking pixel patterns and known-tracker domains.

## Notes
- FR-25 through FR-28 govern this story.
- Design Note N-8 explains the replacement-not-removal approach.
- The tracker domain blocklist itself is managed by feature 3.11 (NG4); this story only consumes the list.
- There is a subtlety: FR-24 (images with unreasonably small dimensions removed entirely) in story 8 vs FR-26 (tracking pixels replaced with indicator) here. The distinction: story 8 removes images that are dimensionally suspicious regardless of detection preference; this story adds the visible indicator and domain/class-based detection on top.
