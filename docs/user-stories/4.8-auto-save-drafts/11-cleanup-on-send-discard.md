# Cleanup Revision Files on Send or Discard

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As any user, I want revision history files to be cleaned up automatically when the draft is sent or discarded, so that old snapshots do not accumulate and waste disk space.

## Blocked by
6-revision-history-storage

## Acceptance Criteria
- When a draft is sent, all local revision files associated with that draft are eligible for cleanup and are removed (either immediately or by the next cleanup cycle). (FR-27, AC-13)
- When a draft is discarded (deleted), all local revision files associated with that draft are eligible for cleanup and are removed. (FR-27, AC-14)
- After cleanup, no orphaned revision files remain for the sent/discarded draft.

## Mapping to Epic
- FR-27
- US-23
- AC-13, AC-14

## HITL / AFK
AFK — lifecycle hook that deletes associated files.

## Estimation
Small — hook into send and discard paths, delete matching revision files.
