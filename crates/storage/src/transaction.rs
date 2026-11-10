//! Explicit transaction and savepoint boundary for URP use cases.

use sqlx::{Postgres, Sqlite, Transaction};

use crate::database::StorageError;

/// A named savepoint from the small, fixed set supported by the storage layer.
///
/// Arbitrary client-provided SQL identifiers are never interpolated into
/// transaction statements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Savepoint {
    /// A nested application operation that can be rolled back independently.
    NestedOperation,
    /// One item in a larger batch operation.
    BatchItem,
}

impl Savepoint {
    const fn create_sql(self) -> &'static str {
        match self {
            Self::NestedOperation => "SAVEPOINT urp_nested_operation",
            Self::BatchItem => "SAVEPOINT urp_batch_item",
        }
    }

    const fn rollback_sql(self) -> &'static str {
        match self {
            Self::NestedOperation => "ROLLBACK TO SAVEPOINT urp_nested_operation",
            Self::BatchItem => "ROLLBACK TO SAVEPOINT urp_batch_item",
        }
    }

    const fn release_sql(self) -> &'static str {
        match self {
            Self::NestedOperation => "RELEASE SAVEPOINT urp_nested_operation",
            Self::BatchItem => "RELEASE SAVEPOINT urp_batch_item",
        }
    }
}

