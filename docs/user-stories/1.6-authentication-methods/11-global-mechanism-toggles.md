# Global Mechanism Toggles

## Parent Feature
#1.6 Authentication Methods

## User Story
As a security-conscious user, I want to disable specific password-based mechanisms (PLAIN, LOGIN, NTLM, CRAM-MD5, APOP) globally in the application's advanced settings, so that the application never attempts a mechanism I consider unacceptable on any connection.

## Acceptance Criteria
- Global toggle settings exist for: PLAIN, LOGIN, NTLM, CRAM-MD5 (via a general SASL toggle), and APOP.
- When a mechanism is disabled globally, the application does not attempt it on any connection, regardless of server advertisement.
- All password-based mechanisms except APOP are enabled by default.
- APOP is disabled by default.
- Disabling CRAM-MD5 causes accounts to fall back to LOGIN or PLAIN on the next connection attempt.
- Toggles are accessible from the application's advanced settings area, not from per-account settings.

## Blocked by
1-password-mechanism-negotiation

## HITL / AFK
AFK

## Notes
- Design Note N-4: toggles are global (not per-account) because they control process-level properties of the protocol libraries.
- The "SASL toggle" for CRAM-MD5 mentioned in FR-25 is slightly ambiguous — it may disable all SASL challenge-response mechanisms or just CRAM-MD5. Implementation should clarify based on what the underlying libraries expose.
