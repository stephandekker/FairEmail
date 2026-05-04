## Parent Feature

#1.3 Quick Setup Wizard

## What to build

Network-based provider discovery using DNS records, covering three detection strategies from FR-10:

1. **DNS NS record match (FR-10.2):** Look up the domain's name-server records and match against known provider patterns. If a match is found, attempt RFC 6186 SRV discovery first, falling back to the matched provider entry.
2. **DNS MX record match (FR-10.3):** Look up the domain's MX records and match against known provider MX patterns or root-domain equivalences.
3. **RFC 6186 DNS SRV discovery (FR-10.4):** Query `_imaps._tcp`, `_imap._tcp`, `_submissions._tcp`, and `_submission._tcp` SRV records for the domain.

Each strategy produces a candidate with a confidence score. DNS NS matches score below bundled-database matches but above ISPDB/autodiscovery. MX matches score below NS. SRV discovery scores below MX. (FR-11, Design Note N-3.)

This slice handles only the discovery logic and candidate production — it does not include UI integration (handled by slice 6) or connectivity checking (slices 7-8).

## Acceptance criteria

- [ ] DNS NS records are looked up and matched against known provider patterns (FR-10.2)
- [ ] DNS MX records are looked up and matched against known provider MX patterns (FR-10.3)
- [ ] RFC 6186 SRV records are queried for IMAP and SMTP services (FR-10.4)
- [ ] Each strategy produces a candidate with an appropriate confidence score (FR-11)
- [ ] DNS-discovered candidates score lower than bundled-database matches (FR-11)
- [ ] Entering an address at a custom domain with correct DNS SRV records produces valid server settings (AC-4)

## Blocked by

- Blocked by 2-bundled-provider-database

## User stories addressed

- US-8 (discover settings via DNS when domain is not in bundled database)
