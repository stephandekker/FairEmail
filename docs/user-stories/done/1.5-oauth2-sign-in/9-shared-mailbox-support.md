# Shared Mailbox Support (Outlook)

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As an Outlook user with access to a shared mailbox, I want to authenticate with my own credentials and then specify the shared mailbox address, so that I can send and receive mail on behalf of the shared address.

## Blocked by
- `8-multi-tenant-support`

## Acceptance Criteria
- During setup for a provider that supports shared mailboxes (e.g. Outlook), the user can enter a shared mailbox identifier.
- The application uses the appropriate username syntax when authenticating to the mail server (e.g. `shared@domain\user@domain` for Outlook IMAP).
- The user authenticates with their own credentials (OAuth flow), not the shared mailbox's credentials.
- Both sending and receiving work for the shared mailbox.

## Mapping to Epic
- FR-40 (shared mailbox identifier, appropriate username syntax)
- US-23 (authenticate with own credentials, access shared mailbox)
- AC-15 (Outlook shared mailbox for both receiving and sending)
- Design Note N-8 (shared mailbox encoded as `shared@domain\user@domain`)

## HITL / AFK
HITL — user enters the shared mailbox address during setup.

## Notes
- Design Note N-8 describes the username encoding convention (`shared@domain\user@domain`) as a provider-specific convention, not a general mechanism. This story is Outlook-specific but should be implemented in a way that could support other providers with similar shared-mailbox mechanisms in the future.
- It is uncertain whether shared mailbox support should be exposed during initial setup, during account editing, or both. The epic does not specify. Recommend supporting both.
