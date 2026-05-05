# Email Client Backend Architecture

A recommended storage and sync architecture for a green-field email client in 2026.

## Recommendation

Use **SQLite as the primary store, with raw `.eml` files for the message bodies on the filesystem**. This is the modern hybrid approach: the durability and recoverability of file-per-message storage, plus the speed and query power of a real database for everything the UI touches.

## Core Architecture

Store each received message as an `.eml` file (raw RFC 5322 bytes) in a content-addressed directory tree. Name files by the SHA-256 of the message and shard them into subdirectories so no single folder gets too large.

In SQLite, keep a `messages` table containing:

- Parsed headers (`From`, `To`, `Subject`, `Date`, `Message-ID`)
- Folder and label associations
- IMAP/JMAP `UID` and `ModSeq` values
- Flags (read, flagged, answered, etc.)
- A pointer to the corresponding `.eml` file on disk

Add an FTS5 virtual table for full-text search over subjects and bodies. Store attachments on disk as well, deduplicated by hash and referenced from the database.

This gives you the best of both worlds:

- The **database** is the index and the source of truth for *state* (read/unread, labels, threading).
- The **filesystem** is the source of truth for *content* — a corrupt database can be rebuilt by re-parsing the `.eml` files, and users can grep, back up, or migrate their mail with standard tools.

## Things to Commit to Early

### Database setup

Enable SQLite's WAL mode and turn on foreign keys. Use FTS5, not the older FTS variants.

### Sync semantics

Plan your schema around IMAP's CONDSTORE/QRESYNC extensions (RFC 7162). Store `MODSEQ` per message and per mailbox so incremental sync is cheap. Bolting this on later is painful.

### Process architecture

Separate the sync engine from the UI. A background process (or thread) owns all network I/O — IMAP, SMTP, JMAP, OAuth refresh — and writes to SQLite. The UI only reads from SQLite and listens for change notifications. This is how Mailspring, Apple Mail, and most modern clients are built. A user click should never block on the network.

### Protocols

Plan for JMAP (RFC 8620/8621), not just IMAP. JMAP is a much saner protocol — JSON, batched, push-native, designed for app developers rather than 1990s mail readers. Fastmail supports it natively, and adoption is growing.

Build your data model close to JMAP's and adapt IMAP to it, rather than the other way around. You'll also want:

- **Microsoft Graph** for modern Outlook/M365 accounts (many no longer expose IMAP cleanly)
- **Gmail API** for Gmail-specific features like labels
- **OAuth 2.0** for both of the above

### Credentials and encryption

Encrypt credentials in the OS keychain — `libsecret` on Linux, Keychain on macOS, DPAPI/Credential Manager on Windows. Never store them in your SQLite file.

## Rebuilding the Index

If the SQLite database (`fairmail.db`) is corrupted or deleted, the message index
can be reconstructed from the on-disk `.eml` files using the `--rebuild-index` CLI
flag:

```bash
fairmail --rebuild-index
```

### What it does

1. Walks `$XDG_DATA_HOME/fairmail/messages/` (or `$FAIRMAIL_DATA_DIR/messages/`)
   and finds every `.eml` file.
2. For each file, computes the SHA-256 hash and verifies it matches the filename.
3. Parses headers with `mail_parser` and derives body text.
4. Inserts (or skips, if already present) a row in the `messages` table and the
   FTS5 full-text search index.
5. Uses the `X-Folder` header, if present, to assign the message to the correct
   folder. Messages without this header are placed in a synthetic "Recovered"
   folder per account.

### What it does NOT reconstruct

- **IMAP UIDs and MODSEQ values** — these are server-assigned. The next
  incremental sync will re-establish UID mapping.
- **Flags** (read/unread, starred, etc.) — flags are reset to zero. The next
  sync will pull current flags from the server.
- **Folder associations** — only the `X-Folder` hint is used. Accurate folder
  placement is restored on the next sync via UID matching.

### Requirements

- At least one account must exist in the database (or have been re-imported from
  `accounts.json.migrated` via account setup).
- The operation is **non-destructive**: `.eml` files are only read, never
  modified or deleted.
- The operation is **idempotent**: running it twice produces the same result
  (upserts on `content_hash`, no duplicate rows).

## What to Avoid

- **mbox** — rewrite-on-delete behavior makes it unworkable above a few thousand messages.
- **Heavyweight databases** (PostgreSQL, MySQL) for a desktop app — the operational burden isn't worth it.
- **Custom binary formats** — you will regret it.
- **Filesystem walks for search** — you need an index from day one.


