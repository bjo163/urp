# URP — Local Development

## Prerequisites

- Git.
- Rust 1.99.1 with rustfmt and clippy (installed through `rust-toolchain.toml`).
- Node.js 26.x Active LTS.
- pnpm 10.9.2.
- Docker Engine + Docker Compose plugin for PostgreSQL.
- A terminal that can run the commands below.

## First boot

```bash
git clone https://github.com/bjo163/urp.git
cd urp
cp .env.example .env
docker compose --env-file .env -f infra/compose/dev.yml up -d
pnpm install
cargo check --workspace
```

## Run applications

Terminal 1 — Rust API:

```bash
cargo run -p urp-api
```

Terminal 2 — Next.js:

```bash
pnpm --filter @urp/web dev
```

The API listens on `http://127.0.0.1:8080` by default and the web app on `http://localhost:3000`. The current API skeleton exposes `/health/live` and `/health/ready`; database readiness is added in the storage phase.

## Canonical checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace

pnpm format:check
pnpm lint
pnpm typecheck
pnpm test
pnpm build
```

## Reset local PostgreSQL

```bash
docker compose --env-file .env -f infra/compose/dev.yml down
```

To delete local development data permanently:

```bash
docker compose --env-file .env -f infra/compose/dev.yml down -v
```

The `down -v` command deletes the named development database volume. Never run it against production.

## Working on an issue

Read `docs/AI_AGENT_PLAYBOOK.md`, the parent epic, the selected issue and the referenced sections of `docs/BLUEPRINT.md`. Pick only tasks whose dependencies are closed. Use one focused branch/PR per issue and include the exact commands/results in the PR description.
