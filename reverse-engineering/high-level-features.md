# FairEmail — High-Level Feature List (Linux Desktop Reframing)

This list reverse-engineers FairEmail's user-visible feature set, but described as if FairEmail were a Linux desktop email application. Mobile-only concepts are translated to their desktop equivalents (e.g. notification channels → libnotify; biometric unlock → system keyring; Play Store updates → distribution package manager). Concepts with no meaningful desktop counterpart are omitted.

Sources synthesized: `README.md`, `PLAYSTORE.txt`, `metadata/en-US/full_description.txt`, `app/src/main/res/layout/fragment_options_*.xml`, `app/src/main/res/menu/*.xml`, `FAQ.md`, `CHANGELOG.md` (current head: 1.2315, 2026-04-27), `tutorials/SETTINGS-OVERVIEW.md`. Where sources disagreed, the latest behavior wins.

---

## 1. Accounts, Identities & Setup

- **1.1 Unlimited accounts.** Multiple IMAP, POP3 and JMAP accounts; each with its own connection settings, color, name, avatar, and category grouping.
- **1.2 Unlimited identities per account.** Each account can host multiple "From" identities (alias, display name, signature, reply-to, BCC, signing/encryption defaults, default folder).
- **1.3 Quick setup wizard.** Guided onboarding that detects the provider from the email address and fills in IMAP/SMTP servers automatically using a bundled provider database.
- **1.4 Manual server configuration.** Full IMAP/POP3 + SMTP setup with host, port, encryption (SSL/TLS, STARTTLS, plain), authentication method, certificate selection, and "Test connection" before saving.
- **1.5 OAuth2 sign-in.** First-class OAuth flows for Gmail, Outlook/Microsoft 365, Yahoo, AOL, Yandex, Mail.ru, etc. Tokens are refreshed automatically.
- **1.6 Authentication methods.** PLAIN, LOGIN, CRAM-MD5, NTLM, OAuth/XOAUTH2, APOP, and EXTERNAL (RFC 4959, e.g. for client-cert auth).
- **1.7 Pre-installed provider database.** Bundled list of providers with known IMAP/SMTP/OAuth settings, available offline.
- **1.8 Account / identity / folder coloring & avatars.** User-defined colors and image avatars per account, identity and folder, used throughout the UI.
- **1.9 Account categories.** Group accounts under user-defined category labels in the navigation drawer.

## 2. Folders

- **2.1 Two-way IMAP synchronization.** Local actions (move, flag, delete, mark read) propagate to the server and vice versa.
- **2.2 Special-folder auto-detection.** Automatically detects Inbox, Sent, Drafts, Trash, Spam, Archive, plus user overrides.
- **2.3 Per-folder sync settings.** Sync on/off, poll interval, days-to-keep, days-to-download, auto-classify, notify on new mail, and download attachments per folder.
- **2.4 Folder subscription management.** Subscribe/unsubscribe IMAP folders; create, rename, delete folders from the UI.
- **2.5 Auto-sync newly added folders.**
- **2.6 Hide system folders.** Choose which folders appear in the folder list.
- **2.7 Folder colors and per-folder unread counters.**
- **2.8 Manual folder operations.** Empty trash, empty spam, expunge, mark all read, force re-sync, force send.

## 3. Reading Messages

