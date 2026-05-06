//! FR-44: Password propagation from inbound account to associated SMTP identities.

use crate::core::credential_store::{
    identity_credential_uuid, CredentialRole, CredentialStore, SecretValue,
};

/// Propagate a new password to all identities associated with an account.
///
/// Writes the given password into the system keychain for each identity
/// using the `IdentitySmtpPassword` role.
///
/// Returns the number of identities whose password was successfully updated.
pub fn propagate_password_to_identities(
    cred_store: &dyn CredentialStore,
    identity_ids: &[i64],
    new_password: &str,
) -> usize {
    let mut updated = 0;
    for &identity_id in identity_ids {
        let uuid = identity_credential_uuid(identity_id);
        if let Err(e) = cred_store.write(
            uuid,
            CredentialRole::IdentitySmtpPassword,
            &SecretValue::new(new_password.to_string()),
        ) {
            eprintln!("Warning: could not propagate password to identity {identity_id}: {e}");
        } else {
            updated += 1;
        }
    }
    updated
}

/// Returns true if the password has actually changed between old and new values.
pub fn password_has_changed(old_password: &str, new_password: &str) -> bool {
    old_password != new_password
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::MemoryCredentialStore;

    #[test]
    fn propagate_updates_all_identities() {
        let store = MemoryCredentialStore::new();
        let ids = vec![1, 2, 3];
        let count = propagate_password_to_identities(&store, &ids, "new-pass");
        assert_eq!(count, 3);

        // Verify each identity got the new password.
        for &id in &ids {
            let uuid = identity_credential_uuid(id);
            let secret = store
                .read(uuid, CredentialRole::IdentitySmtpPassword)
                .unwrap();
            assert_eq!(secret.expose(), "new-pass");
        }
    }

    #[test]
    fn propagate_with_empty_ids_returns_zero() {
        let store = MemoryCredentialStore::new();
        let count = propagate_password_to_identities(&store, &[], "password");
        assert_eq!(count, 0);
    }

    #[test]
    fn password_not_changed_detection() {
        assert!(password_has_changed("old", "new"));
        assert!(!password_has_changed("same", "same"));
        assert!(!password_has_changed("", ""));
    }
}
