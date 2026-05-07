//! Core logic for detecting new messages on the server that are not yet in
//! the local store.  This module contains pure, UI-free business logic that
//! can be unit-tested without a display server or network connection.
//!
//! Primary drivers: US-10, FR-5, FR-6, AC-8.

use std::collections::HashSet;

use crate::core::message::{flags_from_imap, parse_raw_message, NewMessage};
use crate::core::sync_event::SyncEvent;

/// Policy that governs how message bodies are downloaded during sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadPolicy {
    /// Download the full message (envelope + body + attachments).
    #[default]
    Full,
    /// Download only the envelope / headers; body fetched on user request.
    HeadersOnly,
    /// No automatic download at all; everything fetched on explicit user action.
    OnDemand,
}

/// A single new message detected on the server that is absent locally.
#[derive(Debug)]
pub struct DetectedNewMessage {
    pub uid: u32,
    pub flags: u32,
    pub modseq: Option<u64>,
    /// Parsed message ready for database insertion (present when body was
    /// downloaded, i.e. `DownloadPolicy::Full`).
    pub new_message: Option<NewMessage>,
    /// Raw body bytes, if available.
    pub body: Option<Vec<u8>>,
    /// Content hash assigned by the content store (set when body was stored).
    pub content_hash: Option<String>,
}

/// Result of the new-message detection phase.
#[derive(Debug)]
pub struct DetectNewMessagesResult {
    /// Messages that were detected as new and processed.
    pub detected: Vec<DetectedNewMessage>,
    /// Number of full bodies that were fetched and stored.
    pub bodies_fetched: usize,
    /// Number of header-only stubs created.
    pub headers_only: usize,
    /// Sync events to broadcast.
    pub events: Vec<SyncEvent>,
}

/// Compare server UIDs against local UIDs to find new messages.
///
/// Returns the set of UIDs present on the server but absent locally.
pub fn find_new_uids(server_uids: &[u32], local_uids: &[u32]) -> Vec<u32> {
    let local_set: HashSet<u32> = local_uids.iter().copied().collect();
    server_uids
        .iter()
        .copied()
        .filter(|uid| !local_set.contains(uid))
        .collect()
}

/// Determines whether a body should be downloaded for a new message based
/// on the configured download policy.
pub fn should_download_body(policy: DownloadPolicy) -> bool {
    matches!(policy, DownloadPolicy::Full)
}

/// Representation of a raw message fetched from the server (for processing).
#[derive(Debug, Clone)]
pub struct RawNewMessage {
    pub uid: u32,
    pub flags_str: String,
    pub modseq: Option<u64>,
    pub body: Option<Vec<u8>>,
}

