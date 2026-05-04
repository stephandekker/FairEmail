# User Story: Provider-Specific Folder Rules

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
- `4-tier2-name-heuristic-detection`
- `6-role-triggered-default-properties`

## Description
As a user of a provider with non-standard folder semantics (e.g. Gmail's "All Mail" serving as Archive, or a provider with a dual-purpose folder), I want the application to correctly map provider-specific conventions to standard roles and apply any supplementary default behaviors, so that features like "archive" work as expected on my provider.

## Motivation
The generic detection tiers handle most servers, but some large providers have idiosyncratic folder structures that require special handling. Provider-specific rules are a targeted supplement to ensure correct behaviour for these providers without polluting the generic logic.

## Acceptance Criteria
- [ ] The application supports a mechanism to define provider-specific folder rules that trigger additional default behaviours for specific folders on specific servers. _(FR-25)_
- [ ] Provider-specific rules are applied **after** standard detection, as a supplementary layer. _(FR-26)_
- [ ] Provider-specific rules do **not interfere** with user overrides. _(FR-26, AC-15)_
- [ ] Rules can include actions such as: adding a folder to the unified inbox, overriding a folder's display name, or adjusting sync/polling defaults for a specific provider. _(FR-25)_
- [ ] Provider-specific rules for one account do not affect detection or behaviour on other accounts. _(AC-15)_
- [ ] The "All Mail" folder on providers like Gmail is correctly mapped to the Archive role. _(US-20)_

## Sizing
Small-Medium — a rule definition mechanism, a small initial set of provider rules, and integration into the post-detection pipeline.

## HITL / AFK
AFK — the rules are data-driven and well-bounded.

## Notes
- The Android code has `EntityFolder.setSpecials()` (lines 319–334) which applies provider-specific overrides based on server hostname. The desktop implementation should replicate the same rules.
- OQ-3 in the epic flags uncertainty about whether "All Mail" style archives should be distinguished from "move-to" style archives. This story implements the mapping as specified (All Mail → Archive) but the distinction may warrant a follow-up design discussion.
- OQ-7 asks whether provider-specific rules should be externalized into a user-editable or community-maintained configuration. This story implements them as built-in rules; externalization could be a future enhancement.
