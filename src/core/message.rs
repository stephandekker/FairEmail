//! Message domain model and body-text derivation.

// IMAP flag bitmask constants.
pub const FLAG_SEEN: u32 = 1 << 0;
pub const FLAG_ANSWERED: u32 = 1 << 1;
pub const FLAG_FLAGGED: u32 = 1 << 2;
pub const FLAG_DELETED: u32 = 1 << 3;
pub const FLAG_DRAFT: u32 = 1 << 4;

/// A message record as stored in the database.
#[derive(Debug, Clone)]
pub struct Message {
    pub id: i64,
    pub account_id: String,
    pub uid: u32,
    pub modseq: Option<u64>,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references_header: Option<String>,
    pub from_addresses: Option<String>,
    pub to_addresses: Option<String>,
    pub cc_addresses: Option<String>,
    pub bcc_addresses: Option<String>,
    pub subject: Option<String>,
    pub date_received: Option<i64>,
    pub date_sent: Option<i64>,
    pub flags: u32,
    pub size: u64,
    pub content_hash: String,
    pub body_text: Option<String>,
    pub thread_id: Option<String>,
    pub server_thread_id: Option<String>,
}

/// Parameters for inserting a new message (before the id is assigned).
#[derive(Debug, Clone)]
pub struct NewMessage {
    pub account_id: String,
    pub uid: u32,
    pub modseq: Option<u64>,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references_header: Option<String>,
    pub from_addresses: Option<String>,
    pub to_addresses: Option<String>,
    pub cc_addresses: Option<String>,
    pub bcc_addresses: Option<String>,
    pub subject: Option<String>,
    pub date_received: Option<i64>,
    pub date_sent: Option<i64>,
    pub flags: u32,
    pub size: u64,
    pub content_hash: String,
    pub body_text: Option<String>,
    pub thread_id: Option<String>,
    pub server_thread_id: Option<String>,
}

/// Parse IMAP flag strings into a bitmask.
pub fn flags_from_imap(flag_str: &str) -> u32 {
    let mut flags = 0u32;
    let upper = flag_str.to_uppercase();
    if upper.contains("\\SEEN") {
        flags |= FLAG_SEEN;
    }
    if upper.contains("\\ANSWERED") {
        flags |= FLAG_ANSWERED;
    }
    if upper.contains("\\FLAGGED") {
        flags |= FLAG_FLAGGED;
    }
    if upper.contains("\\DELETED") {
        flags |= FLAG_DELETED;
    }
    if upper.contains("\\DRAFT") {
        flags |= FLAG_DRAFT;
    }
    flags
}

/// Format a single `Addr` as a display string.
fn format_addr(a: &mail_parser::Addr) -> Option<String> {
    match (&a.name, &a.address) {
        (Some(name), Some(email)) => Some(format!("{name} <{email}>")),
        (None, Some(email)) => Some(email.to_string()),
        (Some(name), None) => Some(name.to_string()),
        (None, None) => None,
    }
}

