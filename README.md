# URP — Universal Radical Platform

**A next-generation, modular ERP platform built with Rust + Next.js.**

URP is a clean-room ERP platform inspired by the breadth and extensibility of Odoo, not an Odoo clone. It unifies business operations through a typed Rust core, a governed metadata/module runtime, PostgreSQL-first persistence with SQLite for single-host use, and a distinctive radical enterprise interface.

## Product decisions

- **Product scope:** Full ERP, delivered as independently verifiable capabilities.
- **Compatibility:** Odoo-inspired concepts and workflows. No Odoo source, internal ORM, API, database schema, or addon compatibility promise.
- **Backend:** Rust modular monolith, Axum HTTP API, SQLx persistence layer and a first-party ERP ORM above SQLx.
- **Frontend:** Next.js App Router, React and TypeScript with a custom design system.
- **Database:** PostgreSQL production default; SQLite embedded/single-host profile with explicitly tested capability limits.
- **UX:** Radical Enterprise — high-contrast industrial/editorial visual language, grunge/neobrutalist character, clear information design and accessibility.
- **Automation:** Durable jobs, outbox events and governed AI tools that use the same permission and approval boundaries as human actors.

## Repository status

The current implementation branch establishes the initial workspace, typed kernel primitives, SQLx PostgreSQL/SQLite connection adapters, a Rust API health boundary, a Next.js landing workspace, local PostgreSQL Compose configuration and CI workflows.

This is **foundation code, not a production ERP release**. Authentication, migrations, tenant authorization, ORM, module runtime and business domains remain in the issue backlog. Do not treat a planned module or UI card as an implemented feature.

## Start here

- [Product narrative](docs/NARRATIVE.md)
- [Technical blueprint](docs/BLUEPRINT.md)
- [Engineering standards](docs/ENGINEERING_STANDARDS.md)
- [AI agent playbook](docs/AI_AGENT_PLAYBOOK.md)
- [Roadmap](docs/ROADMAP.md)
- [Issue map and execution order](docs/ISSUE_MAP.md)
- [Development setup](docs/DEVELOPMENT.md)
- [Support matrix](docs/SUPPORT_MATRIX.md)
- [Contribution guide](CONTRIBUTING.md)

## Quick start

Requirements: Rust 1.99.1, Node.js 24+, pnpm 12.17.0 and Docker Compose.

```sh
cp .env.example .env
docker compose -f infra/compose/dev.yml up -d

corepack enable
corepack prepare pnpm@12.17.0 --activate
pnpm install

set -a
. ./.env
set +a
cargo run -p urp-api

# In another terminal
pnpm dev
```

The API listens on port 8080. The web app listens on port 3000.

## Architectural rules

1. Rust owns business invariants, authorization, transaction boundaries and durable side effects.
2. Next.js consumes the typed API; it never connects directly to a database.
3. First-party accounting and inventory invariants remain typed and transactional. Dynamic metadata does not replace them.
4. Modules consume declared public contracts and never reach into another module's private tables.
5. PostgreSQL is the default for multi-user production. SQLite WAL is for a single application host and must not be shared over a network filesystem.
6. Every implementation task must satisfy its GitHub issue acceptance criteria and the global engineering standards.

## Licensing

The repository's redistribution license is intentionally pending maintainer approval. See [issue #2](https://github.com/bjo163/urp/issues/2). Do not distribute releases until the license decision and required notices are committed.
