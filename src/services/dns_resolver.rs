//! DNSSEC-validating DNS resolution and DANE (TLSA) record lookup.
//!
//! Provides helpers used by the IMAP client when the user enables the
//! DNSSEC and/or DANE security toggles on an account.

use std::net::SocketAddr;

use hickory_proto::rr::rdata::tlsa::TLSA;
use hickory_proto::rr::RecordType;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::ResolveError;
use hickory_resolver::TokioResolver;

/// Errors that can occur during DNSSEC/DANE operations.
#[derive(Debug, thiserror::Error)]
pub enum DnsSecurityError {
    #[error("DNS resolution failed for {host}: {reason}")]
    ResolutionFailed { host: String, reason: String },
    #[error("DNSSEC validation failed for {host}: response not authenticated")]
    DnssecValidationFailed { host: String },
    #[error("no TLSA records found for {host}:{port}")]
    NoTlsaRecords { host: String, port: u16 },
}

fn build_dnssec_resolver() -> TokioResolver {
    let mut opts = ResolverOpts::default();
    opts.validate = true; // require DNSSEC validation
    TokioResolver::builder_with_config(ResolverConfig::default(), Default::default())
        .with_options(opts)
        .build()
}

/// Resolve a hostname with DNSSEC validation.
///
/// Returns the first resolved `SocketAddr` or a `DnsSecurityError` if resolution
/// fails or the response cannot be DNSSEC-validated.
pub fn resolve_with_dnssec(host: &str, port: u16) -> Result<SocketAddr, DnsSecurityError> {
    let resolver = build_dnssec_resolver();

    let rt = tokio::runtime::Handle::try_current();
    let response = match rt {
        Ok(handle) => {
            // Already inside a tokio runtime — use block_in_place.
            tokio::task::block_in_place(|| handle.block_on(resolver.lookup_ip(host)))
        }
        Err(_) => {
            // No runtime — create a temporary one.
            let rt =
                tokio::runtime::Runtime::new().map_err(|e| DnsSecurityError::ResolutionFailed {
                    host: host.to_string(),
                    reason: e.to_string(),
                })?;
            rt.block_on(resolver.lookup_ip(host))
        }
    }
    .map_err(|e| map_resolve_error(host, e))?;

    let ip = response
        .iter()
        .next()
        .ok_or_else(|| DnsSecurityError::ResolutionFailed {
            host: host.to_string(),
            reason: "no addresses returned".to_string(),
        })?;

    Ok(SocketAddr::new(ip, port))
}

/// Look up TLSA records for a given host and port.
///
/// Queries `_port._tcp.host` for TLSA records using a DNSSEC-validating
/// resolver (DANE requires DNSSEC-signed TLSA records).
pub fn lookup_tlsa(host: &str, port: u16) -> Result<Vec<TLSA>, DnsSecurityError> {
    let resolver = build_dnssec_resolver();
    let tlsa_name = format!("_{port}._tcp.{host}");

    let rt = tokio::runtime::Handle::try_current();
    let response = match rt {
        Ok(handle) => tokio::task::block_in_place(|| {
            handle.block_on(resolver.lookup(&tlsa_name, RecordType::TLSA))
        }),
        Err(_) => {
            let rt =
                tokio::runtime::Runtime::new().map_err(|e| DnsSecurityError::ResolutionFailed {
                    host: host.to_string(),
                    reason: e.to_string(),
                })?;
            rt.block_on(resolver.lookup(&tlsa_name, RecordType::TLSA))
        }
    }
    .map_err(|e| map_resolve_error(host, e))?;

    let records: Vec<TLSA> = response
        .record_iter()
        .filter_map(|r| {
            if let hickory_proto::rr::RData::TLSA(tlsa) = r.data() {
                Some(tlsa.clone())
            } else {
                None
            }
        })
        .collect();

    if records.is_empty() {
        return Err(DnsSecurityError::NoTlsaRecords {
            host: host.to_string(),
            port,
        });
    }

    Ok(records)
}

/// Verify a DER-encoded certificate against a set of TLSA records.
///
/// Supports the most common DANE-EE (usage 3) and DANE-TA (usage 2) modes
/// with SHA-256 (matching type 1) and SHA-512 (matching type 2) hashes,
/// as well as exact match (matching type 0).
pub fn verify_certificate_against_tlsa(cert_der: &[u8], tlsa_records: &[TLSA]) -> bool {
    use hickory_proto::rr::rdata::tlsa::{CertUsage, Matching, Selector};

    for tlsa in tlsa_records {
        // We support DANE-EE (3) and DANE-TA (2) — both match against the
        // end-entity certificate presented by the server.
        match tlsa.cert_usage() {
            CertUsage::DaneEe | CertUsage::DaneTa => {}
            // PKIX-TA (0) and PKIX-EE (1) require full CA chain validation
            // which is already handled by the TLS library; skip here.
            _ => continue,
        }

        let data_to_match = match tlsa.selector() {
            Selector::Full => cert_der.to_vec(),
            // SubjectPublicKeyInfo extraction is complex; for now we only
            // support full-certificate matching (selector 0).
            _ => continue,
        };

        let matches = match tlsa.matching() {
            Matching::Raw => data_to_match == tlsa.cert_data(),
            Matching::Sha256 => {
                use sha2::{Digest, Sha256};
                let hash = Sha256::digest(&data_to_match);
                hash.as_slice() == tlsa.cert_data()
            }
            Matching::Sha512 => {
                use sha2::{Digest, Sha512};
                let hash = Sha512::digest(&data_to_match);
                hash.as_slice() == tlsa.cert_data()
            }
            _ => continue,
        };

        if matches {
            return true;
        }
    }

    false
}

