//! SQLx connection adapters and explicit schema migration support.

mod database;
mod migrations;

pub use database::{Database, DatabaseConfig, StorageError};
pub use migrations::{MigrationError, MigrationStatus};
