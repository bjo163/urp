use std::collections::BTreeMap;

use sqlx::{Row, SqlitePool};

use super::{
    Migration, MigrationError, MigrationStatus, migration_checksum, validate_registry,
    verify_applied_migration,
};

const LEDGER_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS urp_schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    checksum TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
)
"#;

const LOCK_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS urp_schema_migration_lock (
    lock_id INTEGER PRIMARY KEY
)
"#;

const REGISTERED_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "0001_platform_metadata",
    sql: include_str!("../../../../migrations/sqlite/0001_platform_metadata.sql"),
}, Migration {
    version: 2,
    name: "0002_transactional_outbox",
    sql: include_str!("../../../../migrations/sqlite/0002_transactional_outbox.sql"),
}];

/// Applies pending SQLite migrations, or returns a read-only dry-run plan.
pub(crate) async fn sqlite_migrate(
    pool: &SqlitePool,
    dry_run: bool,
) -> Result<Vec<MigrationStatus>, MigrationError> {
    run_with_migrations(pool, REGISTERED_MIGRATIONS, dry_run).await
}

/// Returns the current SQLite migration status without writing schema or ledger data.
pub(crate) async fn sqlite_migration_status(
    pool: &SqlitePool,
) -> Result<Vec<MigrationStatus>, MigrationError> {
    run_with_migrations(pool, REGISTERED_MIGRATIONS, true).await
}

async fn run_with_migrations(
    pool: &SqlitePool,
    migrations: &[Migration],
    dry_run: bool,
) -> Result<Vec<MigrationStatus>, MigrationError> {
    validate_registry(migrations)?;
    check_sqlite_capabilities(pool).await?;

    let ledger_exists: i64 = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'urp_schema_migrations')",
    )
    .fetch_one(pool)
    .await?;

    if ledger_exists == 0 && dry_run {
        return Ok(migrations
            .iter()
            .map(|migration| MigrationStatus {
                version: migration.version,
                name: migration.name.to_owned(),
                checksum: migration_checksum(migration.name, migration.sql),
                applied: false,
            })
            .collect());
    }

    if !dry_run {
        sqlx::query(LEDGER_SQL).execute(pool).await?;
        sqlx::query(LOCK_SQL).execute(pool).await?;
    }

    let applied_rows = if ledger_exists == 0 {
        Vec::new()
    } else {
        sqlx::query("SELECT version, name, checksum FROM urp_schema_migrations ORDER BY version")
            .fetch_all(pool)
            .await?
    };

    let mut applied = BTreeMap::new();
    for row in applied_rows {
        let version: i64 = row.try_get("version")?;
        let name: String = row.try_get("name")?;
        let checksum: String = row.try_get("checksum")?;
        applied.insert(version, (name, checksum));
    }

    for version in applied.keys() {
        if !migrations
            .iter()
            .any(|migration| migration.version == *version)
        {
            return Err(MigrationError::UnknownAppliedMigration { version: *version });
        }
    }

    let mut result = Vec::with_capacity(migrations.len());

    for migration in migrations {
        let checksum = migration_checksum(migration.name, migration.sql);

        if let Some((applied_name, applied_checksum)) = applied.get(&migration.version) {
            verify_applied_migration(migration, applied_name, applied_checksum, &checksum)?;
            result.push(MigrationStatus {
                version: migration.version,
                name: migration.name.to_owned(),
                checksum,
                applied: true,
            });
            continue;
        }

        if dry_run {
            result.push(MigrationStatus {
                version: migration.version,
                name: migration.name.to_owned(),
                checksum,
                applied: false,
            });
            continue;
        }

        let mut transaction = pool.begin().await?;

        // This no-op insert is a write and obtains SQLite's database write lock.
        // The busy timeout configured by the connection adapter bounds contention.
        sqlx::query("INSERT OR IGNORE INTO urp_schema_migration_lock (lock_id) VALUES (1)")
            .execute(&mut *transaction)
            .await?;

        let existing =
            sqlx::query("SELECT name, checksum FROM urp_schema_migrations WHERE version = ?")
                .bind(migration.version)
                .fetch_optional(&mut *transaction)
                .await?;

        if let Some(row) = existing {
            let applied_name: String = row.try_get("name")?;
            let applied_checksum: String = row.try_get("checksum")?;
            verify_applied_migration(migration, &applied_name, &applied_checksum, &checksum)?;
        } else {
            sqlx::query(migration.sql)
                .execute(&mut *transaction)
                .await?;

            sqlx::query(
                "INSERT INTO urp_schema_migrations (version, name, checksum) VALUES (?, ?, ?)",
            )
            .bind(migration.version)
            .bind(migration.name)
            .bind(&checksum)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        result.push(MigrationStatus {
            version: migration.version,
            name: migration.name.to_owned(),
            checksum,
            applied: true,
        });
    }

    Ok(result)
}

