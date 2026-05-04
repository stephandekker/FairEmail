# User Story: Tier 2 — Name-Based Heuristic Detection

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
- `1-role-taxonomy-and-folder-data-model`
- `2-tier1-imap-special-use-detection`

## Description
As a user whose mail server does not advertise SPECIAL-USE attributes, I want the application to infer folder roles from folder names using a multilingual dictionary of known patterns with confidence scoring, so that I am not forced to configure everything manually.

## Motivation
Many servers (especially self-hosted or older providers) lack RFC 6154 support. Name-based heuristics are the second line of defence and cover a large portion of the remaining user base.

## Acceptance Criteria
- [ ] For any folder that was **not** classified by Tier 1 (i.e. still has the User role), the application attempts to infer its role by matching its name against a built-in dictionary. _(FR-8)_
- [ ] Matching is **case-insensitive** and uses **substring containment** (not exact match), so hierarchical names like "INBOX/Drafts" or "Mail/Sent Items" are handled. _(FR-8, N-2)_
- [ ] The dictionary covers at minimum: **English, German, French, Russian, Italian, Polish, Dutch, and Norwegian**, with common folder names for Drafts, Trash, Spam, Sent, and Archive in each language. _(FR-9, AC-3)_
- [ ] Each dictionary entry carries a **confidence score**. When multiple folders match the same role, the candidate with the highest confidence is selected. _(FR-10)_
- [ ] Ties in confidence are broken by preferring **shallower** (less deeply nested) folders, then non-read-only folders. _(FR-10, N-5)_
- [ ] A name-heuristic match **only assigns a role** if no other folder in the same account already holds that role (from Tier 1 or a prior heuristic match). _(FR-11, N-3)_
- [ ] When ambiguous or low-confidence results are the only candidates, the application prefers **not assigning** a role over assigning the wrong one. _(US-9, NFR-7)_
- [ ] After adding an account with standard English folder names but no role metadata, at least Inbox, Sent, Drafts, Trash, and Spam are correctly identified. _(AC-2)_
- [ ] After adding an account with folder names in any of the supported languages, the heuristic correctly identifies Sent, Drafts, Trash, and Spam. _(AC-3)_
- [ ] The dictionary is structured so that adding new languages or entries does not require changes to the matching logic itself. _(NFR-4)_

## Sizing
Medium-Large — dictionary data for 8+ languages, matching engine with scoring, tie-breaking, and duplicate-prevention logic.

## HITL / AFK
AFK — the dictionary and matching rules are well-defined by the epic and existing Android code.

## Notes
- The existing Android code stores the dictionary in `EntityFolder.GUESS_FOLDER_TYPE` (lines 205–255) as a map of pattern → TypeScore. The desktop implementation should replicate the same coverage.
- The epic's OQ-2 flags uncertainty about the confidence threshold: should low-confidence matches be assigned tentatively or left unassigned? US-9 and NFR-7 lean toward "leave unassigned if confidence is too low", but the Android code assigns the best candidate regardless of absolute score. This tension should be resolved during implementation. For now, this story follows the epic's stated preference (US-9: prefer not assigning over assigning wrongly).
- N-2 notes that "all" is deliberately excluded from the dictionary to avoid false positives (e.g. matching "Install"). Care must be taken with short patterns.
