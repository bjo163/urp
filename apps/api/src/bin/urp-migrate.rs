//! Explicit database migration command. Application startup never runs migrations.

use std::{env, error::Error};

use urp_storage::{Database, DatabaseConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let dry_run = arguments.iter().any(|argument| argument == "--dry-run");
    let status_only = arguments.iter().any(|argument| argument == "--status");

    if arguments
        .iter()
        .any(|argument| argument != "--dry-run" && argument != "--status")
    {
        return Err("supported arguments are --dry-run and --status".into());
    }

    let database_url = env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or("DATABASE_URL is required")?;

    let database = Database::connect(DatabaseConfig {
        url: database_url,
        max_connections: 2,
    })
    .await?;

    let migrations = if status_only {
        database.migration_status().await?
    } else {
        database.migrate(dry_run).await?
    };

    for migration in &migrations {
        let state = if migration.applied {
            "APPLIED"
        } else {
            "PENDING"
        };
        println!(
            "{state:7} {:04} {} sha256={}",
            migration.version, migration.name, migration.checksum
        );
    }

    if dry_run || status_only {
        println!("No database schema changes were made.");
    } else {
        println!(
            "Database migrations completed for {}.",
            database.backend_name()
        );
    }

    Ok(())
}
