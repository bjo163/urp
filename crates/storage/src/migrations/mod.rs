//! Explicit, versioned database migration contracts.

mod postgres;
mod sqlite;

use sha2::{Digest, Sha256};
use thiserror::Error;

/// A registered, immutable schema migration.
#[derive(Clone, Copy)]
pub(crate) struct Migration {
    pub(crate) version: i64,
    pub(crate) name: &'static str,
    pub(crate) sql: &'static str,
}

/// The current result of a registered migration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationStatus {
    /// Monotonically increasing migration version.
    pub version: i64,
    /// Stable technical name of the migration.
    pub name: String,
    /// SHA-256 checksum of the migration name and SQL contents.
    pub checksum: String,
    /// Whether the migration is already recorded as applied.
    pub applied: bool,
}

/// Errors returned by explicit schema migration operations.
#[derive(Debug, Error)]
pub enum MigrationError {
    /// The selected operation is not implemented for the active backend.
    #[error("migration operation is not supported by the active database backend")]
    UnsupportedBackend,
    /// The active database lacks a required feature.
    #[error("database backend {backend} does not provide required capability {capability}")]
    UnsupportedCapability {
        /// Database backend being checked.
        backend: &'static str,
        /// Required capability that is unavailable.
        capability: &'static str,
    },
    /// An applied migration differs from the source currently registered in the binary.
    #[error(
        "migration checksum mismatch for version {version} ({name}): applied {applied}, registered {registered}"
    )]
    ChecksumMismatch {
        /// Migration version with a mismatch.
        version: i64,
        /// Stable migration name.
        name: String,
        /// Checksum stored in the database.
        applied: String,
        /// Checksum computed from the current registered migration.
        registered: String,
    },
    /// The database contains a migration version not known by the running binary.
    #[error("database contains unknown applied migration version {version}")]
    UnknownAppliedMigration {
        /// Unknown version found in the migration ledger.
        version: i64,
    },
    /// Registered migrations are not strictly ordered by unique version.
    #[error("registered migrations must have unique, strictly increasing versions")]
    InvalidRegistry,
    /// A SQLx operation failed.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub(crate) fn migration_checksum(name: &str, sql: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    hasher.update([0]);
    hasher.update(sql.as_bytes());

    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn validate_registry(migrations: &[Migration]) -> Result<(), MigrationError> {
    if migrations
        .windows(2)
        .any(|pair| pair[0].version >= pair[1].version)
    {
        return Err(MigrationError::InvalidRegistry);
    }

    Ok(())
}

pub(crate) fn verify_applied_migration(
    migration: &Migration,
    applied_name: &str,
    applied_checksum: &str,
    registered_checksum: &str,
) -> Result<(), MigrationError> {
    if applied_name != migration.name || applied_checksum != registered_checksum {
        return Err(MigrationError::ChecksumMismatch {
            version: migration.version,
            name: migration.name.to_owned(),
            applied: applied_checksum.to_owned(),
            registered: registered_checksum.to_owned(),
        });
    }

    Ok(())
}

pub(crate) use postgres::{postgres_migrate, postgres_migration_status};
pub(crate) use sqlite::{sqlite_migrate, sqlite_migration_status};
