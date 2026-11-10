use std::collections::BTreeMap;

use sqlx::{Row, postgres::PgPool};

use super::{
    Migration, MigrationError, MigrationStatus, migration_checksum, validate_registry,
    verify_applied_migration,
};

const LEDGER_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS urp_schema_migrations (
    version BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    checksum TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
)
"#;

const MIGRATION_LOCK_KEY: i64 = 0x5552_505f_4d49_4752;

const REGISTERED_MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "0001_platform_metadata",
        sql: include_str!("../../../../migrations/postgres/0001_platform_metadata.sql"),
    },
    Migration {
        version: 2,
        name: "0002_transactional_outbox",
        sql: include_str!("../../../../migrations/postgres/0002_transactional_outbox.sql"),
    },
    Migration {
        version: 3,
        name: "0003_user_identity",
        sql: include_str!("../../../../migrations/postgres/0003_user_identity.sql"),
    },
    Migration {
        version: 4,
        name: "0004_auth_identity_status",
        sql: include_str!("../../../../migrations/postgres/0004_auth_identity_status.sql"),
    },
];

/// Applies pending PostgreSQL migrations, or returns a read-only dry-run plan.
pub(crate) async fn postgres_migrate(
    pool: &PgPool,
    dry_run: bool,
) -> Result<Vec<MigrationStatus>, MigrationError> {
    run_with_migrations(pool, REGISTERED_MIGRATIONS, dry_run).await
}

/// Returns the current PostgreSQL migration status without writing schema or ledger data.
pub(crate) async fn postgres_migration_status(
    pool: &PgPool,
) -> Result<Vec<MigrationStatus>, MigrationError> {
    run_with_migrations(pool, REGISTERED_MIGRATIONS, true).await
}