/// Format a `mail_parser::Address` (list or group) as a comma-separated string.
fn format_address(value: &mail_parser::Address) -> Option<String> {
    let parts: Vec<String> = match value {
        mail_parser::Address::List(addrs) => addrs.iter().filter_map(format_addr).collect(),
        mail_parser::Address::Group(groups) => groups
            .iter()
            .flat_map(|g| g.addresses.iter().filter_map(format_addr))
            .collect(),
    };
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

/// Extract a text value from a `HeaderValue` that may be `Text` or `TextList`.
fn header_value_as_text(value: &mail_parser::HeaderValue) -> Option<String> {
    match value {
        mail_parser::HeaderValue::Text(s) => Some(s.to_string()),
        mail_parser::HeaderValue::TextList(list) => {
            if list.is_empty() {
                None
            } else {
                Some(
                    list.iter()
                        .map(|s| s.as_ref())
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            }
        }
        _ => None,
    }
}

/// Parse raw RFC 5322 bytes and extract a `NewMessage`.
///
/// `body_text` derivation: prefers `text/plain`; falls back to `text/html`
/// stripped via `html2text`.
pub fn parse_raw_message(
    account_id: &str,
    uid: u32,
    modseq: Option<u64>,
    flags: u32,
    content_hash: &str,
    raw: &[u8],
) -> NewMessage {
    let parsed = mail_parser::MessageParser::default().parse(raw);

    let (
        message_id,
        in_reply_to,
        references_header,
        from,
        to,
        cc,
        bcc,
        subject,
        date_sent,
        body_text,
    ) = if let Some(msg) = &parsed {
        let message_id = msg.message_id().map(|s| s.to_string());
        let in_reply_to = header_value_as_text(msg.in_reply_to());
        let references = header_value_as_text(msg.references());
        let from = msg.from().and_then(format_address);
        let to = msg.to().and_then(format_address);
        let cc = msg.cc().and_then(format_address);
        let bcc = msg.bcc().and_then(format_address);
        let subject = msg.subject().map(|s| s.to_string());
        let date_sent = msg.date().map(|d| d.to_timestamp());
        let body_text = derive_body_text(msg);

        (
            message_id,
            in_reply_to,
            references,
            from,
            to,
            cc,
            bcc,
            subject,
            date_sent,
            body_text,
        )
    } else {
        (None, None, None, None, None, None, None, None, None, None)
    };

    NewMessage {
        account_id: account_id.to_string(),
        uid,
        modseq,
        message_id,
        in_reply_to,
        references_header,
        from_addresses: from,
        to_addresses: to,
        cc_addresses: cc,
        bcc_addresses: bcc,
        subject,
        date_received: Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        ),
        date_sent,
        flags,
        size: raw.len() as u64,
        content_hash: content_hash.to_string(),
        body_text,
        thread_id: None,
        server_thread_id: None,
    }
}

/// Derive body text: prefer text/plain, fallback to HTML stripped.
pub fn derive_body_text(msg: &mail_parser::Message) -> Option<String> {
    // Prefer text/plain
    if let Some(text) = msg.body_text(0) {
        let t = text.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    // Fallback: text/html → strip HTML
    if let Some(html) = msg.body_html(0) {
        let plain = html2text::from_read(html.as_bytes(), 80);
        let trimmed = plain.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_from_imap_parses_standard_flags() {
        assert_eq!(flags_from_imap("\\Seen"), FLAG_SEEN);
        assert_eq!(flags_from_imap("\\Answered"), FLAG_ANSWERED);
        assert_eq!(flags_from_imap("\\Flagged"), FLAG_FLAGGED);
        assert_eq!(flags_from_imap("\\Deleted"), FLAG_DELETED);
        assert_eq!(flags_from_imap("\\Draft"), FLAG_DRAFT);
    }

    #[test]
    fn flags_from_imap_combines_multiple() {
        let flags = flags_from_imap("\\Seen \\Flagged \\Answered");
        assert_eq!(flags, FLAG_SEEN | FLAG_ANSWERED | FLAG_FLAGGED);
    }

    #[test]
    fn flags_from_imap_empty() {
        assert_eq!(flags_from_imap(""), 0);
    }

    #[test]
    fn body_text_prefers_plain_over_html() {
        let raw = b"From: test@example.com\r\n\
                     Subject: Test\r\n\
                     MIME-Version: 1.0\r\n\
                     Content-Type: multipart/alternative; boundary=\"bound\"\r\n\
                     \r\n\
                     --bound\r\n\
                     Content-Type: text/plain\r\n\
                     \r\n\
                     Plain text body\r\n\
                     --bound\r\n\
                     Content-Type: text/html\r\n\
                     \r\n\
                     <html><body><b>HTML body</b></body></html>\r\n\
                     --bound--\r\n";
        let msg = mail_parser::MessageParser::default().parse(raw).unwrap();
        let body = derive_body_text(&msg).unwrap();
        assert_eq!(body, "Plain text body");
    }

    #[test]
    fn body_text_falls_back_to_html_stripped() {
        let raw = b"From: test@example.com\r\n\
                     Subject: Test\r\n\
                     MIME-Version: 1.0\r\n\
                     Content-Type: text/html\r\n\
                     \r\n\
                     <html><body><b>Bold text</b> and <i>italic</i></body></html>\r\n";
        let msg = mail_parser::MessageParser::default().parse(raw).unwrap();
        let body = derive_body_text(&msg).unwrap();
        // html2text strips tags
        assert!(!body.contains("<b>"));
        assert!(!body.contains("<i>"));
        assert!(body.contains("Bold text"));
        assert!(body.contains("italic"));
    }

    #[test]
    fn parse_raw_message_extracts_headers() {
        let raw = b"From: Alice <alice@example.com>\r\n\
                     To: Bob <bob@example.com>\r\n\
                     Cc: Carol <carol@example.com>\r\n\
                     Subject: Hello World\r\n\
                     Message-ID: <msg001@example.com>\r\n\
                     In-Reply-To: <parent@example.com>\r\n\
                     Date: Mon, 1 Jan 2024 12:00:00 +0000\r\n\
                     \r\n\
                     Body text here.\r\n";
        let msg = parse_raw_message("acct-1", 42, Some(100), FLAG_SEEN, "abcdef", raw);
        assert_eq!(msg.uid, 42);
        assert_eq!(msg.modseq, Some(100));
        assert_eq!(msg.flags, FLAG_SEEN);
        assert_eq!(msg.content_hash, "abcdef");
        assert_eq!(msg.message_id.as_deref(), Some("msg001@example.com"));
        assert_eq!(msg.in_reply_to.as_deref(), Some("parent@example.com"));
        assert!(msg
            .from_addresses
            .as_ref()
            .unwrap()
            .contains("alice@example.com"));
        assert!(msg
            .to_addresses
            .as_ref()
            .unwrap()
            .contains("bob@example.com"));
        assert!(msg
            .cc_addresses
            .as_ref()
            .unwrap()
            .contains("carol@example.com"));
        assert_eq!(msg.subject.as_deref(), Some("Hello World"));
        assert!(msg.body_text.as_ref().unwrap().contains("Body text here"));
    }
}
