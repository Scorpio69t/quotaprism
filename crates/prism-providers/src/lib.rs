use chrono::{Duration, Utc};
use prism_core::{
    quota_percent, AccountSnapshot, Balance, Overview, Period, PublicError, QuotaProvider,
    QuotaSnapshot, QuotaWindow, SourceKind, Unit,
};

pub struct DemoProvider;

impl QuotaProvider for DemoProvider {
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

#[cfg(test)]
mod tests {
    use super::*;

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
