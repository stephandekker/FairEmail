## Parent Feature

#1.3 Quick Setup Wizard

## What to build

When a network-discovered candidate's server hostname matches a bundled provider entry, the wizard replaces the network-discovered settings with the bundled entry's settings — preserving the discovery score — so that provider-specific flags and documentation are available (FR-12, Design Note N-4).

**Why this matters:** network discovery (DNS, ISPDB, autodiscovery) returns only basic connection parameters (hostname, port, encryption). The bundled database carries rich provider-specific metadata: keep-alive interval, NOOP mode, partial-fetch support, TLS ceiling, app-password hints, documentation links, and OAuth config. By replacing network-discovered settings with the bundled entry when the hostname matches, the wizard ensures these tuning parameters are applied even for domains discovered via MX or ISPDB rather than direct domain match.

## Acceptance criteria

- [ ] When a network-discovered candidate's hostname matches a bundled provider entry, the settings are replaced with the bundled entry's values (FR-12)
- [ ] The original discovery score is preserved after the merge (FR-12)
- [ ] Provider-specific flags (keep-alive, NOOP, partial-fetch, TLS ceiling) from the bundled entry are applied
- [ ] Provider documentation link and app-password hint from the bundled entry are available
- [ ] If no bundled entry matches the hostname, the network-discovered settings are used as-is

## Blocked by

- Blocked by 15-dns-provider-discovery
- Blocked by 16-ispdb-and-vendor-autodiscovery

## User stories addressed

- US-7 (provider-specific settings applied even for network-discovered domains)
- US-8 (network discovery enriched with bundled provider metadata)