async fn run_with_migrations(
    pool: &PgPool,
    migrations: &[Migration],
    dry_run: bool,
) -> Result<Vec<MigrationStatus>, MigrationError> {
    validate_registry(migrations)?;

    let ledger_exists: bool =
        sqlx::query_scalar("SELECT to_regclass('public.urp_schema_migrations') IS NOT NULL")
            .fetch_one(pool)
            .await?;

    if !ledger_exists && dry_run {
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

    if !ledger_exists {
        sqlx::query(LEDGER_SQL).execute(pool).await?;
    }

    let applied_rows =
        sqlx::query("SELECT version, name, checksum FROM urp_schema_migrations ORDER BY version")
            .fetch_all(pool)
            .await?;

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
        sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(MIGRATION_LOCK_KEY)
            .execute(&mut *transaction)
            .await?;

        let existing =
            sqlx::query("SELECT name, checksum FROM urp_schema_migrations WHERE version = $1")
                .bind(migration.version)
                .fetch_optional(&mut *transaction)
                .await?;

        if let Some(row) = existing {
            let applied_name: String = row.try_get("name")?;
            let applied_checksum: String = row.try_get("checksum")?;
            verify_applied_migration(migration, &applied_name, &applied_checksum, &checksum)?;
        } else {
            // Migration SQL is trusted, versioned source code. raw_sql is required for DDL scripts containing multiple statements.
            sqlx::raw_sql(migration.sql)
                .execute(&mut *transaction)
                .await?;

            sqlx::query(
                "INSERT INTO urp_schema_migrations (version, name, checksum) VALUES ($1, $2, $3)",
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

#[cfg(test)]
mod tests {
    use std::env;

    use sqlx::{Row, postgres::PgPoolOptions};
    use uuid::Uuid;

    use super::{
        Migration, REGISTERED_MIGRATIONS, migration_checksum, postgres_migrate, run_with_migrations,
    };
    use crate::MigrationError;

    #[test]
    fn sha256_checksum_is_stable() {
        let checksum = migration_checksum("fixture", "SELECT 1");
        assert_eq!(checksum.len(), 64);
        assert_eq!(checksum, migration_checksum("fixture", "SELECT 1"));
        assert_ne!(checksum, migration_checksum("fixture", "SELECT 2"));
    }

    #[test]
    fn migration_registry_is_ordered_and_unique() {
        assert!(super::validate_registry(REGISTERED_MIGRATIONS).is_ok());
        assert_eq!(
            REGISTERED_MIGRATIONS.first().map(|item| item.version),
            Some(1)
        );
    }

    /// Uses only the dedicated disposable CI/test database, never application DATABASE_URL.
    #[tokio::test]
    async fn postgres_migrations_support_dry_run_idempotency_and_checksum_detection()
    -> Result<(), Box<dyn std::error::Error>> {
        let Ok(database_url) = env::var("URP_MIGRATION_TEST_DATABASE_URL") else {
            return Ok(());
        };

        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&database_url)
            .await?;

        let ledger_exists: bool =
            sqlx::query_scalar("SELECT to_regclass('public.urp_schema_migrations') IS NOT NULL")
                .fetch_one(&pool)
                .await?;
        if ledger_exists {
            return Err("URP migration test database must be empty".into());
        }

        let dry_run = run_with_migrations(&pool, REGISTERED_MIGRATIONS, true).await?;
        assert!(!dry_run.is_empty());
        assert!(dry_run.iter().all(|migration| !migration.applied));

        let first_run = postgres_migrate(&pool, false).await?;
        assert!(!first_run.is_empty());
        assert!(first_run.iter().all(|migration| migration.applied));

        let second_run = postgres_migrate(&pool, false).await?;
        assert_eq!(second_run, first_run);

        let altered_migrations = [
            Migration {
                version: 1,
                name: "0001_platform_metadata",
                sql: "SELECT 1",
            },
            REGISTERED_MIGRATIONS[1],
            REGISTERED_MIGRATIONS[2],
            REGISTERED_MIGRATIONS[3],
        ];
        let mismatch = run_with_migrations(&pool, &altered_migrations, false).await;
        assert!(matches!(
            mismatch,
            Err(MigrationError::ChecksumMismatch { version: 1, .. })
        ));

        let ledger_row = sqlx::query("SELECT COUNT(*) AS count FROM urp_schema_migrations")
            .fetch_one(&pool)
            .await?;
        let ledger_count: i64 = ledger_row.try_get("count")?;
        assert_eq!(ledger_count, REGISTERED_MIGRATIONS.len() as i64);

        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO urp_users (user_id, display_name, status) VALUES ($1, $2, 'active')",
        )
        .bind(user_id)
        .bind("Alice Example")
        .execute(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO urp_auth_identities (identity_id, user_id, provider_key, login_normalized, email_normalized, email_verified) VALUES ($1, $2, 'local', 'alice', 'alice@example.test', TRUE)",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .execute(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO urp_user_credentials (user_id, hash_scheme, hash_version, encoded_hash) VALUES ($1, 'argon2id', 1, '$argon2id$v=19$m=65536,t=3,p=1$c2FsdA$aGFzaA')",
        )
        .bind(user_id)
        .execute(&pool)
        .await?;

        let duplicate_login_user = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO urp_users (user_id, display_name, status) VALUES ($1, 'Second User', 'active')",
        )
        .bind(duplicate_login_user)
        .execute(&pool)
        .await?;
        let duplicate_login = sqlx::query(
            "INSERT INTO urp_auth_identities (identity_id, user_id, provider_key, login_normalized) VALUES ($1, $2, 'local', 'alice')",
        )
        .bind(Uuid::new_v4())
        .bind(duplicate_login_user)
        .execute(&pool)
        .await;
        assert!(duplicate_login.is_err());

        let duplicate_email = sqlx::query(
            "INSERT INTO urp_auth_identities (identity_id, user_id, provider_key, provider_subject, login_normalized, email_normalized) VALUES ($1, $2, 'oidc', 'subject-2', 'alice-oidc', 'alice@example.test')",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .execute(&pool)
        .await;
        assert!(duplicate_email.is_err());

        let disabled_without_timestamp = sqlx::query(
            "INSERT INTO urp_users (user_id, display_name, status) VALUES ($1, 'Invalid Disabled', 'disabled')",
        )
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await;
        assert!(disabled_without_timestamp.is_err());

        let delete_user_with_identity = sqlx::query("DELETE FROM urp_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await;
        assert!(delete_user_with_identity.is_err());

        sqlx::query("DROP TABLE urp_user_credentials")
            .execute(&pool)
            .await?;
        sqlx::query("DROP TABLE urp_auth_identities")
            .execute(&pool)
            .await?;
        sqlx::query("DROP TABLE urp_users").execute(&pool).await?;

        sqlx::query("DROP TABLE urp_platform_metadata")
            .execute(&pool)
            .await?;
        sqlx::query("DROP TABLE urp_schema_migrations")
            .execute(&pool)
            .await?;
        pool.close().await;
        Ok(())
    }
}
