# Revision History Storage

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As a careful composer, I want the application to keep a history of prior saved states of my draft body as numbered revision snapshots, so that I can return to an earlier version if my recent edits were a mistake.

## Blocked by
1-local-draft-persistence

## Acceptance Criteria
- When revision history is enabled (default), each save creates a new numbered revision snapshot of the draft body, incrementing the revision counter. Previous revisions are retained. (FR-9)
- The first save of a new draft creates revision 1. (FR-10)
- When revision history is disabled, each save overwrites the single stored snapshot; no prior versions are retained and the revision counter does not advance. (FR-11)
- Revision snapshots are stored locally only and are never synchronized to the server. (FR-12)
- Revisions capture the user-composed body only — not quoted text or signatures. (Design Note N-5)
- The mechanism handles drafts with dozens of revisions without noticeable performance degradation. (NFR-4)
- Revision storage is efficient and does not grow without bound (cleanup is a separate story). (NFR-5)

## Mapping to Epic
- FR-9, FR-10, FR-11, FR-12
- NFR-4, NFR-5, NFR-8 (privacy — local only)
- US-9, US-13, US-14
- Design Notes N-4, N-5

## HITL / AFK
AFK — storage logic with clear contracts.

## Notes
- **OQ-2 (Revision branching):** The epic notes that the source application uses append-only history — undoing to revision 3 of 5 and then editing creates revision 6, leaving 4–5 accessible via redo. This differs from the common "discard forward history on edit" convention. The epic flags this as an open question. This story should implement the append-only approach as described, and the open question should be resolved before or during implementation.
- **OQ-3 (Revision storage limits):** The epic flags that there is no cap on revision count. This story does not introduce a cap, matching the epic, but the concern is noted.
- **OQ-4 (Quoted text in revisions):** The epic notes revisions capture body only, not quoted text. Flagged as OQ-4.
- **OQ-5 (Signature in revisions):** Signatures are excluded from revision snapshots. Flagged as OQ-5.

## Estimation
Medium — needs a revision-numbering scheme, snapshot storage, and a toggle between history and overwrite modes.
