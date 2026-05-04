# Image Blocking and Placeholders

## Parent Feature
#3.6 Safe HTML View

## Blocked by
2-core-sanitization-pipeline

## Description
Implement remote image blocking in the safe view: remote images are not loaded by default, with placeholders shown in their place. Inline (embedded/cid:) images are displayable via a user preference (default: off). Images with empty `src` attributes are removed. Data-URI images (base64-encoded) are allowed through. CSS `background-image` declarations with URL references are converted to regular image elements and subjected to the same blocking rules.

## Motivation
Remote images leak the user's IP address and confirm message opens. Blocking them by default with visible placeholders lets the user know content was blocked without making network requests. Converting background-image tricks to regular images prevents CSS-based bypasses of image blocking.

## Acceptance Criteria
- [ ] Remote images (http/https src) are not loaded in the safe view by default.
- [ ] When remote images are blocked and placeholders are enabled (default: on), a placeholder element is shown in place of each blocked image.
- [ ] Inline/embedded images (cid: references) are hidden by default but displayable when the inline-images preference is enabled.
- [ ] Images with empty `src` attributes are removed entirely.
- [ ] Data-URI images (base64-encoded) pass through the sanitizer and are displayed.
- [ ] CSS `background-image: url(...)` declarations are converted to regular `<img>` elements subject to the same blocking/placeholder rules.
- [ ] A test message with remote images shows placeholders; enabling images for that message/sender loads them.

## HITL/AFK Classification
AFK — testable with HTML containing various image types and verifying no network requests occur.

## Notes
- FR-29 through FR-34 govern this story.
- Design Note N-4 explains why inline images are off by default.
- The actual "allow images for this sender/domain" preference storage is part of story 13 (view-toggle-and-per-sender-preferences). This story implements the blocking/placeholder mechanism and the global preference for inline images.
- The remote image fetching, caching, and proxying mechanism is out of scope (NG1, covered by feature 3.10).