/// A backend-specific SQLx transaction opened from a URP database pool.
///
/// The variants are public so application adapters can issue typed,
/// parameterized SQL through SQLx. They are never serialized or exposed through
/// HTTP contracts.
#[must_use = "a database transaction must be committed or rolled back explicitly"]
pub enum StorageTransaction {
    /// PostgreSQL transaction.
    Postgres(Transaction<'static, Postgres>),
    /// SQLite transaction.
    Sqlite(Transaction<'static, Sqlite>),
}

impl StorageTransaction {
    /// Commits the transaction.
    pub async fn commit(self) -> Result<(), StorageError> {
        match self {
            Self::Postgres(transaction) => transaction.commit().await?,
            Self::Sqlite(transaction) => transaction.commit().await?,
        }

        Ok(())
    }

    /// Rolls back the transaction.
    pub async fn rollback(self) -> Result<(), StorageError> {
        match self {
            Self::Postgres(transaction) => transaction.rollback().await?,
            Self::Sqlite(transaction) => transaction.rollback().await?,
        }

        Ok(())
    }

    /// Creates one of the supported named savepoints.
    pub async fn create_savepoint(&mut self, savepoint: Savepoint) -> Result<(), StorageError> {
        self.execute_static(savepoint.create_sql()).await
    }

    /// Rolls back to a previously created savepoint without releasing it.
    pub async fn rollback_to_savepoint(
        &mut self,
        savepoint: Savepoint,
    ) -> Result<(), StorageError> {
        self.execute_static(savepoint.rollback_sql()).await
    }

    /// Releases a previously created savepoint.
    pub async fn release_savepoint(&mut self, savepoint: Savepoint) -> Result<(), StorageError> {
        self.execute_static(savepoint.release_sql()).await
    }

    async fn execute_static(&mut self, statement: &'static str) -> Result<(), StorageError> {
        match self {
            Self::Postgres(transaction) => {
                sqlx::query(statement).execute(&mut **transaction).await?;
            }
            Self::Sqlite(transaction) => {
                sqlx::query(statement).execute(&mut **transaction).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use sqlx::{
        postgres::{PgPool, PgPoolOptions},
        sqlite::SqlitePool,
    };

    use super::{Savepoint, StorageTransaction};
    use crate::{Database, DatabaseConfig, StorageError};

    fn temporary_database_path() -> PathBuf {
        static NEXT_DATABASE_ID: AtomicU64 = AtomicU64::new(1);
        let sequence = NEXT_DATABASE_ID.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!(
            "urp_transaction_{}_{}.db",
            std::process::id(),
            sequence
        ))
    }

    async fn insert_value(
        transaction: &mut StorageTransaction,
        value: &str,
    ) -> Result<(), StorageError> {
        match transaction {
            StorageTransaction::Postgres(inner) => {
                sqlx::query("INSERT INTO urp_storage_transaction_test (value) VALUES ($1)")
                    .bind(value)
                    .execute(&mut **inner)
                    .await?;
            }
            StorageTransaction::Sqlite(inner) => {
                sqlx::query("INSERT INTO urp_storage_transaction_test (value) VALUES (?)")
                    .bind(value)
                    .execute(&mut **inner)
                    .await?;
            }
        }

        Ok(())
    }

    async fn row_count_postgres(pool: &PgPool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM urp_storage_transaction_test")
            .fetch_one(pool)
            .await
    }

    async fn row_count_sqlite(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM urp_storage_transaction_test")
            .fetch_one(pool)
            .await
    }

    async fn exercise_transaction_contract(
        database: &Database,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match database {
            Database::Postgres(pool) => {
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS urp_storage_transaction_test (value TEXT NOT NULL)",
                )
                .execute(pool)
                .await?;
                sqlx::query("TRUNCATE TABLE urp_storage_transaction_test")
                    .execute(pool)
                    .await?;
            }
            Database::Sqlite(pool) => {
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS urp_storage_transaction_test (value TEXT NOT NULL)",
                )
                .execute(pool)
                .await?;
                sqlx::query("DELETE FROM urp_storage_transaction_test")
                    .execute(pool)
                    .await?;
            }
        }

        let mut rolled_back = database.begin_transaction().await?;
        insert_value(&mut rolled_back, "rolled-back").await?;
        rolled_back.rollback().await?;

        let count_after_rollback = match database {
            Database::Postgres(pool) => row_count_postgres(pool).await?,
            Database::Sqlite(pool) => row_count_sqlite(pool).await?,
        };
        assert_eq!(count_after_rollback, 0);

        let mut committed = database.begin_transaction().await?;
        insert_value(&mut committed, "committed").await?;
        committed.create_savepoint(Savepoint::BatchItem).await?;
        insert_value(&mut committed, "savepoint-rolled-back").await?;
        committed
            .rollback_to_savepoint(Savepoint::BatchItem)
            .await?;
        committed.release_savepoint(Savepoint::BatchItem).await?;
        committed.commit().await?;

        let count_after_commit = match database {
            Database::Postgres(pool) => row_count_postgres(pool).await?,
            Database::Sqlite(pool) => row_count_sqlite(pool).await?,
        };
        assert_eq!(count_after_commit, 1);

        Ok(())
    }

    #[tokio::test]
    async fn sqlite_transaction_commit_rollback_and_savepoints_work()
    -> Result<(), Box<dyn std::error::Error>> {
        let path = temporary_database_path();
        let database = Database::connect(DatabaseConfig {
            url: format!("sqlite://{}", path.display()),
            max_connections: 2,
        })
        .await?;

        exercise_transaction_contract(&database).await?;

        if let Database::Sqlite(pool) = &database {
            sqlx::query("DROP TABLE urp_storage_transaction_test")
                .execute(pool)
                .await?;
            pool.close().await;
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(format!("{}-wal", path.display()));
        let _ = fs::remove_file(format!("{}-shm", path.display()));
        Ok(())
    }

    #[tokio::test]
    async fn postgres_transaction_commit_rollback_and_savepoints_work()
    -> Result<(), Box<dyn std::error::Error>> {
        let Ok(database_url) = env::var("URP_MIGRATION_TEST_DATABASE_URL") else {
            return Ok(());
        };

        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&database_url)
            .await?;
        let database = Database::Postgres(pool.clone());

        exercise_transaction_contract(&database).await?;

        sqlx::query("DROP TABLE urp_storage_transaction_test")
            .execute(&pool)
            .await?;
        pool.close().await;
        Ok(())
    }
}
