# Provider Database Format (NFR-5)

FairEmail uses a JSON-based provider database format. The bundled database is
compiled into the application binary, but users can supply an additional file
to add new providers or override existing entries.

## File Location

The user-supplied provider file is loaded from:

```
$XDG_CONFIG_HOME/fairmail/providers.json
```

If `$XDG_CONFIG_HOME` is not set, it defaults to:

```
~/.config/fairmail/providers.json
```

If the file does not exist, the application uses only the bundled database.

## Format

The file contains a JSON array of provider objects:

```json
[
  {
    "id": "corpmail",
    "display_name": "Corporate Mail",
    "domain_patterns": ["corp.example.com", "*.corp.example.com"],
    "mx_patterns": ["mx.corp.example.com"],
    "incoming": {
      "hostname": "imap.corp.example.com",
      "port": 993,
      "encryption": "SslTls"
    },
    "outgoing": {
      "hostname": "smtp.corp.example.com",
      "port": 465,
      "encryption": "SslTls"
    },
    "username_type": "EmailAddress",
    "keep_alive_interval": 15,
    "noop_keep_alive": false,
    "partial_fetch": true,
    "max_tls_version": "Tls1_3",
    "app_password_required": false,
    "documentation_url": null,
    "localized_docs": [],
    "oauth": null,
    "display_order": 200,
    "enabled": true
  }
]
```

## Field Reference

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique identifier for the provider |
| `display_name` | string | Human-readable name shown in the UI |
| `domain_patterns` | string[] | Domain patterns for matching. Supports `*.example.com` wildcards |
| `mx_patterns` | string[] | MX record patterns for DNS-based matching |
| `incoming` | object | IMAP server configuration |
| `incoming.hostname` | string | IMAP server hostname |
| `incoming.port` | integer | IMAP server port (typically 993) |
| `incoming.encryption` | string | One of: `"SslTls"`, `"StartTls"`, `"None"` |
| `outgoing` | object | SMTP server configuration |
| `outgoing.hostname` | string | SMTP server hostname |
| `outgoing.port` | integer | SMTP server port (typically 465 or 587) |
| `outgoing.encryption` | string | One of: `"SslTls"`, `"StartTls"`, `"None"` |
| `username_type` | string | `"EmailAddress"` (full email) or `"LocalPart"` (user only) |
| `keep_alive_interval` | integer | IMAP keep-alive interval in minutes |
| `noop_keep_alive` | boolean | Whether to use NOOP command for keep-alive |
| `partial_fetch` | boolean | Whether the server supports partial message fetch |
| `max_tls_version` | string | `"Tls1_2"` or `"Tls1_3"` |
| `app_password_required` | boolean | Whether an app-specific password is required |
| `documentation_url` | string or null | Link to provider setup documentation |
| `localized_docs` | array | Localized documentation snippets (see below) |
| `oauth` | object or null | OAuth configuration (see below) |
| `display_order` | integer | Display priority (lower = higher priority) |
| `enabled` | boolean | Whether this provider entry is active |

### Localized Documentation

```json
{
  "locale": "en",
  "text": "Enable IMAP access in Settings > Forwarding and POP/IMAP"
}
```

### OAuth Configuration

```json
{
  "auth_url": "https://accounts.example.com/o/oauth2/auth",
  "token_url": "https://accounts.example.com/o/oauth2/token",
  "scopes": ["https://mail.example.com/"],
  "client_id": null
}
```

## Merge Behavior

- If a user-supplied entry has the same `id` as a bundled entry, the
  user-supplied entry **completely replaces** the bundled entry.
- Entries with new IDs are **added** to the database.
- All entries (bundled and user-supplied) participate equally in
  domain-matching and score-based ranking.

## Domain Matching

Providers are matched by email domain:

1. **Exact match**: `domain_patterns` contains the exact domain (score: 1.0)
2. **Wildcard match**: `domain_patterns` contains `*.parent.domain` and the
   email domain is a subdomain (score: 0.9)

## Examples

### Adding a Corporate Mail Server

```json
[
  {
    "id": "acme-corp",
    "display_name": "ACME Corporation",
    "domain_patterns": ["acme.com", "*.acme.com"],
    "mx_patterns": [],
    "incoming": {
      "hostname": "mail.acme.com",
      "port": 993,
      "encryption": "SslTls"
    },
    "outgoing": {
      "hostname": "mail.acme.com",
      "port": 465,
      "encryption": "SslTls"
    },
    "username_type": "EmailAddress",
    "keep_alive_interval": 15,
    "noop_keep_alive": false,
    "partial_fetch": true,
    "max_tls_version": "Tls1_3",
    "app_password_required": false,
    "documentation_url": "https://wiki.acme.com/email-setup",
    "localized_docs": [],
    "oauth": null,
    "display_order": 100,
    "enabled": true
  }
]
```

### Overriding a Bundled Provider

To override Gmail's settings (e.g., to use a corporate Google Workspace
with custom hostnames), use `"id": "gmail"`:

```json
[
  {
    "id": "gmail",
    "display_name": "Gmail (Custom)",
    "domain_patterns": ["gmail.com", "googlemail.com", "workspace.acme.com"],
    "mx_patterns": [],
    "incoming": {
      "hostname": "imap.gmail.com",
      "port": 993,
      "encryption": "SslTls"
    },
    "outgoing": {
      "hostname": "smtp.gmail.com",
      "port": 465,
      "encryption": "SslTls"
    },
    "username_type": "EmailAddress",
    "keep_alive_interval": 15,
    "noop_keep_alive": false,
    "partial_fetch": true,
    "max_tls_version": "Tls1_3",
    "app_password_required": false,
    "documentation_url": null,
    "localized_docs": [],
    "oauth": null,
    "display_order": 1,
    "enabled": true
  }
]
```
