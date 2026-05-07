# User Story: MX-Based Matching for Custom Domains

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **custom-domain user** whose domain is hosted on a well-known provider (e.g. Google Workspace, Fastmail, Hetzner), I want the application to identify my hosting provider via MX DNS records and apply the correct settings, so that my custom domain works without manual configuration.

This slice adds MX-based matching as a secondary strategy when domain-pattern matching produces no result:
- When no provider domain pattern matches, and network is available, look up the MX records for the user's domain (FR-8).
- Match the MX records against the MX patterns of all enabled providers.
- On a match, return the provider entry as if the domain had matched directly.

## Acceptance Criteria
- [ ] When an email address's domain does not match any provider's domain patterns, the application performs a DNS MX lookup for that domain (requires network).
- [ ] The retrieved MX hostnames are matched against all enabled providers' MX patterns using regular-expression matching.
- [ ] When an MX pattern matches, the corresponding provider's settings are returned and pre-filled (AC-3).
- [ ] When no MX pattern matches either, the result is "no match" and the application proceeds to downstream fallback discovery (AC-4).
- [ ] MX-based matching is skipped when the device is offline (no network error shown — the application simply proceeds to the next fallback).

## Blocked by
`1-provider-data-model-and-domain-matching`

## HITL / AFK
**AFK** — Well-defined lookup and matching logic. DNS MX resolution is a standard library operation.

## Notes
- This is the one place where the "offline-first" principle is relaxed (Design Note N-4 in the epic). The MX lookup requires a network call. This is acceptable because domain-pattern matching (which is offline) has already failed at this point.
- The epic does not specify a timeout for MX lookups. The existing Android app uses standard DNS resolution. A reasonable timeout should be chosen to avoid blocking the UI.