/// Process a batch of raw new messages into `DetectedNewMessage` entries.
///
/// When `policy` is `Full`, the body bytes are parsed into a `NewMessage`
/// suitable for database insertion.  When `HeadersOnly` or `OnDemand`, the
/// message is recorded as a stub (uid + flags only) so the UI can display
/// the envelope without the body.
pub fn process_new_messages(
    account_id: &str,
    folder_name: &str,
    raw_messages: &[RawNewMessage],
    policy: DownloadPolicy,
) -> DetectNewMessagesResult {
    let mut detected = Vec::with_capacity(raw_messages.len());
    let mut bodies_fetched: usize = 0;
    let mut headers_only: usize = 0;

    for raw in raw_messages {
        let flags = flags_from_imap(&raw.flags_str);

        match (should_download_body(policy), &raw.body) {
            // Full download and body is available.
            (true, Some(body)) => {
                let new_msg = parse_raw_message(
                    account_id, raw.uid, raw.modseq, flags,
                    "", // content_hash filled in by caller after storing
                    body,
                );
                detected.push(DetectedNewMessage {
                    uid: raw.uid,
                    flags,
                    modseq: raw.modseq,
                    new_message: Some(new_msg),
                    body: Some(body.clone()),
                    content_hash: None,
                });
                bodies_fetched += 1;
            }
            // Full download requested but no body available — skip.
            (true, None) => {}
            // Headers-only or on-demand: record stub.
            (false, _) => {
                // Even in headers-only mode, if we have the body data we can
                // parse the envelope from it; we just won't store the body.
                let new_msg = raw.body.as_ref().map(|body| {
                    parse_raw_message(account_id, raw.uid, raw.modseq, flags, "", body)
                });
                detected.push(DetectedNewMessage {
                    uid: raw.uid,
                    flags,
                    modseq: raw.modseq,
                    new_message: new_msg,
                    body: None,
                    content_hash: None,
                });
                headers_only += 1;
            }
        }
    }

    let events = if bodies_fetched > 0 || headers_only > 0 {
        vec![SyncEvent::NewMailReceived {
            account_id: account_id.to_string(),
            folder_name: folder_name.to_string(),
            bodies_fetched,
        }]
    } else {
        Vec::new()
    };

    DetectNewMessagesResult {
        detected,
        bodies_fetched,
        headers_only,
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_new_uids_detects_server_only() {
        let server = vec![1, 2, 3, 4, 5];
        let local = vec![1, 2, 3];
        let new = find_new_uids(&server, &local);
        assert_eq!(new, vec![4, 5]);
    }

    #[test]
    fn find_new_uids_empty_when_in_sync() {
        let server = vec![1, 2, 3];
        let local = vec![1, 2, 3];
        let new = find_new_uids(&server, &local);
        assert!(new.is_empty());
    }

    #[test]
    fn find_new_uids_all_new_on_empty_local() {
        let server = vec![10, 20, 30];
        let local: Vec<u32> = vec![];
        let new = find_new_uids(&server, &local);
        assert_eq!(new, vec![10, 20, 30]);
    }

    #[test]
    fn find_new_uids_empty_server() {
        let server: Vec<u32> = vec![];
        let local = vec![1, 2, 3];
        let new = find_new_uids(&server, &local);
        assert!(new.is_empty());
    }

    #[test]
    fn should_download_body_full_policy() {
        assert!(should_download_body(DownloadPolicy::Full));
    }

    #[test]
    fn should_download_body_headers_only_policy() {
        assert!(!should_download_body(DownloadPolicy::HeadersOnly));
    }

    #[test]
    fn should_download_body_on_demand_policy() {
        assert!(!should_download_body(DownloadPolicy::OnDemand));
    }

    fn make_raw_email(subject: &str) -> Vec<u8> {
        format!(
            "From: test@example.com\r\n\
             Subject: {subject}\r\n\
             Message-ID: <{subject}@example.com>\r\n\
             Date: Mon, 1 Jan 2024 12:00:00 +0000\r\n\
             \r\n\
             Body of {subject}\r\n"
        )
        .into_bytes()
    }

    #[test]
    fn process_new_messages_full_policy_with_body() {
        let raw = vec![RawNewMessage {
            uid: 10,
            flags_str: "\\Seen".to_string(),
            modseq: Some(100),
            body: Some(make_raw_email("test1")),
        }];

        let result = process_new_messages("acct-1", "INBOX", &raw, DownloadPolicy::Full);
        assert_eq!(result.bodies_fetched, 1);
        assert_eq!(result.headers_only, 0);
        assert_eq!(result.detected.len(), 1);
        assert!(result.detected[0].new_message.is_some());
        assert!(result.detected[0].body.is_some());
        assert_eq!(result.events.len(), 1);
        match &result.events[0] {
            SyncEvent::NewMailReceived {
                account_id,
                folder_name,
                bodies_fetched,
            } => {
                assert_eq!(account_id, "acct-1");
                assert_eq!(folder_name, "INBOX");
                assert_eq!(*bodies_fetched, 1);
            }
            _ => panic!("expected NewMailReceived event"),
        }
    }

    #[test]
    fn process_new_messages_full_policy_no_body_skipped() {
        let raw = vec![RawNewMessage {
            uid: 10,
            flags_str: String::new(),
            modseq: None,
            body: None,
        }];

        let result = process_new_messages("acct-1", "INBOX", &raw, DownloadPolicy::Full);
        assert_eq!(result.bodies_fetched, 0);
        assert_eq!(result.detected.len(), 0);
        assert!(result.events.is_empty());
    }

    #[test]
    fn process_new_messages_headers_only_creates_stub() {
        let raw = vec![RawNewMessage {
            uid: 20,
            flags_str: "\\Flagged".to_string(),
            modseq: Some(200),
            body: Some(make_raw_email("stub")),
        }];

        let result = process_new_messages("acct-1", "INBOX", &raw, DownloadPolicy::HeadersOnly);
        assert_eq!(result.bodies_fetched, 0);
        assert_eq!(result.headers_only, 1);
        assert_eq!(result.detected.len(), 1);
        // Envelope parsed from body but body not stored.
        assert!(result.detected[0].new_message.is_some());
        assert!(result.detected[0].body.is_none());
        assert_eq!(result.events.len(), 1);
    }

    #[test]
    fn process_new_messages_on_demand_creates_stub() {
        let raw = vec![RawNewMessage {
            uid: 30,
            flags_str: String::new(),
            modseq: None,
            body: Some(make_raw_email("ondemand")),
        }];

        let result = process_new_messages("acct-1", "Sent", &raw, DownloadPolicy::OnDemand);
        assert_eq!(result.bodies_fetched, 0);
        assert_eq!(result.headers_only, 1);
        assert_eq!(result.detected.len(), 1);
        assert!(result.detected[0].body.is_none());
    }

    #[test]
    fn process_new_messages_multiple_mixed() {
        let raw = vec![
            RawNewMessage {
                uid: 1,
                flags_str: String::new(),
                modseq: Some(10),
                body: Some(make_raw_email("msg1")),
            },
            RawNewMessage {
                uid: 2,
                flags_str: "\\Seen".to_string(),
                modseq: Some(20),
                body: Some(make_raw_email("msg2")),
            },
            RawNewMessage {
                uid: 3,
                flags_str: String::new(),
                modseq: Some(30),
                body: Some(make_raw_email("msg3")),
            },
        ];

        let result = process_new_messages("acct-1", "INBOX", &raw, DownloadPolicy::Full);
        assert_eq!(result.bodies_fetched, 3);
        assert_eq!(result.detected.len(), 3);
        assert_eq!(result.events.len(), 1);
    }

    #[test]
    fn process_new_messages_empty_input() {
        let result = process_new_messages("acct-1", "INBOX", &[], DownloadPolicy::Full);
        assert_eq!(result.bodies_fetched, 0);
        assert_eq!(result.headers_only, 0);
        assert!(result.detected.is_empty());
        assert!(result.events.is_empty());
    }

    #[test]
    fn no_duplicates_when_all_uids_already_local() {
        // Simulate: server has UIDs 1-5, local already has all of them.
        let server = vec![1, 2, 3, 4, 5];
        let local = vec![1, 2, 3, 4, 5];
        let new = find_new_uids(&server, &local);
        assert!(new.is_empty(), "no new UIDs should be detected");
    }

    #[test]
    fn default_download_policy_is_full() {
        assert_eq!(DownloadPolicy::default(), DownloadPolicy::Full);
    }

    #[test]
    fn flags_preserved_on_new_messages() {
        use crate::core::message::{FLAG_FLAGGED, FLAG_SEEN};

        let raw = vec![RawNewMessage {
            uid: 42,
            flags_str: "\\Seen \\Flagged".to_string(),
            modseq: Some(99),
            body: Some(make_raw_email("flagged")),
        }];

        let result = process_new_messages("acct-1", "INBOX", &raw, DownloadPolicy::Full);
        assert_eq!(result.detected.len(), 1);
        assert_eq!(result.detected[0].flags, FLAG_SEEN | FLAG_FLAGGED);
        let msg = result.detected[0].new_message.as_ref().unwrap();
        assert_eq!(msg.flags, FLAG_SEEN | FLAG_FLAGGED);
    }

    #[test]
    fn new_message_placed_in_correct_folder_event() {
        let raw = vec![RawNewMessage {
            uid: 1,
            flags_str: String::new(),
            modseq: None,
            body: Some(make_raw_email("foldertest")),
        }];

        let result = process_new_messages("acct-1", "Drafts", &raw, DownloadPolicy::Full);
        match &result.events[0] {
            SyncEvent::NewMailReceived { folder_name, .. } => {
                assert_eq!(folder_name, "Drafts");
            }
            _ => panic!("expected NewMailReceived"),
        }
    }
}
