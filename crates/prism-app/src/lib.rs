use chrono::Utc;
use prism_core::{
    AccountSummary, AddAccountInput, Overview, ProviderSummary, PublicError, UpdateAccountInput,
};
use prism_providers::ProviderRegistry;
use prism_security::{SecretError, SecretStore};
use prism_storage::{SqliteStore, StorageError, StoredAccount};
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct AccountService {
    store: Mutex<SqliteStore>,
    providers: ProviderRegistry,
    secrets: Arc<dyn SecretStore>,
}

impl AccountService {
    pub fn open(path: &Path, secrets: Arc<dyn SecretStore>) -> Result<Self, PublicError> {
        let store = SqliteStore::open(path).map_err(storage_public_error)?;
        Ok(Self {
            store: Mutex::new(store),
            providers: ProviderRegistry::demo(),
            secrets,
        })
    }

    pub fn open_in_memory(secrets: Arc<dyn SecretStore>) -> Result<Self, PublicError> {
        let store = SqliteStore::open_in_memory().map_err(storage_public_error)?;
        Ok(Self {
            store: Mutex::new(store),
            providers: ProviderRegistry::demo(),
            secrets,
        })
    }

    pub fn list_providers(&self) -> Vec<ProviderSummary> {
        self.providers.summaries()
    }

    pub fn list_accounts(&self) -> Result<Vec<AccountSummary>, PublicError> {
        let store = self.store.lock().map_err(lock_error)?;
        store
            .list_stored_accounts()
            .map(|accounts| accounts.into_iter().map(to_account_summary).collect())
            .map_err(storage_public_error)
    }

    pub fn add_account(&self, input: AddAccountInput) -> Result<AccountSummary, PublicError> {
        if input.alias.trim().is_empty() {
            return Err(PublicError {
                code: "invalid_account_alias".into(),
                message: "Account alias cannot be empty".into(),
            });
        }
        let provider = self
            .providers
            .get(&input.provider_id)
            .ok_or_else(|| PublicError {
                code: "provider_not_found".into(),
                message: "Provider is not registered".into(),
            })?;
        let metadata = provider.metadata();
        let account = StoredAccount {
            account_id: format!("account-{}", Uuid::new_v4()),
            provider_id: metadata.id,
            provider_name: metadata.name,
            alias: input.alias.trim().to_string(),
            credential_ref: None,
            enabled: true,
            created_at: Utc::now(),
        };
        let store = self.store.lock().map_err(lock_error)?;
        store.save_account(&account).map_err(storage_public_error)?;
        Ok(to_account_summary(account))
    }

    pub fn update_account(
        &self,
        account_id: &str,
        input: UpdateAccountInput,
    ) -> Result<AccountSummary, PublicError> {
        if input
            .alias
            .as_deref()
            .is_some_and(|alias| alias.trim().is_empty())
        {
            return Err(PublicError {
                code: "invalid_account_alias".into(),
                message: "Account alias cannot be empty".into(),
            });
        }
        let alias = input.alias.as_deref().map(str::trim);
        let store = self.store.lock().map_err(lock_error)?;
        let updated = store
            .update_account(account_id, alias, input.enabled)
            .map_err(storage_public_error)?;
        if !updated {
            return Err(account_not_found(account_id));
        }
        store
            .get_stored_account(account_id)
            .map_err(storage_public_error)?
            .map(to_account_summary)
            .ok_or_else(|| account_not_found(account_id))
    }

    pub fn remove_account(&self, account_id: &str) -> Result<bool, PublicError> {
        let store = self.store.lock().map_err(lock_error)?;
        let account = store
            .get_stored_account(account_id)
            .map_err(storage_public_error)?
            .ok_or_else(|| account_not_found(account_id))?;
        if let Some(reference) = account.credential_ref.as_deref() {
            let reference =
                prism_security::SecretRef::new(reference).map_err(|error| PublicError {
                    code: "invalid_secret_reference".into(),
                    message: error.to_string(),
                })?;
            match self.secrets.delete(&reference) {
                Ok(()) => {}
                Err(SecretError::NotFound) => {}
                Err(error) => {
                    return Err(PublicError {
                        code: "secret_store_error".into(),
                        message: error.to_string(),
                    })
                }
            }
        }
        store
            .delete_account(account_id)
            .map_err(storage_public_error)
    }

    pub fn refresh_demo(&self) -> Result<Overview, PublicError> {
        let overview = self.providers.fetch_demo_overview()?;
        let store = self.store.lock().map_err(lock_error)?;
        for account in &overview.accounts {
            if store
                .get_stored_account(&account.account_id)
                .map_err(storage_public_error)?
                .is_none()
            {
                store
                    .save_account(&StoredAccount {
                        account_id: account.account_id.clone(),
                        provider_id: "demo".into(),
                        provider_name: account.provider_name.clone(),
                        alias: account.alias.clone(),
                        credential_ref: None,
                        enabled: true,
                        created_at: account.snapshot.observed_at,
                    })
                    .map_err(storage_public_error)?;
            }
            store
                .save_account_snapshot(account)
                .map_err(storage_public_error)?;
        }
        Ok(overview)
    }

