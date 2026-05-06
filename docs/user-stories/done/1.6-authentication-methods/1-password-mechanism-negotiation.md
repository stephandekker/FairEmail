# Password-Based Mechanism Negotiation

## Parent Feature
#1.6 Authentication Methods

## User Story
As a user adding an account with a username and password, I want the application to automatically negotiate the strongest authentication mechanism that both the application and the server support, so that I do not have to understand or choose a mechanism myself.

## Acceptance Criteria
- When connecting to a server advertising CRAM-MD5, LOGIN, and PLAIN, the application authenticates using CRAM-MD5 (the highest-priority mechanism).
- The preference order is: CRAM-MD5 > LOGIN > PLAIN > NTLM.
- The application inspects the server's capability advertisement (IMAP CAPABILITY, SMTP EHLO, POP3 CAPA) to determine available mechanisms.
- The negotiated mechanism is logged for diagnostic purposes.
- Supported mechanisms for IMAP: PLAIN, LOGIN, CRAM-MD5, NTLM, XOAUTH2, EXTERNAL.
- Supported mechanisms for POP3: PLAIN, LOGIN, CRAM-MD5, NTLM, XOAUTH2, APOP, EXTERNAL.
- Supported mechanisms for SMTP: PLAIN, LOGIN, CRAM-MD5, NTLM, EXTERNAL.

## Blocked by
(none — this is the foundational slice)

## HITL / AFK
AFK — no human-in-the-loop needed beyond code review.

## Notes
- This story establishes the core negotiation logic that all other password-based stories depend on.
- The epic specifies NTLM last in preference order despite it being challenge-response; this matches Design Note N-2 rationale (CRAM-MD5 preferred because it never transmits the password).
