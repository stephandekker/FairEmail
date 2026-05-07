# User Story: Discovery Pipeline Trust Scoring

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **user whose email provider can be discovered by multiple methods**, I want the bundled catalogue match to always take priority over network-derived configurations (DNS SRV, ISPDB, Autodiscovery, port scan), so that I get the most reliable and complete configuration available.

This slice integrates the provider catalogue into the broader discovery pipeline using trust scoring:
- Each discovery method assigns a numeric trust score to its results (FR-9).
- Bundled catalogue entries (both domain-matched and MX-matched) receive the highest trust score.
- When multiple discovery methods return configurations for the same provider, the highest-scoring configuration wins (Design Note N-2).
- The discovery pipeline does not stop at the first match — it collects results from all available methods and ranks them.

## Acceptance Criteria
- [ ] Bundled catalogue matches receive a higher trust score than any network-derived configuration method.
- [ ] When both the bundled catalogue and a network discovery method return configurations for the same domain, the bundled catalogue's configuration is used.
- [ ] When the bundled catalogue has no entry for a domain, network-derived configurations (DNS SRV, ISPDB, Autodiscovery, port scan) are used as fallback (AC-4).
- [ ] The trust-score ranking is deterministic: given the same inputs, the same configuration always wins.

## Blocked by
`1-provider-data-model-and-domain-matching`, `5-mx-based-matching`

## HITL / AFK
**AFK** — Numeric scoring with clear precedence rules. Testable with mock discovery results.

## Notes
- The existing Android app uses a scoring system: built-in = 100, DNS SRV = 50, ISPDB/Autodiscovery = 20, port scan = 10. The exact numbers are not prescribed by the epic, but the relative ordering must be preserved.
- This story touches the integration boundary with the discovery pipeline (epic 1.3/1.4). The trust-scoring contract should be designed so that adding new discovery methods in the future is straightforward.
