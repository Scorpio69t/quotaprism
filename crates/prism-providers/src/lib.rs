use chrono::{Duration, Utc};
use prism_core::{
    quota_percent, AccountSnapshot, Balance, Overview, Period, ProviderCapabilities,
    ProviderMetadata, ProviderSummary, PublicError, QuotaProvider, QuotaSnapshot, QuotaWindow,
    SourceKind, Unit,
};
use std::collections::HashMap;
use std::sync::Arc;

pub struct DemoProvider;

impl QuotaProvider for DemoProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: "demo".into(),
            name: "Offline Demo".into(),
            description: "Simulated local quota data".into(),
            demo: true,
        }
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities { quota_read: true }
    }

    fn fetch_overview(&self) -> Result<Overview, PublicError> {
        let now = Utc::now();
        let account = |id: &str, name: &str, alias: &str, windows, balances| AccountSnapshot {
            account_id: id.into(),
            provider_name: name.into(),
            alias: alias.into(),
            snapshot: QuotaSnapshot {
                observed_at: now,
                source: SourceKind::Demo,
                windows,
                balances,
            },
        };
        Ok(Overview {
            demo: true,
            accounts: vec![
                account(
                    "demo-1",
                    "Atlas Code",
                    "Workspace",
                    vec![
                        QuotaWindow {
                            window_id: "rolling".into(),
                            label: "5-hour window".into(),
                            unit: Unit::Request,
                            period: Period::Rolling { seconds: 18_000 },
                            used: Some("32".into()),
                            remaining: Some("68".into()),
                            limit: Some("100".into()),
                            used_percent: quota_percent(Some(32), Some(100), None)?,
                            remaining_percent: quota_percent(Some(68), Some(100), None)?,
                            reset_at: None,
                            next_recovery_at: Some(now + Duration::minutes(78)),
                        },
                        QuotaWindow {
                            window_id: "weekly".into(),
                            label: "Weekly window".into(),
                            unit: Unit::Percent,
                            period: Period::Fixed,
                            used: None,
                            remaining: None,
                            limit: None,
                            used_percent: None,
                            remaining_percent: quota_percent(None, None, Some(45.0))?,
                            reset_at: Some(now + Duration::days(3)),
                            next_recovery_at: None,
                        },
                    ],
                    vec![],
                ),
                account(
                    "demo-2",
                    "Nova Studio",
                    "Personal",
                    vec![QuotaWindow {
                        window_id: "credits".into(),
                        label: "Monthly credits".into(),
                        unit: Unit::Credit,
                        period: Period::Fixed,
                        used: Some("88".into()),
                        remaining: Some("12".into()),
                        limit: Some("100".into()),
                        used_percent: quota_percent(Some(88), Some(100), None)?,
                        remaining_percent: quota_percent(Some(12), Some(100), None)?,
                        reset_at: None,
                        next_recovery_at: None,
                    }],
                    vec![],
                ),
                account(
                    "demo-3",
                    "Orbit API",
                    "Development",
                    vec![QuotaWindow {
                        window_id: "tokens".into(),
                        label: "Token allowance".into(),
                        unit: Unit::Token,
                        period: Period::Unknown,
                        used: None,
                        remaining: None,
                        limit: None,
                        used_percent: None,
                        remaining_percent: None,
                        reset_at: None,
                        next_recovery_at: None,
                    }],
                    vec![Balance {
                        amount: "24.50".into(),
                        currency: "USD".into(),
                    }],
                ),
            ],
        })
    }
}

pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn QuotaProvider>>,
}

impl ProviderRegistry {
    pub fn demo() -> Self {
        let provider = Arc::new(DemoProvider);
        let mut providers = HashMap::new();
        providers.insert(
            provider.metadata().id.clone(),
            provider as Arc<dyn QuotaProvider>,
        );
        Self { providers }
    }

    pub fn summaries(&self) -> Vec<ProviderSummary> {
        let mut summaries = self
            .providers
            .values()
            .map(|provider| {
                let metadata = provider.metadata();
                let capabilities = provider.capabilities();
                ProviderSummary {
                    id: metadata.id,
                    name: metadata.name,
                    description: metadata.description,
                    quota_read: capabilities.quota_read,
                    demo: metadata.demo,
                }
            })
            .collect::<Vec<_>>();
        summaries.sort_by(|left, right| left.id.cmp(&right.id));
        summaries
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn QuotaProvider>> {
        self.providers.get(id).cloned()
    }

    pub fn fetch_demo_overview(&self) -> Result<Overview, PublicError> {
        self.get("demo")
            .ok_or_else(|| PublicError {
                code: "provider_not_found".into(),
                message: "Demo provider is unavailable".into(),
            })?
            .fetch_overview()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_only_exposes_registered_demo_provider() {
        let registry = ProviderRegistry::demo();
        let summaries = registry.summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, "demo");
        assert!(summaries[0].demo);
        assert!(summaries[0].quota_read);
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn demo_preserves_independent_windows_and_balance() {
        let overview = DemoProvider.fetch_overview().unwrap();
        assert!(overview.demo);
        assert_eq!(overview.accounts.len(), 3);
        let first = &overview.accounts[0].snapshot;
        assert_eq!(first.windows.len(), 2);
        assert_eq!(first.windows[0].used.as_deref(), Some("32"));
        assert_eq!(first.windows[0].used_percent, Some(32.0));
        assert!(first.windows[1].used_percent.is_none());
        assert!(matches!(
            first.windows[0].period,
            Period::Rolling { seconds: 18_000 }
        ));
        assert!(first.windows[0].reset_at.is_none());
        assert!(first.windows[0].next_recovery_at.is_some());
        let last = &overview.accounts[2].snapshot;
        assert!(last.windows[0].remaining_percent.is_none());
        assert_eq!(last.balances[0].amount, "24.50");
        assert_eq!(last.balances[0].currency, "USD");
    }
}
