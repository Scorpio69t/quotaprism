use chrono::{DateTime, Utc};
use prism_core::AccountSnapshot;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

const SCHEMA_VERSION: i64 = 2;

#[derive(Clone, Debug)]
pub struct StoredAccount {
    pub account_id: String,
    pub provider_id: String,
    pub provider_name: String,
    pub alias: String,
    pub credential_ref: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("snapshot serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid stored timestamp: {0}")]
    Timestamp(#[from] chrono::ParseError),
    #[error("unsupported SQLite schema version {0}")]
    UnsupportedSchema(i64),
}

pub struct SqliteStore {
    connection: Connection,
}

impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let mut connection = Connection::open(path)?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        apply_migrations(&mut connection)?;
        Ok(Self { connection })
    }

    pub fn open_in_memory() -> Result<Self, StorageError> {
        let mut connection = Connection::open_in_memory()?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        apply_migrations(&mut connection)?;
        Ok(Self { connection })
    }

    pub fn save_account_snapshot(&self, account: &AccountSnapshot) -> Result<(), StorageError> {
        let normalized = serde_json::to_string(&account.snapshot)?;
        let source = match &account.snapshot.source {
            prism_core::SourceKind::Demo => "demo",
        };
        let transaction = self.connection.unchecked_transaction()?;

        transaction.execute(
            r#"
            INSERT INTO accounts (
                account_id,
                provider_id,
                provider_name,
                alias,
                credential_ref,
                enabled,
                created_at
            )
            VALUES (?1, ?2, ?3, ?4, NULL, 1, ?5)
            ON CONFLICT(account_id) DO UPDATE SET
                provider_name = excluded.provider_name
            "#,
            params![
                account.account_id,
                "demo",
                account.provider_name,
                account.alias,
                Utc::now().to_rfc3339()
            ],
        )?;
        transaction.execute(
            r#"
            INSERT INTO quota_snapshots (
                account_id,
                observed_at,
                source,
                schema_version,
                normalized_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(account_id) DO UPDATE SET
                observed_at = excluded.observed_at,
                source = excluded.source,
                schema_version = excluded.schema_version,
                normalized_json = excluded.normalized_json
            "#,
            params![
                account.account_id,
                account.snapshot.observed_at.to_rfc3339(),
                source,
                SCHEMA_VERSION,
                normalized,
            ],
        )?;

        transaction.commit()?;
        Ok(())
    }

    pub fn get_account(&self, account_id: &str) -> Result<Option<AccountSnapshot>, StorageError> {
        let record = self
            .connection
            .query_row(
                r#"
                SELECT
                    a.account_id,
                    a.provider_name,
                    a.alias,
                    q.normalized_json
                FROM accounts AS a
                LEFT JOIN quota_snapshots AS q ON q.account_id = a.account_id
                WHERE a.account_id = ?1
                "#,
                params![account_id],
                |row| {
                    let account_id: String = row.get(0)?;
                    let provider_name: String = row.get(1)?;
                    let alias: String = row.get(2)?;
                    let normalized: Option<String> = row.get(3)?;
                    Ok((account_id, provider_name, alias, normalized))
                },
            )
            .optional()?;

        match record {
            Some((account_id, provider_name, alias, Some(normalized))) => {
                let snapshot = serde_json::from_str(&normalized)?;
                Ok(Some(AccountSnapshot {
                    account_id,
                    provider_name,
                    alias,
                    snapshot,
                }))
            }
            Some((_, _, _, None)) => Ok(None),
            None => Ok(None),
        }
    }

    pub fn list_accounts(&self) -> Result<Vec<AccountSnapshot>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT account_id
            FROM accounts
            ORDER BY account_id
            "#,
        )?;
        let account_ids = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        let mut accounts = Vec::with_capacity(account_ids.len());
        for account_id in account_ids {
            if let Some(account) = self.get_account(&account_id)? {
                accounts.push(account);
            }
        }
        Ok(accounts)
    }

    pub fn delete_account(&self, account_id: &str) -> Result<bool, StorageError> {
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "DELETE FROM quota_snapshots WHERE account_id = ?1",
            params![account_id],
        )?;
        let deleted = transaction.execute(
            "DELETE FROM accounts WHERE account_id = ?1",
            params![account_id],
        )?;
        transaction.commit()?;
        Ok(deleted > 0)
    }

    pub fn save_account(&self, account: &StoredAccount) -> Result<(), StorageError> {
        let credential_ref = account.credential_ref.as_deref();
        self.connection.execute(
            r#"
            INSERT INTO accounts (
                account_id,
                provider_id,
                provider_name,
                alias,
                credential_ref,
                enabled,
                created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(account_id) DO UPDATE SET
                provider_id = excluded.provider_id,
                provider_name = excluded.provider_name,
                alias = excluded.alias,
                credential_ref = excluded.credential_ref,
                enabled = excluded.enabled,
                created_at = excluded.created_at
            "#,
            params![
                account.account_id,
                account.provider_id,
                account.provider_name,
                account.alias,
                credential_ref,
                account.enabled as i64,
                account.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn get_stored_account(
        &self,
        account_id: &str,
    ) -> Result<Option<StoredAccount>, StorageError> {
        let record = self
            .connection
            .query_row(
                r#"
                SELECT
                    account_id,
                    provider_id,
                    provider_name,
                    alias,
                    credential_ref,
                    enabled,
                    created_at
                FROM accounts
                WHERE account_id = ?1
                "#,
                params![account_id],
                |row| {
                    let created_at: String = row.get(6)?;
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, i64>(5)?,
                        created_at,
                    ))
                },
            )
            .optional()?;

        match record {
            Some((
                account_id,
                provider_id,
                provider_name,
                alias,
                credential_ref,
                enabled,
                created_at,
            )) => Ok(Some(StoredAccount {
                account_id,
                provider_id,
                provider_name,
                alias,
                credential_ref,
                enabled: enabled != 0,
                created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
            })),
            None => Ok(None),
        }
    }

    pub fn list_stored_accounts(&self) -> Result<Vec<StoredAccount>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT account_id
            FROM accounts
            ORDER BY account_id
            "#,
        )?;
        let account_ids = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        let mut accounts = Vec::with_capacity(account_ids.len());
        for account_id in account_ids {
            if let Some(account) = self.get_stored_account(&account_id)? {
                accounts.push(account);
            }
        }
        Ok(accounts)
    }

    pub fn update_account(
        &self,
        account_id: &str,
        alias: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<bool, StorageError> {
        let mut updated = false;
        let transaction = self.connection.unchecked_transaction()?;
        if let Some(alias) = alias {
            updated |= transaction.execute(
                "UPDATE accounts SET alias = ?1 WHERE account_id = ?2",
                params![alias, account_id],
            )? > 0;
        }
        if let Some(enabled) = enabled {
            updated |= transaction.execute(
                "UPDATE accounts SET enabled = ?1 WHERE account_id = ?2",
                params![enabled as i64, account_id],
            )? > 0;
        }
        transaction.commit()?;
        Ok(updated)
    }

    pub fn schema_version(&self) -> Result<i64, StorageError> {
        Ok(self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))?)
    }
}

