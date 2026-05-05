## Parent Feature

#1.3 Quick Setup Wizard

## What to build

Network-based provider discovery using external autoconfig services, covering two detection strategies from FR-10:

1. **ISPDB (FR-10.5):** Query the Thunderbird project's Internet Service Provider Database for the domain. This is an online, community-curated autoconfig source.
2. **Vendor autodiscovery (FR-10.6):** Attempt vendor-specific autodiscovery protocols (e.g. Microsoft Autodiscover) for the domain.

Each strategy produces a candidate with a confidence score. ISPDB scores below DNS-based discovery. Vendor autodiscovery scores below ISPDB. (FR-11, Design Note N-3.)

**Privacy:** the user's password is never transmitted to these services (FR-38). Only the email domain is used in queries.

## Acceptance criteria

- [ ] The Thunderbird ISPDB is queried for the domain when higher-priority strategies fail (FR-10.5)
- [ ] Vendor autodiscovery protocols are attempted for the domain (FR-10.6)
- [ ] Each strategy produces a candidate with an appropriate confidence score (FR-11)
- [ ] ISPDB candidates score lower than DNS-discovered candidates (FR-11)
- [ ] The user's password is never sent to ISPDB or autodiscovery services (FR-38)
- [ ] Entering a domain with a correct ISPDB entry but no SRV records produces valid settings (AC-5)

## Blocked by

- Blocked by 2-bundled-provider-database

## User stories addressed

- US-8 (discover settings via online autoconfig when DNS discovery fails)
