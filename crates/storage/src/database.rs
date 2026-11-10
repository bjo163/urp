//! SQLx connection adapters for PostgreSQL and SQLite.

use std::{str::FromStr, time::Duration};

use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions},
};
use thiserror::Error;

use crate::migrations::{
    MigrationError, MigrationStatus, postgres_migrate, postgres_migration_status,
};

/// Configuration used to establish a database connection pool.
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    /// PostgreSQL or SQLite connection URL.
    pub url: String,
    /// Maximum number of connections in the pool.
    pub max_connections: u32,
}

/// A connected URP database backend.
#[derive(Clone)]
pub enum Database {
    /// PostgreSQL connection pool for production multi-user deployments.
    Postgres(PgPool),
    /// SQLite connection pool for embedded and single-host deployments.
    Sqlite(SqlitePool),
}

/// Errors returned by database connection and health operations.
#[derive(Debug, Error)]
pub enum StorageError {
    /// The configured URL is not a supported PostgreSQL or SQLite URL.
    #[error("DATABASE_URL uses an unsupported database scheme")]
    UnsupportedScheme,
    /// The underlying SQLx operation failed.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl Database {
    /// Connects to a supported database and configures its pool.
    pub async fn connect(config: DatabaseConfig) -> Result<Self, StorageError> {
        if config.url.starts_with("postgres://") || config.url.starts_with("postgresql://") {
            let pool = PgPoolOptions::new()
                .max_connections(config.max_connections)
                .connect(&config.url)
                .await?;
            return Ok(Self::Postgres(pool));
        }

        if config.url.starts_with("sqlite:") {
            let options = SqliteConnectOptions::from_str(&config.url)?
                .create_if_missing(true)
                .foreign_keys(true)
                .journal_mode(SqliteJournalMode::Wal)
                .busy_timeout(Duration::from_secs(5));

            let pool = SqlitePoolOptions::new()
                .max_connections(config.max_connections)
                .connect_with(options)
                .await?;
            return Ok(Self::Sqlite(pool));
        }

        Err(StorageError::UnsupportedScheme)
    }

    /// Executes a lightweight query against the active database backend.
    pub async fn health_check(&self) -> Result<(), StorageError> {
        match self {
            Self::Postgres(pool) => {
                sqlx::query("SELECT 1").execute(pool).await?;
            }
            Self::Sqlite(pool) => {
                sqlx::query("SELECT 1").execute(pool).await?;
            }
        }

        Ok(())
    }

    /// Applies registered PostgreSQL migrations explicitly.
    ///
    /// Use a dry run to inspect pending changes without writing schema or
    /// migration-ledger data. This method is never called at API startup.
    pub async fn migrate_postgres(
        &self,
        dry_run: bool,
    ) -> Result<Vec<MigrationStatus>, MigrationError> {
        match self {
            Self::Postgres(pool) => postgres_migrate(pool, dry_run).await,
            Self::Sqlite(_) => Err(MigrationError::UnsupportedBackend),
        }
    }

    /// Returns PostgreSQL migration state without making database changes.
    pub async fn migration_status_postgres(&self) -> Result<Vec<MigrationStatus>, MigrationError> {
        match self {
            Self::Postgres(pool) => postgres_migration_status(pool).await,
            Self::Sqlite(_) => Err(MigrationError::UnsupportedBackend),
        }
    }

    /// Returns the stable backend identifier used in diagnostics.
    pub const fn backend_name(&self) -> &'static str {
        match self {
            Self::Postgres(_) => "postgresql",
            Self::Sqlite(_) => "sqlite",
        }
    }
}
