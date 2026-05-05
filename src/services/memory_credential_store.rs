use std::cell::RefCell;
use std::collections::HashMap;

use uuid::Uuid;

use crate::core::credential_store::{
    CredentialError, CredentialRole, CredentialStore, SecretValue,
};

/// In-memory credential store for unit tests. Does not require D-Bus or libsecret.
#[derive(Debug, Default)]
pub struct MemoryCredentialStore {
    secrets: RefCell<HashMap<(Uuid, CredentialRole), String>>,
}

impl MemoryCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStore for MemoryCredentialStore {
    fn read(&self, account_id: Uuid, role: CredentialRole) -> Result<SecretValue, CredentialError> {
        let secrets = self.secrets.borrow();
        secrets
            .get(&(account_id, role))
            .map(|v| SecretValue::new(v.clone()))
            .ok_or(CredentialError::NotFound { account_id, role })
    }

    fn write(
        &self,
        account_id: Uuid,
        role: CredentialRole,
        secret: &SecretValue,
    ) -> Result<(), CredentialError> {
        self.secrets
            .borrow_mut()
            .insert((account_id, role), secret.expose().to_string());
        Ok(())
    }

    fn delete(&self, account_id: Uuid, role: CredentialRole) -> Result<(), CredentialError> {
        self.secrets.borrow_mut().remove(&(account_id, role));
        Ok(())
    }

    fn delete_all_for_account(&self, account_id: Uuid) -> Result<(), CredentialError> {
        self.secrets
            .borrow_mut()
            .retain(|(id, _), _| *id != account_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read_roundtrip() {
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();
        let secret = SecretValue::new("password123".into());

        store
            .write(id, CredentialRole::ImapPassword, &secret)
            .unwrap();
        let read = store.read(id, CredentialRole::ImapPassword).unwrap();
        assert_eq!(read.expose(), "password123");
    }

    #[test]
    fn read_not_found() {
        let store = MemoryCredentialStore::new();
        let result = store.read(Uuid::new_v4(), CredentialRole::ImapPassword);
        assert!(matches!(result, Err(CredentialError::NotFound { .. })));
    }

    #[test]
    fn delete_removes_credential() {
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();
        let secret = SecretValue::new("pass".into());

        store
            .write(id, CredentialRole::ImapPassword, &secret)
            .unwrap();
        store.delete(id, CredentialRole::ImapPassword).unwrap();

        let result = store.read(id, CredentialRole::ImapPassword);
        assert!(matches!(result, Err(CredentialError::NotFound { .. })));
    }

    #[test]
    fn delete_all_for_account_removes_all_roles() {
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();
        let other_id = Uuid::new_v4();

        store
            .write(
                id,
                CredentialRole::ImapPassword,
                &SecretValue::new("imap".into()),
            )
            .unwrap();
        store
            .write(
                id,
                CredentialRole::SmtpPassword,
                &SecretValue::new("smtp".into()),
            )
            .unwrap();
        store
            .write(
                other_id,
                CredentialRole::ImapPassword,
                &SecretValue::new("other".into()),
            )
            .unwrap();

        store.delete_all_for_account(id).unwrap();

        assert!(matches!(
            store.read(id, CredentialRole::ImapPassword),
            Err(CredentialError::NotFound { .. })
        ));
        assert!(matches!(
            store.read(id, CredentialRole::SmtpPassword),
            Err(CredentialError::NotFound { .. })
        ));
        // Other account's credential is untouched.
        assert_eq!(
            store
                .read(other_id, CredentialRole::ImapPassword)
                .unwrap()
                .expose(),
            "other"
        );
    }

    #[test]
    fn write_overwrites_existing() {
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();

        store
            .write(
                id,
                CredentialRole::ImapPassword,
                &SecretValue::new("old".into()),
            )
            .unwrap();
        store
            .write(
                id,
                CredentialRole::ImapPassword,
                &SecretValue::new("new".into()),
            )
            .unwrap();

        let read = store.read(id, CredentialRole::ImapPassword).unwrap();
        assert_eq!(read.expose(), "new");
    }

    #[test]
    fn different_roles_are_independent() {
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();

        store
            .write(
                id,
                CredentialRole::ImapPassword,
                &SecretValue::new("imap-pass".into()),
            )
            .unwrap();
        store
            .write(
                id,
                CredentialRole::SmtpPassword,
                &SecretValue::new("smtp-pass".into()),
            )
            .unwrap();

        assert_eq!(
            store
                .read(id, CredentialRole::ImapPassword)
                .unwrap()
                .expose(),
            "imap-pass"
        );
        assert_eq!(
            store
                .read(id, CredentialRole::SmtpPassword)
                .unwrap()
                .expose(),
            "smtp-pass"
        );
    }
}