    pub fn get_overview(&self) -> Result<Overview, PublicError> {
        self.refresh_demo()
    }
}

fn to_account_summary(account: StoredAccount) -> AccountSummary {
    AccountSummary {
        account_id: account.account_id,
        provider_id: account.provider_id,
        provider_name: account.provider_name,
        alias: account.alias,
        credential_ref: account.credential_ref,
        enabled: account.enabled,
        created_at: account.created_at,
    }
}

fn account_not_found(account_id: &str) -> PublicError {
    PublicError {
        code: "account_not_found".into(),
        message: format!("Account '{account_id}' was not found"),
    }
}

fn lock_error(_: impl std::fmt::Display) -> PublicError {
    PublicError {
        code: "service_lock_error".into(),
        message: "Account service became unavailable".into(),
    }
}

fn storage_public_error(error: StorageError) -> PublicError {
    PublicError {
        code: "storage_error".into(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_security::{MockSecretStore, SecretRef, SecretValue};

    #[test]
    fn demo_registry_and_refresh_seed_persistent_demo_accounts() {
        let secrets = Arc::new(MockSecretStore::new());
        let service = AccountService::open_in_memory(secrets).unwrap();
        let providers = service.list_providers();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "demo");

        let overview = service.refresh_demo().unwrap();
        assert_eq!(overview.accounts.len(), 3);
        assert_eq!(service.list_accounts().unwrap().len(), 3);
    }

    #[test]
    fn account_update_is_isolated_and_remove_is_consistent() {
        let secrets = Arc::new(MockSecretStore::new());
        let service = AccountService::open_in_memory(secrets.clone()).unwrap();
        let first = service
            .add_account(AddAccountInput {
                provider_id: "demo".into(),
                alias: "First".into(),
            })
            .unwrap();
        let second = service
            .add_account(AddAccountInput {
                provider_id: "demo".into(),
                alias: "Second".into(),
            })
            .unwrap();

        let updated = service
            .update_account(
                &first.account_id,
                UpdateAccountInput {
                    alias: Some("Renamed".into()),
                    enabled: Some(false),
                },
            )
            .unwrap();
        assert_eq!(updated.alias, "Renamed");
        assert!(!updated.enabled);

        let unchanged = service
            .list_accounts()
            .unwrap()
            .into_iter()
            .find(|account| account.account_id == second.account_id)
            .unwrap();
        assert_eq!(unchanged.alias, "Second");
        assert!(unchanged.enabled);

        assert!(service.remove_account(&first.account_id).unwrap());
        assert!(matches!(
            service.remove_account(&first.account_id),
            Err(PublicError { ref code, .. }) if code == "account_not_found"
        ));
    }

    #[test]
    fn demo_refresh_preserves_user_account_alias() {
        let secrets = Arc::new(MockSecretStore::new());
        let service = AccountService::open_in_memory(secrets).unwrap();
        service.refresh_demo().unwrap();
        let account = service.list_accounts().unwrap().into_iter().next().unwrap();
        let updated = service
            .update_account(
                &account.account_id,
                UpdateAccountInput {
                    alias: Some("My renamed account".into()),
                    enabled: None,
                },
            )
            .unwrap();
        assert_eq!(updated.alias, "My renamed account");

        service.refresh_demo().unwrap();
        let after_refresh = service
            .list_accounts()
            .unwrap()
            .into_iter()
            .find(|item| item.account_id == account.account_id)
            .unwrap();
        assert_eq!(after_refresh.alias, "My renamed account");
    }

    #[test]
    fn remove_account_removes_matching_secret_reference() {
        let secrets = Arc::new(MockSecretStore::new());
        let service = AccountService::open_in_memory(secrets.clone()).unwrap();
        let reference = SecretRef::new("account:demo").unwrap();
        secrets
            .set(&reference, &SecretValue::new("private").unwrap())
            .unwrap();

        let account = StoredAccount {
            account_id: "secure-account".into(),
            provider_id: "demo".into(),
            provider_name: "Offline Demo".into(),
            alias: "Secure".into(),
            credential_ref: Some(reference.as_str().into()),
            enabled: true,
            created_at: Utc::now(),
        };
        {
            let store = service.store.lock().unwrap();
            store.save_account(&account).unwrap();
        }

        assert!(service.remove_account("secure-account").unwrap());
        assert!(matches!(
            secrets.get(&reference),
            Err(prism_security::SecretError::NotFound)
        ));
    }
}
