use std::collections::HashMap;

use uuid::Uuid;

use crate::core::credential_store::{
    CredentialError, CredentialRole, CredentialStore, SecretValue,
};

const APPLICATION_ATTR: &str = "application";
const APPLICATION_VALUE: &str = "fairmail";
const ACCOUNT_ID_ATTR: &str = "fairmail-account-id";
const ROLE_ATTR: &str = "role";

/// Credential store backed by the freedesktop Secret Service API (libsecret/GNOME Keyring).
/// Stores secrets with attributes: `application=fairmail`, `fairmail-account-id=<uuid>`, `role=<role>`.
#[derive(Default)]
pub struct LibsecretCredentialStore {
    _private: (),
}

impl LibsecretCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn build_attributes(account_id: Uuid, role: CredentialRole) -> HashMap<&'static str, String> {
        let mut attrs = HashMap::new();
        attrs.insert(APPLICATION_ATTR, APPLICATION_VALUE.to_string());
        attrs.insert(ACCOUNT_ID_ATTR, account_id.to_string());
        attrs.insert(ROLE_ATTR, role.as_str().to_string());
        attrs
    }

    fn build_account_attributes(account_id: Uuid) -> HashMap<&'static str, String> {
        let mut attrs = HashMap::new();
        attrs.insert(APPLICATION_ATTR, APPLICATION_VALUE.to_string());
        attrs.insert(ACCOUNT_ID_ATTR, account_id.to_string());
        attrs
    }

    fn map_ss_error(e: secret_service::Error) -> CredentialError {
        match e {
            secret_service::Error::Locked => {
                CredentialError::KeychainUnavailable("system keychain is locked".into())
            }
            secret_service::Error::Zbus(ref zbus_err) => {
                CredentialError::KeychainUnavailable(format!("D-Bus error: {zbus_err}"))
            }
            other => CredentialError::Other(format!("{other}")),
        }
    }

    fn to_str_map<'a>(attrs: &'a HashMap<&'static str, String>) -> HashMap<&'a str, &'a str> {
        attrs.iter().map(|(k, v)| (*k, v.as_str())).collect()
    }

    /// Run a blocking async operation on a temporary tokio runtime.
    fn block_on<F: std::future::Future<Output = T>, T>(f: F) -> T {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime for keychain access");
        rt.block_on(f)
    }
}

impl CredentialStore for LibsecretCredentialStore {
    fn read(&self, account_id: Uuid, role: CredentialRole) -> Result<SecretValue, CredentialError> {
        Self::block_on(async {
            let ss = secret_service::SecretService::connect(secret_service::EncryptionType::Dh)
                .await
                .map_err(Self::map_ss_error)?;

            let collection = ss
                .get_default_collection()
                .await
                .map_err(Self::map_ss_error)?;

            if collection.is_locked().await.map_err(Self::map_ss_error)? {
                collection.unlock().await.map_err(Self::map_ss_error)?;
            }

            let attrs = Self::build_attributes(account_id, role);
            let attr_refs = Self::to_str_map(&attrs);

            let items = collection
                .search_items(attr_refs)
                .await
                .map_err(Self::map_ss_error)?;

            let item = items
                .first()
                .ok_or(CredentialError::NotFound { account_id, role })?;

            let secret_bytes = item.get_secret().await.map_err(Self::map_ss_error)?;
            let secret_str = String::from_utf8(secret_bytes)
                .map_err(|e| CredentialError::Other(format!("invalid UTF-8 in secret: {e}")))?;

            Ok(SecretValue::new(secret_str))
        })
    }

    fn write(
        &self,
        account_id: Uuid,
        role: CredentialRole,
        secret: &SecretValue,
    ) -> Result<(), CredentialError> {
        Self::block_on(async {
            let ss = secret_service::SecretService::connect(secret_service::EncryptionType::Dh)
                .await
                .map_err(Self::map_ss_error)?;

            let collection = ss
                .get_default_collection()
                .await
                .map_err(Self::map_ss_error)?;

            if collection.is_locked().await.map_err(Self::map_ss_error)? {
                collection.unlock().await.map_err(Self::map_ss_error)?;
            }

            let attrs = Self::build_attributes(account_id, role);
            let attr_refs = Self::to_str_map(&attrs);

            // Delete existing item if present (upsert semantics).
            let existing = collection
                .search_items(attr_refs.clone())
                .await
                .map_err(Self::map_ss_error)?;
            for item in existing {
                item.delete().await.map_err(Self::map_ss_error)?;
            }

            let label = format!("FairEmail {} {}", account_id, role.as_str());
            collection
                .create_item(
                    &label,
                    attr_refs,
                    secret.expose().as_bytes(),
                    true,
                    "text/plain",
                )
                .await
                .map_err(Self::map_ss_error)?;

            Ok(())
        })
    }

    fn delete(&self, account_id: Uuid, role: CredentialRole) -> Result<(), CredentialError> {
        Self::block_on(async {
            let ss = secret_service::SecretService::connect(secret_service::EncryptionType::Dh)
                .await
                .map_err(Self::map_ss_error)?;

            let collection = ss
                .get_default_collection()
                .await
                .map_err(Self::map_ss_error)?;

            if collection.is_locked().await.map_err(Self::map_ss_error)? {
                collection.unlock().await.map_err(Self::map_ss_error)?;
            }

            let attrs = Self::build_attributes(account_id, role);
            let attr_refs = Self::to_str_map(&attrs);

            let items = collection
                .search_items(attr_refs)
                .await
                .map_err(Self::map_ss_error)?;

            for item in items {
                item.delete().await.map_err(Self::map_ss_error)?;
            }

            Ok(())
        })
    }

    fn delete_all_for_account(&self, account_id: Uuid) -> Result<(), CredentialError> {
        Self::block_on(async {
            let ss = secret_service::SecretService::connect(secret_service::EncryptionType::Dh)
                .await
                .map_err(Self::map_ss_error)?;

            let collection = ss
                .get_default_collection()
                .await
                .map_err(Self::map_ss_error)?;

            if collection.is_locked().await.map_err(Self::map_ss_error)? {
                collection.unlock().await.map_err(Self::map_ss_error)?;
            }

            let attrs = Self::build_account_attributes(account_id);
            let attr_refs = Self::to_str_map(&attrs);

            let items = collection
                .search_items(attr_refs)
                .await
                .map_err(Self::map_ss_error)?;

            for item in items {
                item.delete().await.map_err(Self::map_ss_error)?;
            }

            Ok(())
        })
    }
}
