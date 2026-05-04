# Default Network Posture — Mail-Server-Only Traffic

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As a default user, when I install the application and configure my mail account without changing any settings, I want the application to communicate only with my configured mail servers (IMAP, POP3, SMTP, JMAP) and necessary DNS resolvers, so that no third party learns that I use this application, what I read, or whom I email.

## Blocked by
*(none — this is the foundational slice)*

## Acceptance Criteria
- A fresh installation with one IMAP account configured and all settings at defaults produces network traffic only to the configured IMAP and SMTP servers and DNS lookups needed to resolve them.
- No other hostnames are contacted.
- The application contains no analytics, telemetry, usage tracking, or behavioral profiling code that activates in the default configuration.
- The application does not use cloud-based push notification services in its default configuration; mail arrival notifications are driven by direct IMAP IDLE connections or local polling.
- The application does not perform automatic update checks against external servers in its default configuration.
- Core email operations (reading, composing, searching, organizing, synchronizing) function at full speed without any external service dependency.
- The application remains fully functional for all core email operations when no internet connectivity exists beyond the mail server.

## Mapping to Epic
- US-1, US-2, US-3, US-4
- FR-1, FR-2, FR-3, FR-4
- NFR-5, NFR-6
- AC-1, AC-11

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This is the foundational privacy contract of the entire application. Every subsequent story in this epic builds on this guarantee.
- Verification of this story should include network traffic analysis (e.g. a packet capture during a fresh-install session) to confirm no unexpected outbound connections.
- FR-3 specifies no cloud-based push notifications by default. On Linux desktop, this is likely simpler than on Android since there is no vendor push infrastructure (FCM/GCM) to contend with. The primary mechanism will be IMAP IDLE or polling.
- FR-4 covers update checks. On Linux, the system package manager may handle updates, making this a non-issue for some distribution channels. The requirement is that the application itself does not phone home.