fn map_resolve_error(host: &str, e: ResolveError) -> DnsSecurityError {
    let msg = e.to_string();
    if msg.contains("DNSSEC") || msg.contains("dnssec") || msg.contains("no DNSKEY") {
        DnsSecurityError::DnssecValidationFailed {
            host: host.to_string(),
        }
    } else {
        DnsSecurityError::ResolutionFailed {
            host: host.to_string(),
            reason: msg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hickory_proto::rr::rdata::tlsa::{CertUsage, Matching, Selector, TLSA};

    #[test]
    fn verify_tlsa_sha256_exact_match() {
        use sha2::{Digest, Sha256};
        let cert_der = b"fake-certificate-data";
        let hash = Sha256::digest(cert_der);

        let tlsa = TLSA::new(
            CertUsage::DaneEe,
            Selector::Full,
            Matching::Sha256,
            hash.to_vec(),
        );

        assert!(verify_certificate_against_tlsa(cert_der, &[tlsa]));
    }

    #[test]
    fn verify_tlsa_sha256_mismatch() {
        let cert_der = b"fake-certificate-data";
        let wrong_hash = vec![0u8; 32];

        let tlsa = TLSA::new(
            CertUsage::DaneEe,
            Selector::Full,
            Matching::Sha256,
            wrong_hash,
        );

        assert!(!verify_certificate_against_tlsa(cert_der, &[tlsa]));
    }

    #[test]
    fn verify_tlsa_raw_match() {
        let cert_der = b"raw-certificate-bytes";

        let tlsa = TLSA::new(
            CertUsage::DaneEe,
            Selector::Full,
            Matching::Raw,
            cert_der.to_vec(),
        );

        assert!(verify_certificate_against_tlsa(cert_der, &[tlsa]));
    }

    #[test]
    fn verify_tlsa_sha512_match() {
        use sha2::{Digest, Sha512};
        let cert_der = b"test-cert-for-sha512";
        let hash = Sha512::digest(cert_der);

        let tlsa = TLSA::new(
            CertUsage::DaneEe,
            Selector::Full,
            Matching::Sha512,
            hash.to_vec(),
        );

        assert!(verify_certificate_against_tlsa(cert_der, &[tlsa]));
    }

    #[test]
    fn verify_tlsa_ignores_unsupported_cert_usage() {
        use sha2::{Digest, Sha256};
        let cert_der = b"fake-cert";
        let hash = Sha256::digest(cert_der);

        // PKIX-TA (usage 0) is not handled by our DANE verifier.
        let tlsa = TLSA::new(
            CertUsage::PkixTa,
            Selector::Full,
            Matching::Sha256,
            hash.to_vec(),
        );

        assert!(!verify_certificate_against_tlsa(cert_der, &[tlsa]));
    }

    #[test]
    fn verify_tlsa_multiple_records_one_matches() {
        use sha2::{Digest, Sha256};
        let cert_der = b"real-cert-data";
        let hash = Sha256::digest(cert_der);

        let wrong = TLSA::new(
            CertUsage::DaneEe,
            Selector::Full,
            Matching::Sha256,
            vec![0u8; 32],
        );
        let correct = TLSA::new(
            CertUsage::DaneEe,
            Selector::Full,
            Matching::Sha256,
            hash.to_vec(),
        );

        assert!(verify_certificate_against_tlsa(cert_der, &[wrong, correct]));
    }

    #[test]
    fn verify_tlsa_empty_records() {
        let cert_der = b"any-cert";
        assert!(!verify_certificate_against_tlsa(cert_der, &[]));
    }

    #[test]
    fn verify_tlsa_dane_ta_usage_supported() {
        use sha2::{Digest, Sha256};
        let cert_der = b"dane-ta-cert";
        let hash = Sha256::digest(cert_der);

        let tlsa = TLSA::new(
            CertUsage::DaneTa,
            Selector::Full,
            Matching::Sha256,
            hash.to_vec(),
        );

        assert!(verify_certificate_against_tlsa(cert_der, &[tlsa]));
    }
}