- **3.1 Unified inbox.** Cross-account merged inbox; togglable per-account or per-folder.
- **3.2 Conversation threading.** RFC-compliant threading using `In-Reply-To` / `References`, plus Gmail-style and subject-fallback modes; configurable per-account.
- **3.3 Reading-pane layouts.** Single, two-column or three-column pane layouts that adapt to window width.
- **3.4 Auto-expand first / latest unread message** in a conversation, with collapse/expand controls per message.
- **3.5 Mark as read on view, on scroll, or never** — with an optional delay and an undo grace period.
- **3.6 Safe HTML view.** Sanitized HTML rendering with scripts, remote fonts, form actions and unsafe styling stripped.
- **3.7 Original (raw) message view.** Switch to the original HTML/plain rendering if needed.
- **3.8 Reformatted plain-text view.** Optional re-flow into a phishing-resistant plain layout.
- **3.9 Inline attachment preview.** Inline preview for images, audio, PDF, EML and barcodes.
- **3.10 External-image confirmation.** Block remote images by default; one-click "show images" with optional "always allow this sender / domain".
- **3.11 Tracker-image detection.** Heuristic detection of tracking pixels using bundled blocklists (Disconnect, AdGuard, etc.) — blocked images are reported.
- **3.12 Link confirmation & sanitization.** Confirm before opening links; strip known tracking parameters; warn on punycode/spoofed domains; "open in private browser" option.
- **3.13 Authentication results display.** SPF / DKIM / DMARC / ARC / DNSSEC / MTA-STS / DANE indicators per message; warning banner on failure.
- **3.14 BIMI logo display** with optional VMC certificate verification.
- **3.15 Conversation actions.** Reply, reply-all, forward, forward-as-attachment, redirect/bounce, edit-as-new, resend.
- **3.16 Per-message actions.** Snooze, hide, pin, flag with color, set importance, edit subject, edit notes, manage IMAP keywords/labels, move, copy, delete, delete attachments.
- **3.17 Show original headers / source / parsed HTML / alternative text** for power users.
- **3.18 Charset override** when a message declares an incorrect charset.
- **3.19 Translate message** body via on-device or cloud translation provider.
- **3.20 Summarize message** via configurable LLM (OpenAI, Gemini, Ollama, Groq, Mistral, DeepInfra, OpenAI-compatible endpoints).
- **3.21 Inline TTS read-aloud** of message body via system speech-dispatcher.
- **3.22 Calendar invitations.** Display ICS invites inline; accept / decline / tentative; add to system calendar (Evolution / KOrganizer via DAV).
- **3.23 Add message / event to calendar.**
- **3.24 Auto-save vCard for senders** to a local address book.
- **3.25 One-click unsubscribe** (RFC 8058 + List-Unsubscribe header) with confirmation dialog.
- **3.26 Pin / save message** to a starred / pinned area or to disk as `.eml`.

## 4. Composing Messages

- **4.1 Rich text editor** with formatting toolbar: bold, italic, underline, strikethrough, sub/super-script, font, size, color, highlight, bullet/numbered lists, indentation, alignment, headings, links, horizontal rule, clear formatting.
- **4.2 Plain-text mode** per identity or per message.
- **4.3 Markdown editor** with live preview, served by CommonMark.
- **4.4 HTML / plain dual-part output** controllable per send.
- **4.5 Signature management.** Per-identity signatures (HTML or plain), with placement (above/below quote, before/after reply text), variables (name, email, date), and disable-on-reply option.
- **4.6 Reply templates ("answers").** Saved snippets that can include placeholders (`$name$`, `$subject$`, etc.), inserted into a draft on demand or auto-applied.
- **4.7 Quote handling.** Customizable quote prefix, collapsible quote on receive, auto-collapse depth, "reply above quote" vs. "reply below".
- **4.8 Auto-save drafts.** Drafts saved on paragraph break, on punctuation, or on a timer; revision history kept locally with rollback.
- **4.9 Send delay (undo-send).** Configurable grace period (e.g. 5 / 15 / 30 s) during which the send can be cancelled.
- **4.10 Schedule send.** Send at a specific date / time.
- **4.11 Recipient autocomplete.** Suggestions drawn from sent mail, received mail, frequency rank, the local address book, or the system address book (Evolution Data Server).
- **4.12 Insert contact group.** Expand a saved group of recipients into the To / Cc / Bcc fields.
- **4.13 Recipient chips with validation.** Each recipient rendered as a chip with PGP/S-MIME key indicator and trust state.
- **4.14 Attachments.** Add files, images (with optional resize / EXIF strip / HEIC re-encode), inline images, vCards, pre-existing message attachments, or "attach this thread as EML". Collapsible attachment list.
- **4.15 Auto-attach own vCard.**
- **4.16 Forward as attachment / edit as new / redirect.**
- **4.17 Inline image management** with drag-to-reorder.
- **4.18 Encryption defaults per identity** (sign / encrypt / sign+encrypt / none) with auto-detect of recipient keys.
- **4.19 Read & delivery receipts.** Request DSN / MDN per message; standard or legacy formats.
- **4.20 Subject prefix normalization** (`Re:` / `Fwd:` deduplication, locale variants).
- **4.21 Reply movement.** Optionally move the original message to a chosen folder after replying.
- **4.22 Standard / configurable shortcut keys** for common compose actions.
- **4.23 Find-in-text** while composing or reading.
- **4.24 Print message** (standard CUPS print dialog).