fn apply_migrations(connection: &mut Connection) -> Result<(), StorageError> {
    let version = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version > SCHEMA_VERSION {
        return Err(StorageError::UnsupportedSchema(version));
    }
    if version == SCHEMA_VERSION {
        return Ok(());
    }

    let transaction = connection.transaction()?;
    if version == 0 {
        transaction.execute_batch(
            r#"
            CREATE TABLE accounts (
                account_id TEXT PRIMARY KEY,
                provider_id TEXT NOT NULL,
                provider_name TEXT NOT NULL,
                alias TEXT NOT NULL,
                credential_ref TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            );

            CREATE TABLE quota_snapshots (
                account_id TEXT PRIMARY KEY,
                observed_at TEXT NOT NULL,
                source TEXT NOT NULL,
                schema_version INTEGER NOT NULL,
                normalized_json TEXT NOT NULL,
                FOREIGN KEY(account_id) REFERENCES accounts(account_id) ON DELETE CASCADE
            );

            CREATE INDEX quota_snapshots_observed_at_idx
                ON quota_snapshots(observed_at DESC);
            "#,
        )?;
    } else if version == 1 {
        transaction.execute_batch(
            r#"
            ALTER TABLE accounts ADD COLUMN provider_id TEXT NOT NULL DEFAULT 'demo';
            ALTER TABLE accounts ADD COLUMN credential_ref TEXT;
            ALTER TABLE accounts ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1;
            ALTER TABLE accounts ADD COLUMN created_at TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z';
            CREATE INDEX IF NOT EXISTS quota_snapshots_observed_at_idx
                ON quota_snapshots(observed_at DESC);
            "#,
        )?;
    } else {
        return Err(StorageError::UnsupportedSchema(version));
    }
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use prism_core::{
        AccountSnapshot, Balance, Period, QuotaSnapshot, QuotaWindow, SourceKind, Unit,
    };

    fn demo_account() -> AccountSnapshot {
        let observed_at = Utc::now();
        AccountSnapshot {
            account_id: "demo-1".into(),
            provider_name: "Demo".into(),
            alias: "Personal".into(),
            snapshot: QuotaSnapshot {
                observed_at,
                source: SourceKind::Demo,
                windows: vec![QuotaWindow {
                    window_id: "five-hour".into(),
                    label: "5-hour window".into(),
                    unit: Unit::Request,
                    period: Period::Rolling { seconds: 18_000 },
                    used: Some("32".into()),
                    remaining: Some("68".into()),
                    limit: Some("100".into()),
                    used_percent: Some(32.0),
                    remaining_percent: Some(68.0),
                    reset_at: None,
                    next_recovery_at: Some(observed_at + Duration::minutes(78)),
                }],
                balances: vec![Balance {
                    amount: "24.50".into(),
                    currency: "USD".into(),
                }],
            },
        }
    }

    #[test]
    fn migrates_and_reopens_from_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("quotaprism.sqlite");
        {
            let store = SqliteStore::open(&path).unwrap();
            assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
            store.save_account_snapshot(&demo_account()).unwrap();
        }

        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
        let account = store.get_account("demo-1").unwrap().unwrap();
        assert_eq!(account.account_id, "demo-1");
        assert_eq!(account.snapshot.windows[0].used.as_deref(), Some("32"));
        assert_eq!(account.snapshot.balances[0].amount, "24.50");
        assert_eq!(store.list_accounts().unwrap().len(), 1);
    }

    #[test]
    fn delete_account_removes_snapshot_transactionally() {
        let store = SqliteStore::open_in_memory().unwrap();
        store.save_account_snapshot(&demo_account()).unwrap();
        assert!(store.delete_account("demo-1").unwrap());
        assert!(store.get_account("demo-1").unwrap().is_none());
        assert!(store.list_accounts().unwrap().is_empty());
    }

    #[test]
    fn migration_failure_rolls_back_schema() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("broken.sqlite");
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE accounts (unexpected_column INTEGER); PRAGMA user_version = 0;",
                )
                .unwrap();
        }

        assert!(SqliteStore::open(&path).is_err());
        let connection = Connection::open(&path).unwrap();
        let snapshot_table_exists: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'quota_snapshots'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(snapshot_table_exists, 0);
    }

    #[test]
    fn migrates_v1_account_schema_transactionally() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("v1.sqlite");
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    r#"
                    CREATE TABLE accounts (
                        account_id TEXT PRIMARY KEY,
                        provider_name TEXT NOT NULL,
                        alias TEXT NOT NULL
                    );
                    CREATE TABLE quota_snapshots (
                        account_id TEXT PRIMARY KEY,
                        observed_at TEXT NOT NULL,
                        source TEXT NOT NULL,
                        schema_version INTEGER NOT NULL,
                        normalized_json TEXT NOT NULL,
                        FOREIGN KEY(account_id) REFERENCES accounts(account_id) ON DELETE CASCADE
                    );
                    INSERT INTO accounts (account_id, provider_name, alias)
                        VALUES ('demo-1', 'Demo', 'Personal');
                    PRAGMA user_version = 1;
                    "#,
                )
                .unwrap();
        }

        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
        let account = store.get_stored_account("demo-1").unwrap().unwrap();
        assert_eq!(account.provider_id, "demo");
        assert_eq!(account.alias, "Personal");
        assert!(account.enabled);
    }
}
