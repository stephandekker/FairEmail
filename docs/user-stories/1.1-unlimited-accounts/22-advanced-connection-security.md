# Advanced Connection Security Settings

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As a security-conscious user, I want to configure advanced security options per account — DNSSEC, DANE, certificate pinning, client certificates, and insecure connection override — so that I can harden or relax connection security to match each server's capabilities.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- Each account exposes optional security settings: DNSSEC enforcement flag, DANE enforcement flag, insecure-connection flag, server certificate fingerprint (for pinning), client certificate reference, and authentication realm (FR-4).
- DNSSEC and DANE can be enabled per account (US-8).
- A specific certificate fingerprint can be pinned, or insecure connections can be allowed, for a single account without affecting others (US-9).
- A client certificate can be selected from the system keystore for mutual TLS authentication (US-10).
- These settings are hidden behind an expandable "Advanced" section (FR-53).

## Mapping to Epic
- US-8, US-9, US-10
- FR-4, FR-53

## HITL / AFK
AFK

## Notes
- Client certificate selection from the system keystore is platform-specific. On Linux, this likely involves reading from NSS or PKCS#11 stores. The exact mechanism is an implementation detail.
