use chrono::Utc;
use prism_core::{
    AccountSummary, AddAccountInput, Overview, ProviderSummary, PublicError, RefreshStatus,
    UpdateAccountInput,
};
use prism_providers::ProviderRegistry;
use prism_security::{SecretError, SecretStore};
use prism_storage::{SqliteStore, StorageError, StoredAccount};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct NotificationKey {
    account_id: String,
    window_id: String,
    cycle_fingerprint: String,
    threshold: u8,
}

pub struct OverviewCache {
    snapshot: Mutex<Option<Overview>>,
    refresh_status: Mutex<RefreshStatus>,
    notifications: Mutex<HashMap<NotificationKey, ()>>,
    revision: AtomicU64,
}

impl Default for OverviewCache {
    fn default() -> Self {
        Self {
            snapshot: Mutex::new(None),
            refresh_status: Mutex::new(RefreshStatus {
                in_flight: false,
                last_attempt_at: None,
                last_success_at: None,
                retry_after: None,
                error: None,
                stale: false,
                revision: 0,
            }),
            notifications: Mutex::new(HashMap::new()),
            revision: AtomicU64::new(0),
        }
    }
}

impl OverviewCache {
    pub fn begin_refresh(&self) {
        let now = Utc::now();
        let revision = self.revision.fetch_add(1, Ordering::SeqCst) + 1;
        let mut status = self
            .refresh_status
            .lock()
            .expect("refresh status lock poisoned");
        status.in_flight = true;
        status.last_attempt_at = Some(now);
        status.revision = revision;
    }

    pub fn record_success(&self, overview: Overview) {
        let now = Utc::now();
        let revision = self.revision.fetch_add(1, Ordering::SeqCst) + 1;
        *self.snapshot.lock().expect("overview cache lock poisoned") = Some(overview);
        let mut status = self
            .refresh_status
            .lock()
            .expect("refresh status lock poisoned");
        status.in_flight = false;
        status.last_success_at = Some(now);
        status.retry_after = None;
        status.error = None;
        status.stale = false;
        status.revision = revision;
    }

    pub fn record_failure(&self, error: PublicError) {
        let now = Utc::now();
        let revision = self.revision.fetch_add(1, Ordering::SeqCst) + 1;
        let snapshot_exists = self
            .snapshot
            .lock()
            .expect("overview cache lock poisoned")
            .is_some();
        let mut status = self
            .refresh_status
            .lock()
            .expect("refresh status lock poisoned");
        status.in_flight = false;
        status.last_attempt_at = Some(now);
        status.error = Some(error);
        status.stale = snapshot_exists;
        status.revision = revision;
    }

    pub fn snapshot(&self) -> Option<Overview> {
        self.snapshot
            .lock()
            .expect("overview cache lock poisoned")
            .clone()
    }

    pub fn status(&self) -> RefreshStatus {
        self.refresh_status
            .lock()
            .expect("refresh status lock poisoned")
            .clone()
    }

    pub fn should_notify(
        &self,
        account_id: &str,
        window_id: &str,
        cycle_fingerprint: &str,
        threshold: u8,
    ) -> bool {
        let key = NotificationKey {
            account_id: account_id.to_string(),
            window_id: window_id.to_string(),
            cycle_fingerprint: cycle_fingerprint.to_string(),
            threshold,
        };
        self.notifications
            .lock()
            .expect("notification lock poisoned")
            .insert(key, ())
            .is_none()
    }
}

pub struct AccountService {
    store: Mutex<SqliteStore>,
    providers: ProviderRegistry,
    secrets: Arc<dyn SecretStore>,
    cache: OverviewCache,
}

impl AccountService {
    pub fn open(path: &Path, secrets: Arc<dyn SecretStore>) -> Result<Self, PublicError> {
        let store = SqliteStore::open(path).map_err(storage_public_error)?;
        Ok(Self {
            store: Mutex::new(store),
            providers: ProviderRegistry::demo(),
            secrets,
            cache: OverviewCache::default(),
        })
    }

    pub fn open_in_memory(secrets: Arc<dyn SecretStore>) -> Result<Self, PublicError> {
        let store = SqliteStore::open_in_memory().map_err(storage_public_error)?;
        Ok(Self {
            store: Mutex::new(store),
            providers: ProviderRegistry::demo(),
            secrets,
            cache: OverviewCache::default(),
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
        self.cache.begin_refresh();
        let overview = match self.providers.fetch_demo_overview() {
            Ok(overview) => overview,
            Err(error) => {
                self.cache.record_failure(error.clone());
                return Err(error);
            }
        };
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
        self.cache.record_success(overview.clone());
        Ok(overview)
    }

    pub fn get_overview(&self) -> Result<Overview, PublicError> {
        match self.refresh_demo() {
            Ok(overview) => Ok(overview),
            Err(error) => self.cache.snapshot().ok_or(error),
        }
    }

    pub fn refresh_status(&self) -> RefreshStatus {
        self.cache.status()
    }

    pub fn should_notify(
        &self,
        account_id: &str,
        window_id: &str,
        cycle_fingerprint: &str,
        threshold: u8,
    ) -> bool {
        self.cache
            .should_notify(account_id, window_id, cycle_fingerprint, threshold)
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
    use prism_core::QuotaProvider;
    use prism_security::{MockSecretStore, SecretRef, SecretValue};

    #[test]
    fn failed_refresh_retains_last_successful_overview() {
        let cache = OverviewCache::default();
        let overview = prism_providers::DemoProvider.fetch_overview().unwrap();
        cache.record_success(overview.clone());
        cache.record_failure(PublicError {
            code: "network_error".into(),
            message: "offline".into(),
        });
        assert!(cache.snapshot().unwrap().demo);
        let status = cache.status();
        assert!(status.stale);
        assert_eq!(status.error.unwrap().code, "network_error");
        assert!(!status.in_flight);
    }

    #[test]
    fn notification_dedup_isolated_by_account_window_cycle_and_threshold() {
        let cache = OverviewCache::default();
        assert!(cache.should_notify("a", "window", "cycle-1", 15));
        assert!(!cache.should_notify("a", "window", "cycle-1", 15));
        assert!(cache.should_notify("a", "window", "cycle-1", 10));
        assert!(cache.should_notify("a", "window", "cycle-2", 15));
        assert!(cache.should_notify("b", "window", "cycle-1", 15));
    }

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
        let status = service.refresh_status();
        assert!(!status.in_flight);
        assert!(!status.stale);
        assert!(status.last_success_at.is_some());
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