## 5. Search

- **5.1 Server-side IMAP search** with date range, sender, recipient, subject, body, flags.
- **5.2 Local full-text search index** of message bodies (configurable size and exclusion list).
- **5.3 Saved searches.** Persist a query as a virtual folder.
- **5.4 Quick filter bar.** Toggle filters for read/unread, flagged/unflagged, attachment, deleted, snoozed/hidden, duplicates, sent, trash, language.
- **5.5 Sort options.** Sort by time, unread, starred, priority, sender, sender name, subject, size, attachment count, snooze time — ascending or descending.
- **5.6 Search-in-text** within an open message or draft.
- **5.7 Select-all-found** to act on the current search result set in bulk.

## 6. Rules & Automation

- **6.1 Per-folder filter rules.** Conditions on sender, recipient, subject, header, body, schedule, group membership, attachments, age, unsubscribe, flagged, etc.
- **6.2 Rule actions.** Move, copy, delete, mark read/unread, flag with color, set importance, hide, snooze, add keyword/label, set priority, run TTS, forward, auto-reply, run external command (D-Bus / shell hook), add sender to address book.
- **6.3 Rule scheduling.** Active only during chosen days / hours.
- **6.4 Create rule from message.** "Block sender", "Always move from X to Y" shortcuts pre-fill a rule from the open message.
- **6.5 Stop processing** flag to short-circuit subsequent rules.
- **6.6 Test rule** against existing messages before enabling.
- **6.7 Bayesian / on-device classifier.** Optional automatic message categorization that learns from where the user moves messages; can be applied automatically or as a suggestion.

## 7. Synchronization & Connectivity

- **7.1 IMAP IDLE (push)** with multi-folder IDLE, server capability detection, and automatic fallback to polling.
- **7.2 Configurable poll interval** (per account and per folder), independent of IDLE.
- **7.3 Schedule-based sync.** "Work hours" mode that only syncs during selected days/times, with exception accounts.
- **7.4 Network-aware sync.** Different behavior on metered vs. unmetered, on VPN, or when offline.
- **7.5 Offline storage and operations.** All actions queue locally and replay when the connection returns.
- **7.6 Operations queue.** Visible list of pending IMAP/SMTP operations with retry, cancel, and error inspection.
- **7.7 Compaction / expunge** of deleted messages on demand or on a schedule.
- **7.8 Connection tuning.** Custom timeouts, connect/read/write, prefer IPv4/IPv6, custom DNS resolver, optional DNS-over-HTTPS, TCP keep-alive interval.
- **7.9 TLS hardening.** TLS 1.2/1.3 only, OCSP, certificate transparency, hostname verification mode, per-account pinned certificate, optional FIPS-mode crypto via Bouncy Castle, custom trust anchors / system store / both.
- **7.10 Client identification (RFC 7162 ID command)** with generic vs. per-account identity, optional "lie" to the server for privacy.
- **7.11 POP3 download cap** (max messages / days), leave-on-server, UIDL tracking.
- **7.12 Standalone VPN bind.** Bind connections to a specific network interface (useful for split-tunnel).
- **7.13 Captive-portal detection** with safe-mode prompt.

## 8. Notifications

- **8.1 Desktop notifications** (libnotify) for new mail, sync errors, send confirmations.
- **8.2 Per-account / per-folder / per-sender** notification rules: sound, persistence, summary vs. individual, "do not notify".
- **8.3 Notification actions.** Reply, archive, delete, trash, spam, mark seen, flag, snooze, hide, move-to, junk, TTS — chosen per account.
- **8.4 Quick reply from notification** (where the desktop notification daemon supports inline reply).
- **8.5 Summary notification** — single grouped notification with count and quick "mark all read".
- **8.6 Sender previews / avatars** in notifications.
- **8.7 Quiet hours.** Suppress notifications during user-defined times.
- **8.8 Sound / vibration / LED equivalents.** Custom sound per account; system tray badge or panel indicator for unread count.
- **8.9 Background daemon (system tray icon).** Persistent process running as a user `systemd` unit, with status icon showing connection / unread state — equivalent of the Android foreground service.
- **8.10 Autostart on login.**

## 9. Display & Theming

