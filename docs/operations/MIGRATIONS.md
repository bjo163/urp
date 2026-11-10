# URP Database Migrations

URP migrations are explicit release/deployment operations. The API process does not mutate production schema during startup.

## Commands

Set DATABASE_URL to a database the deployment migration role can modify.

Preview pending migrations without writing schema or ledger data:

```sh
cargo run -p urp-api --bin urp-migrate -- --dry-run
```

Inspect current migration state (also read-only):

```sh
cargo run -p urp-api --bin urp-migrate -- --status
```

Apply pending migrations for the configured backend:

```sh
cargo run -p urp-api --bin urp-migrate
```

The runner records ordered migration versions and SHA-256 checksums. Each schema change and its ledger insert commit in one database transaction. PostgreSQL uses a transaction-scoped advisory lock to serialize concurrent migrators. SQLite uses a transactional write-lock row and the configured busy timeout.

## PostgreSQL

The PostgreSQL runner creates urp_schema_migrations with version, name, checksum and applied timestamp. It blocks checksum/name drift and unknown applied versions. Production migration commands must be run explicitly with a migration-capable database role.

## SQLite

SQLite has a separate SQL dialect and checks required capabilities before migration. The supported profile requires foreign keys and WAL journaling. Use a local database file on the same host as the application; WAL files must not be shared over a network filesystem. SQLite is not a multi-node shared database.

## Safety rules

- Never edit a migration after it has been applied.
- Checksum or name drift is a hard error. Create a new migration instead.
- Dry-run and status commands do not create the ledger or execute migration SQL.
- Back up production data before applying a release that changes schema.
- Application runtime credentials should not have DDL privileges unless the deployment explicitly requires them.
