# Periodic Orphan Revision Cleanup

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As any user, I want a periodic background cleanup process to remove orphaned revision files (for drafts that no longer exist), so that storage remains tidy even after unexpected termination.

## Blocked by
6-revision-history-storage, 11-cleanup-on-send-discard

## Acceptance Criteria
- A periodic background process scans for revision files whose associated draft no longer exists or whose content has been purged, and removes them. (FR-28)
- Stale revision files are not retained indefinitely; the cleanup process removes them after a reasonable retention period. (FR-29)
- The cleanup process runs without user intervention and does not interrupt the user's workflow.
- The cleanup process correctly distinguishes active drafts (whose revisions must not be deleted) from orphaned ones.

## Mapping to Epic
- FR-28, FR-29
- NFR-5 (storage efficiency)
- US-24

## HITL / AFK
AFK — background maintenance task with clear rules.

## Notes
- The epic does not specify the retention period or cleanup frequency. These should be decided during implementation. A reasonable default might be to clean up orphaned files older than 7 days, running the cleanup on application startup or at a low-frequency interval.

## Estimation
Small-to-medium — file-system scan, draft-existence check, age-based retention logic.
