# Contact Store Purge

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, I want the local contact store to periodically purge old, rarely-contacted entries based on configurable age and frequency thresholds, so that the store does not grow unboundedly with stale data.

## Blocked by
- `1-local-contact-store-with-sent-mail-learning`

## Acceptance Criteria
- The local contact store periodically evaluates entries for purging.
- A "contact purge age" setting controls how old an entry must be (in months) before it is eligible (default: 1 month).
- A "contact purge frequency" setting controls the minimum number of interactions below which an entry is eligible (default: 0, meaning purge by age only).
- An entry is eligible for purging only if it meets both the age and frequency criteria.
- Purged entries are removed from the store entirely.
- Entries marked as "favorite" or "ignored" are not purged.
- After configuring a purge age of 1 month and frequency of 0, contacts older than 1 month with 0 additional interactions are eligible for removal.

## HITL/AFK Classification
**AFK** — behaviour is well-defined; purge logic is a background process.

## Notes
- OQ-5 in the epic questions whether the default purge age of 1 month is too aggressive for a desktop client. This story follows the epic as written (default: 1 month). Consider adjusting the default if user feedback suggests it is too aggressive.
- FR-16 defines the purge behaviour.
- FR-40g and FR-40h define the configuration settings.
