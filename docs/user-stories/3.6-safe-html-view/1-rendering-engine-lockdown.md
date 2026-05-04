# Rendering Engine Lockdown

## Parent Feature
#3.6 Safe HTML View

## Blocked by
None

## Description
Configure the rendering engine (webview/embedded browser component) used for displaying email messages so that scripting, local file access, cookie storage, and attribution/tracking APIs are unconditionally disabled. This is the foundational defense-in-depth layer that protects the user even if the sanitization pipeline has a bypass.

## Motivation
The rendering engine lockdown is independent of the sanitization pipeline and must be in place first. It ensures that even unsanitized content (e.g. in the original view) cannot execute scripts, read local files, or set cookies. Every other story in this epic assumes this lockdown is active.

## Acceptance Criteria
- [ ] The rendering engine has JavaScript/script execution disabled at the engine configuration level.
- [ ] The rendering engine cannot access local filesystem paths (file:// protocol blocked or equivalent).
- [ ] The rendering engine does not store or send cookies.
- [ ] Attribution/tracking APIs exposed by the rendering engine are disabled.
- [ ] These restrictions apply regardless of the content being rendered (safe view and original view alike).
- [ ] Attempting to execute a script from rendered content produces no effect (verifiable via a test message containing `<script>alert(1)</script>`).
- [ ] Attempting to load a local file from rendered content produces no effect.

## HITL/AFK Classification
AFK — no human review needed during implementation; verification is via automated tests.

## Notes
- The specific rendering engine choice (e.g. WebKit-based, Chromium-based, or custom) is not prescribed by the epic. This story is engine-agnostic and applies to whatever component is selected.
- FR-48 through FR-51 are the governing requirements.
