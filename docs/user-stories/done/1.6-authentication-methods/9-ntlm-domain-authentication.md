# NTLM Domain Authentication

## Parent Feature
#1.6 Authentication Methods

## User Story
As an enterprise user on a Windows domain, I want to specify a domain/realm alongside my username and password, so that NTLM authentication succeeds against my organization's Exchange server.

## Acceptance Criteria
- The account settings include a domain/realm field.
- The domain/realm value is sent as the NTLM domain during authentication.
- Specifying the correct domain allows NTLM authentication to succeed against a server requiring it.
- Omitting the domain when the server requires it causes authentication to fail with a clear error.
- The same realm field serves both NTLM (as Windows domain) and CRAM-MD5 (as SASL realm) per Design Note N-7.

## Blocked by
1-password-mechanism-negotiation

## HITL / AFK
HITL — OQ-2 flags that NTLM is documented as "untested" in the existing FAQ; verification against a real Exchange server is recommended before advertising support.

## Notes
- OQ-2: NTLM reliability should be verified before it is advertised as supported. Consider flagging this as experimental in documentation.
- Design Note N-7: a single realm/domain field serves dual purpose for CRAM-MD5 and NTLM.