- **9.1 Light / dark / black themes** with auto-switch following the system theme (GNOME / KDE / freedesktop color-scheme).
- **9.2 Beige / sepia low-contrast theme.**
- **9.3 Accent color picker.**
- **9.4 Density modes.** Compact, normal, relaxed list density; configurable line spacing and inter-message spacing.
- **9.5 Card vs. tabular message list.**
- **9.6 Two-line / three-line / preview-text message rows.**
- **9.7 Configurable list columns.** Avatar, account color, flag, importance, attachments, size, preview, count, date format, relative-vs-absolute timestamps, day-bucket headers.
- **9.8 Column-based reading layouts.** Single-pane, two-pane (folder + message list + reader), three-pane.
- **9.9 Per-conversation grouping.** Inline category headers (today / yesterday / this week / older).
- **9.10 Customizable swipe / keyboard actions.** Each direction (or shortcut) can be bound to: archive, trash, delete permanently, junk, move-to, snooze, hide, flag, mark read/unread, forward, reply.
- **9.11 Configurable action buttons** in the message list and reader (which actions appear, in what order).
- **9.12 Configurable navigation drawer.** Show/hide entries (Folders, Labels, Search, Operations, Logs, Help, etc.).
- **9.13 Reading-mode font controls.** Override message font, monospace toggle, zoom level, "force light background for dark messages".
- **9.14 Image width clamping** to viewport width.
- **9.15 Standard system fonts** plus bundled Roboto / monospace fallback.

## 10. Privacy

- **10.1 No telemetry by default.** No analytics, no remote crash reporting; opt-in error reporting via Bugsnag if user explicitly enables it.
- **10.2 No third-party servers.** All data stays local; sync goes only to the user's mail server.
- **10.3 Tracking-pixel stripping.** Bundled blocklists from Disconnect, AdGuard, EasyList — keep automatically updated.
- **10.4 Tracking-link debouncing** using Brave's debounce list (rewrites known tracking redirectors to their final URL).
- **10.5 Image proxy disabled by default.** Optional opt-in per provider.
- **10.6 Confirm before opening links / images / attachments.**
- **10.7 Disable web fonts and remote CSS** in the safe view.
- **10.8 Generic timezone & locale** in outgoing mail headers (avoid fingerprinting).
- **10.9 Strip EXIF / metadata** from attached images on send.
- **10.10 Reformat suspicious messages** to plain text (anti-phishing).
- **10.11 Suspicious-link warning** for punycode, mismatched display vs. target, IDN homographs.
- **10.12 Public-suffix-list-based domain checks.**

## 11. Security

- **11.1 OpenPGP** sign / encrypt / decrypt / verify, integrated with system GPG agent (via OpenPGP-compatible API; OpenKeychain on Android, GnuPG on Linux). Auto-detect recipient keys, attach own key on first send, encrypted-subjects support.
- **11.2 Autocrypt** Level 1 (header advertisement, mutual mode, key recovery via setup message).
- **11.3 S/MIME** sign / encrypt / decrypt / verify with bundled Mozilla CA root list, plus user-imported `.p12` / `.pfx` certificates kept in the system keyring.
- **11.4 DANE / TLSA** verification of MX hosts.
- **11.5 Certificate Transparency** check on TLS connections (optional).
- **11.6 Application lock.** Lock the app behind the system keyring / PolKit / fingerprint-equivalent (e.g. `pam_u2f`, GNOME Keyring, KWallet) with idle timeout and re-prompt on resume.
- **11.7 Per-database encryption-at-rest** for the local message store (SQLCipher).
- **11.8 Master password** option to encrypt account credentials in the local store.
- **11.9 Confidential view.** Mode that blanks the window contents when it loses focus or the screen locks.
- **11.10 Block external content** in encrypted messages.

## 12. Address Book / Contacts

- **12.1 Local contact store.** All sent/received addresses are remembered with first-seen / last-seen / count, used for autocomplete and grouping.
- **12.2 System address-book integration.** Read/write through the desktop's contact store (Evolution Data Server / KAddressBook), with a per-account toggle.
- **12.3 Contact groups.** User-defined groups expandable in compose.
- **12.4 Per-contact avatar, color, alias, notes.**
- **12.5 Trusted / blocked / never-favorite flags** influence notification, autocomplete and tracker handling.
- **12.6 vCard import / export** (single contact and bulk).
- **12.7 Auto-add senders** to the local address book based on rules (e.g. only after replying).
- **12.8 Identicons / Gravatar / Libravatar / BIMI / favicon / contact-photo** as avatar sources, in user-chosen priority order; all fetches optional.