async fn check_sqlite_capabilities(pool: &SqlitePool) -> Result<(), MigrationError> {
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(pool)
        .await?;
    if foreign_keys != 1 {
        return Err(MigrationError::UnsupportedCapability {
            backend: "sqlite",
            capability: "foreign_keys",
        });
    }

    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(pool)
        .await?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(MigrationError::UnsupportedCapability {
            backend: "sqlite",
            capability: "write_ahead_logging",
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    use super::{Migration, REGISTERED_MIGRATIONS, check_sqlite_capabilities, run_with_migrations};
    use crate::{Database, DatabaseConfig, MigrationError};

    fn temporary_database_path() -> PathBuf {
        static NEXT_DATABASE_ID: AtomicU64 = AtomicU64::new(1);
        let sequence = NEXT_DATABASE_ID.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!(
            "urp_sqlite_migration_{}_{}.db",
            std::process::id(),
            sequence
        ))
    }

    #[tokio::test]
    async fn sqlite_migrations_support_dry_run_idempotency_and_checksum_detection()
    -> Result<(), Box<dyn std::error::Error>> {
        let database_path = temporary_database_path();
        let database_url = format!("sqlite://{}", database_path.display());
        let database = Database::connect(DatabaseConfig {
            url: database_url,
            max_connections: 4,
        })
        .await?;

        let dry_run = database.migrate_sqlite(true).await?;
        assert!(!dry_run.is_empty());
        assert!(dry_run.iter().all(|migration| !migration.applied));

        let first_run = database.migrate_sqlite(false).await?;
        assert!(!first_run.is_empty());
        assert!(first_run.iter().all(|migration| migration.applied));

        let second_run = database.migrate_sqlite(false).await?;
        assert_eq!(second_run, first_run);

        let pool = match &database {
            Database::Sqlite(pool) => pool.clone(),
            Database::Postgres(_) => return Err("expected SQLite database".into()),
        };

        let altered_migrations = [Migration {
            version: 1,
            name: "0001_platform_metadata",
            sql: "SELECT 1",
        }];
        let mismatch = run_with_migrations(&pool, &altered_migrations, false).await;
        assert!(matches!(
            mismatch,
            Err(MigrationError::ChecksumMismatch { version: 1, .. })
        ));

        match database {
            Database::Sqlite(pool) => pool.close().await,
            Database::Postgres(_) => return Err("expected SQLite database".into()),
        }
        let _ = fs::remove_file(&database_path);
        let _ = fs::remove_file(format!("{}-wal", database_path.display()));
        let _ = fs::remove_file(format!("{}-shm", database_path.display()));

        Ok(())
    }

    #[tokio::test]
    async fn sqlite_migration_rejects_disabled_foreign_keys()
    -> Result<(), Box<dyn std::error::Error>> {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .foreign_keys(false);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        let result = check_sqlite_capabilities(&pool).await;
        assert!(matches!(
            result,
            Err(MigrationError::UnsupportedCapability {
                backend: "sqlite",
                capability: "foreign_keys"
            })
        ));

        pool.close().await;
        assert!(super::validate_registry(REGISTERED_MIGRATIONS).is_ok());
        Ok(())
    }
}
