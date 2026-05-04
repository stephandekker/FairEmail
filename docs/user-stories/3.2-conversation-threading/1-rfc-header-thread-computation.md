# RFC-Header-Based Thread Computation

## Parent Feature
#3.2 Conversation Threading

## User Story
As a standard user, when messages arrive in my mailbox, I want the application to extract `Message-ID`, `In-Reply-To`, and `References` headers and compute a thread identifier so that replies are grouped with their parent messages into conversations.

## Blocked by
_(none — this is the foundational slice)_

## Acceptance Criteria
- [ ] Every incoming message has its `Message-ID`, `In-Reply-To`, and `References` headers extracted and stored.
- [ ] `References` are processed in reverse order (most recent parent first); `In-Reply-To` is treated as the highest-priority reference (FR-6).
- [ ] A reply to a message is assigned the same thread identifier as the original (AC-1).
- [ ] A chain of three or more replies forms a single conversation, not multiple pairwise conversations (AC-2).
- [ ] A maximum limit of at least 450 references is enforced per message to avoid query-size constraints (FR-8).
- [ ] When threading headers are missing or malformed, the message is assigned its own thread identifier (NFR-7).
- [ ] Thread computation works entirely on locally stored data with no network round-trips (NFR-4).
- [ ] Re-processing a message that has already been threaded produces the same thread assignment (NFR-5).
- [ ] Thread computation completes without noticeably delaying message display under normal load (NFR-1).

## HITL / AFK
AFK — no human review needed beyond normal code review. This is a deterministic data-processing slice.

## Notes
- This story establishes the core threading data model and the RFC-header strategy. All other threading stories depend on it.
- The story deliberately does not address UI display of threads — that is covered in later stories. This slice covers the computation and persistence of thread identifiers.
- NFR-3 (determinism) applies: given the same messages and settings, the same threads must be produced regardless of message arrival order. This is a design constraint to keep in mind during implementation.
