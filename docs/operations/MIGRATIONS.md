# URP Database Migrations

URP migrations are explicit release/deployment operations. The API process does not mutate production schema during startup.

## PostgreSQL commands

Set DATABASE_URL to a database the deployment migration role can modify.

Preview pending migrations without writing the schema or migration ledger:

```sh
cargo run -p urp-api --bin urp-migrate -- --dry-run
```

Inspect current migration state (also read-only):

```sh
cargo run -p urp-api --bin urp-migrate -- --status
```

Apply pending migrations:

```sh
cargo run -p urp-api --bin urp-migrate
```

The runner creates urp_schema_migrations, records ordered migration versions and SHA-256 checksums, and applies each schema change and ledger insert in one database transaction. A PostgreSQL transaction-scoped advisory lock serializes concurrent migration attempts.

## Safety rules

- Never edit a migration after it has been applied.
- Checksum or name drift is a hard error. Create a new migration instead.
- A running binary fails migration if the database contains an unknown applied version.
- Dry-run and status commands do not create the ledger or execute schema SQL.
- Production migration commands must be run explicitly with a migration-capable database role.
- Back up production data before applying a release that changes schema.
- Application runtime credentials should not have DDL privileges unless the deployment explicitly requires them.

## SQLite

SQLite migrations use a separate SQL dialect and capability checks. Do not assume PostgreSQL DDL is portable. Use a local file on the same host as the application; WAL files must not be shared over a network filesystem.