## 13. AI Integration

- **13.1 Pluggable LLM providers.** OpenAI, Google Gemini, Groq, Mistral, DeepInfra, Ollama (local), or any OpenAI-compatible endpoint with a custom base URL and model name.
- **13.2 Summarize selected message or thread.**
- **13.3 Translate selected message** (auto-detect source language, configurable target language).
- **13.4 Auto-translate on receive** (per account, per folder, or per language).
- **13.5 AI-assisted compose / reply** with custom system prompts.
- **13.6 Cost guardrails.** Per-prompt token caps and per-account opt-in.

## 14. External Integrations

- **14.1 CalDAV / WebDAV / Nextcloud.** Save attachments and create calendar events directly to a configured WebDAV / Nextcloud / CalDAV endpoint.
- **14.2 xdg-open and desktop "Open with…"** for attachments and links.
- **14.3 D-Bus interface** for scripting (equivalent to Tasker / Locale plug-in on Android): trigger sync, send a draft, mark folder as read, etc.
- **14.4 CLI / shell hook** as a rule action ("run command" with message variables).
- **14.5 System share.** "Send to…" target so other applications can share files / text into a new compose window.
- **14.6 System mailto: handler** registration.

## 15. Backup & Portability

- **15.1 Encrypted settings export.** Export accounts, identities, rules, signatures, templates, certificates, address book, and preferences to a single password-encrypted archive; import on another machine.
- **15.2 Selective export / import** (choose which accounts and which categories of data).
- **15.3 Cloud backup.** Optional encrypted backup to a user-supplied cloud (WebDAV, IMAP "FairEmail/backup" folder, or OAuth-based providers); push, pull, wipe.
- **15.4 Raw `.eml` export / import** for individual messages or whole folders.
- **15.5 Database export** for power users (SQLite dump of the local store).

## 16. Logging & Diagnostics

- **16.1 Operations log** showing every IMAP / SMTP exchange and queued operation, with timestamps and error detail.
- **16.2 Per-account connection log** (toggle protocol-level logging).
- **16.3 Debug mode.** Verbose logging, jsoup HTML inspector, message body charset inspector, and "Reset all questions" to undo all "don't ask again" choices.
- **16.4 Crash & exception viewer.**
- **16.5 Bug-report bundle.** One-click export of logs, settings (sanitized) and last error trace for support.

## 17. Distribution & Updates (Linux equivalent)

- **17.1 Distribution-channel parity.** Provided as Flatpak (Flathub), AppImage, native `.deb` / `.rpm`, AUR, and a vendor APT repository — corresponding to FairEmail's Play / F-Droid / GitHub / Amazon flavors.
- **17.2 Built-in update check.** For users running the upstream tarball / AppImage, the app can check the project release feed and prompt to update (off by default for distro packages, where the system package manager handles updates).
- **17.3 Reproducible builds.**
- **17.4 Signed releases.** Public signing key fingerprint published; the install verifies the signature on update.

## 18. Power-User & Misc

- **18.1 Per-account default folders** for archive, trash, junk, drafts, sent.
- **18.2 Auto-clean.** Auto-delete trashed messages older than N days; auto-clean local drafts; auto-clean orphan attachments.
- **18.3 Database housekeeping** on a schedule (vacuum, reindex).
- **18.4 Message expiration / TTL** per account or per rule.
- **18.5 Snooze with smart presets** (later today, tomorrow morning, this weekend, next week, custom).
- **18.6 Pin / mark message as starting a project.**
- **18.7 Notes per message** (private, non-syncing).
- **18.8 Per-message keywords / IMAP labels** management (Gmail labels supported natively).
- **18.9 Open-Xchange flag colors** support.
- **18.10 Public Suffix List**, **Brave debounce list**, **Mozilla CA root list**, **Disconnect** and **AdGuard** filter lists are bundled and updated with the application.
- **18.11 Reset onboarding hints** and "don't ask again" choices.
- **18.12 About / legend / shortcut cheatsheet** screen.
- **18.13 Multi-window** support: open multiple compose windows or message readers side-by-side.
