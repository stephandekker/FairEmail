// Detection progress feedback for the setup wizard (FR-14, US-10).
//
// This module defines the progress steps reported during provider detection
// and connectivity checking. It is UI-free so it can be unit-tested without
// a display server.

/// The detection strategies / phases that the wizard cycles through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionStep {
    /// Looking up the provider in the bundled database.
    LookupBundledDatabase,
    /// Performing DNS record lookups for the domain.
    DnsLookup,
    /// Querying autoconfig services (e.g. Mozilla ISPDB).
    CheckAutoconfig,
    /// Scanning ports on a specific host.
    ScanPorts { host: String },
    /// Connecting to the IMAP server.
    ConnectingImap,
    /// Connecting to the SMTP server.
    ConnectingSmtp,
    /// Authenticating with the server.
    Authenticating,
}

impl DetectionStep {
    /// Returns a user-facing description of the current step.
    /// These strings are marked for translation via gettext.
    pub fn message(&self) -> String {
        match self {
            Self::LookupBundledDatabase => gettextrs::gettext("Looking up provider database..."),
            Self::DnsLookup => gettextrs::gettext("Looking up DNS records..."),
            Self::CheckAutoconfig => gettextrs::gettext("Checking autoconfig..."),
            Self::ScanPorts { host } => {
                gettextrs::gettext("Scanning ports on %s...").replace("%s", host)
            }
            Self::ConnectingImap => gettextrs::gettext("Connecting to IMAP server..."),
            Self::ConnectingSmtp => gettextrs::gettext("Connecting to SMTP server..."),
            Self::Authenticating => gettextrs::gettext("Authenticating..."),
        }
    }

    /// Returns an accessible description suitable for screen readers (NFR-8).
    /// This provides additional context beyond the visible label text.
    pub fn accessible_description(&self) -> String {
        match self {
            Self::LookupBundledDatabase => {
                gettextrs::gettext("Checking bundled provider database for known configuration")
            }
            Self::DnsLookup => {
                gettextrs::gettext("Performing DNS record lookup to discover mail server settings")
            }
            Self::CheckAutoconfig => {
                gettextrs::gettext("Querying autoconfig service for server configuration")
            }
            Self::ScanPorts { host } => {
                gettextrs::gettext("Scanning network ports on %s to detect available services")
                    .replace("%s", host)
            }
            Self::ConnectingImap => {
                gettextrs::gettext("Establishing connection to incoming mail server")
            }
            Self::ConnectingSmtp => {
                gettextrs::gettext("Establishing connection to outgoing mail server")
            }
            Self::Authenticating => {
                gettextrs::gettext("Verifying credentials with the mail server")
            }
        }
    }
}

/// Callback type that detection strategies invoke to report progress.
/// The UI layer provides a concrete implementation that updates widgets.
#[allow(dead_code)]
pub type ProgressCallback = Box<dyn Fn(DetectionStep) + 'static>;

/// The ordered sequence of detection steps for a full auto-detection run.
/// Used by the detection pipeline to report progress in order.
pub fn detection_sequence(host: &str) -> Vec<DetectionStep> {
    vec![
        DetectionStep::LookupBundledDatabase,
        DetectionStep::DnsLookup,
        DetectionStep::CheckAutoconfig,
        DetectionStep::ScanPorts {
            host: host.to_string(),
        },
        DetectionStep::ConnectingImap,
        DetectionStep::ConnectingSmtp,
        DetectionStep::Authenticating,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_step_messages_are_non_empty() {
        let steps = detection_sequence("mail.example.com");
        for step in &steps {
            let msg = step.message();
            assert!(!msg.is_empty(), "Step {step:?} has empty message");
        }
    }

    #[test]
    fn test_detection_step_accessible_descriptions_are_non_empty() {
        let steps = detection_sequence("mail.example.com");
        for step in &steps {
            let desc = step.accessible_description();
            assert!(
                !desc.is_empty(),
                "Step {step:?} has empty accessible description"
            );
        }
    }

    #[test]
    fn test_scan_ports_includes_host() {
        let step = DetectionStep::ScanPorts {
            host: "imap.gmail.com".to_string(),
        };
        assert!(step.message().contains("imap.gmail.com"));
        assert!(step.accessible_description().contains("imap.gmail.com"));
    }

    #[test]
    fn test_detection_sequence_order() {
        let seq = detection_sequence("example.com");
        assert_eq!(seq.len(), 7);
        assert_eq!(seq[0], DetectionStep::LookupBundledDatabase);
        assert_eq!(seq[1], DetectionStep::DnsLookup);
        assert_eq!(seq[2], DetectionStep::CheckAutoconfig);
        assert_eq!(
            seq[3],
            DetectionStep::ScanPorts {
                host: "example.com".to_string()
            }
        );
        assert_eq!(seq[4], DetectionStep::ConnectingImap);
        assert_eq!(seq[5], DetectionStep::ConnectingSmtp);
        assert_eq!(seq[6], DetectionStep::Authenticating);
    }

    #[test]
    fn test_progress_callback_type_is_callable() {
        let called = std::cell::Cell::new(false);
        let cb: Box<dyn Fn(DetectionStep)> = Box::new(|_step| {
            called.set(true);
        });
        cb(DetectionStep::DnsLookup);
        assert!(called.get());
    }

    #[test]
    fn test_detection_steps_have_distinct_messages() {
        let steps = detection_sequence("host.example.com");
        let messages: Vec<String> = steps.iter().map(|s| s.message()).collect();
        for (i, msg) in messages.iter().enumerate() {
            for (j, other) in messages.iter().enumerate() {
                if i != j {
                    assert_ne!(msg, other, "Steps {i} and {j} have identical messages");
                }
            }
        }
    }
}
