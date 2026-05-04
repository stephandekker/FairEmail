# Parallel Per-Account Queue Processing

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user with multiple accounts, I want each account's operation queue to be processed independently and in parallel, so that a slow or failing server on one account does not block operations for my other accounts.

## Acceptance Criteria
- Multiple accounts process their operation queues in parallel, independently.
- A transient or permanent failure on one account does not delay or block queue processing for other accounts.
- Each account's queue respects its own priority ordering independently.
- The operations view shows account information per operation so the user can see which account is affected.

## Complexity
Small

## Blocked by
3-replay-single-operation

## HITL/AFK
AFK

## Notes
- This is primarily an architectural concern about the replay engine: it must not serialize all operations through a single connection or thread. Each account should have its own replay worker/connection.
