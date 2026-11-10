# URP — Universal Radical Platform

**A next-generation, modular ERP platform built with Rust + Next.js.**

URP is a clean-room ERP platform inspired by the breadth and extensibility of Odoo, not an Odoo clone. It aims to unify business operations through strong transactional foundations, an extensible module runtime, a metadata-driven application layer and a distinctive radical enterprise interface.

## Product direction

- **Scope:** Full ERP platform, delivered incrementally through independently verifiable capabilities.
- **Compatibility:** Odoo-inspired concepts and workflows; no binary addon, ORM, API or source compatibility promise.
- **Backend:** Rust modular monolith, Axum HTTP API, SQLx persistence layer and a purpose-built ERP ORM above SQLx.
- **Frontend:** Next.js App Router + TypeScript with a first-party design system.
- **Databases:** PostgreSQL production default; SQLite for tested single-host/embedded deployments.
- **UX:** Radical enterprise — expressive industrial/grunge/neobrutalist identity with disciplined information architecture and accessibility.
- **Deployment:** Self-hosted first, Docker Compose reference topology.

## Source of truth

1. [Product narrative](docs/NARRATIVE.md)
2. [Technical blueprint](docs/BLUEPRINT.md)
3. [Engineering standards](docs/ENGINEERING_STANDARDS.md)
4. [Autonomous AI agent playbook](docs/AI_AGENT_PLAYBOOK.md)
5. [Roadmap](docs/ROADMAP.md)
6. [GitHub issue map](docs/ISSUE_MAP.md)
7. [Supported runtime matrix](docs/SUPPORT_MATRIX.md)
8. [Threat model](docs/security/THREAT_MODEL.md)

When documents conflict, stop and propose a focused documentation correction before changing code. GitHub Issues are the executable work queue; implement only the explicit scope of an issue.

## Development

See [Local Development](docs/DEVELOPMENT.md) and [Configuration Contract](docs/CONFIGURATION.md).

```bash
cp .env.example .env
docker compose --env-file .env -f infra/compose/dev.yml up -d
pnpm install
cargo check --workspace
pnpm dev:web
```

## Project status

URP is at the greenfield foundation stage. The repository currently contains the product specification and bootstrap workspace; business modules are not yet production functionality. Check the [GitHub issue map](docs/ISSUE_MAP.md) for the authoritative implementation backlog.

## Non-goals

- Claiming complete compatibility with every Odoo edition or third-party addon.
- Copying Odoo source code, proprietary assets or trademarks.
- Starting with microservices or distributed infrastructure without measured need.
- Replacing typed accounting and inventory invariants with dynamic metadata.
- Shipping radical visuals at the expense of readability, accessibility or operational trust.

## Licensing

The project license is intentionally pending maintainer approval. See [issue #2](https://github.com/bjo163/urp/issues/2). Do not distribute source bundles or binaries until the decision and required notices are committed.
