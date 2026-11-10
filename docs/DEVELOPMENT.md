# URP Development Setup

## Requirements

- Git
- Rust 1.99.1 (rustfmt and clippy components are installed by the repository toolchain file)
- Node.js 24+
- pnpm 12.17.0
- Docker Engine with the Compose plugin

## First boot

1. Clone the repository and enter the project directory.
2. Copy `.env.example` to `.env`. These values are for local development only.
3. Start PostgreSQL:

   ```sh
   docker compose -f infra/compose/dev.yml up -d
   ```

4. Install the web dependencies:

   ```sh
   corepack enable
   corepack prepare pnpm@12.17.0 --activate
   pnpm install
   ```

5. Start the API from the repository root:

   ```sh
   set -a
   . ./.env
   set +a
   cargo run -p urp-api
   ```

6. Start the web app in another terminal:

   ```sh
   pnpm dev
   ```

The API listens on port 8080 by default. The web app listens on port 3000.

## Validation commands

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --filter @urp/web lint
pnpm --filter @urp/web typecheck
pnpm --filter @urp/web test
pnpm --filter @urp/web build
```

## Health endpoints

- `GET /health/live` reports that the API process is running.
- `GET /health/ready` checks the configured database connection.
- `GET /api/v1/` reports the public API version.

## Reset local PostgreSQL data

This command deletes the local development database volume. Never use it against production:

```sh
docker compose -f infra/compose/dev.yml down -v
```

## Current implementation boundary

The foundation contains the Rust workspace, typed kernel identifiers/errors/clock, SQLx PostgreSQL and SQLite connection pools, API health routes and a first Next.js landing workspace. It does not yet implement authentication, session management, database migrations, tenant policy, the ERP ORM, module lifecycle, or business domain APIs. Those remain tracked in GitHub Issues.
