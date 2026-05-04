# User Story: Re-Sync — Preserve Existing Assignments and Classify New Folders

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
- `2-tier1-imap-special-use-detection`
- `4-tier2-name-heuristic-detection`

## Description
As any user, when the application re-synchronizes the folder list from the server, I want existing role assignments to remain stable and any newly discovered folders to be classified without displacing existing assignments, so that my configuration is never unexpectedly disrupted by a sync cycle.

## Motivation
Folder re-synchronization happens frequently (on every connect, on manual refresh, etc.). If detection re-ran from scratch each time, it could flip-flop assignments or undo user overrides. Stability is essential for trust.

## Acceptance Criteria
- [ ] On re-synchronization, the application does **not change** the role of a folder that already has an assigned role, except when: _(FR-17)_
  - A folder transitions between User and System types based on updated server attributes.
  - A folder gains or loses selectability.
  - The folder previously holding the Inbox role no longer exists, and a new folder claims Inbox via server attributes.
- [ ] Newly discovered folders (not previously seen) are classified using the full detection strategy (Tier 1 then Tier 2), but do **not displace** an existing role assignment. _(FR-18, AC-12)_
- [ ] After a manual override (Story 7), re-synchronizing the folder list does **not revert** the user's choice. _(AC-6, FR-16)_
- [ ] If the server fails to return a folder list (e.g. network error), the application retains previously detected assignments and retries on next synchronization, rather than clearing assignments. _(NFR-6)_
- [ ] Once a role is assigned (automatically or manually), it remains stable across unlimited re-synchronization cycles unless the underlying server state genuinely changes. _(NFR-3)_

## Sizing
Medium — integration of detection pipeline with sync lifecycle, conditional logic for when to re-detect vs. preserve.

## HITL / AFK
AFK — the rules are explicit in the epic.

## Notes
- The Android code handles this in `Core.java` (lines 2890–3131), where folder sync creates/updates folder records and conditionally applies `guessTypes()`. The desktop implementation should replicate the same "classify new, preserve existing" semantics.
