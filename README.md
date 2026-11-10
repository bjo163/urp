# URP — Universal Radical Platform

**A next-generation, modular ERP platform built with Rust + Next.js.**

URP is a clean-room ERP platform inspired by the breadth and extensibility of Odoo, not an Odoo clone. Its goal is to provide a cohesive business operating system: strong transactional foundations, an extensible module runtime, a metadata-driven application layer, and a distinctive radical enterprise interface.

## Product direction

- **Scope:** Full ERP platform, delivered incrementally by independently verifiable capabilities.
- **Compatibility:** Odoo-inspired concepts and workflows; no binary addon, ORM, API, or source compatibility promise.
- **Backend:** Rust modular monolith; Axum HTTP API; SQLx as the SQL execution layer; a purpose-built metadata/ORM runtime above it.
- **Frontend:** Next.js App Router + TypeScript, with a custom design system.
- **Databases:** PostgreSQL as the production default; SQLite as a supported single-host/embedded deployment target with explicitly tested capability differences.
- **UX:** Radical enterprise — high-contrast, expressive, grunge/neobrutalist visual identity with disciplined information architecture, accessibility, and usability.
- **Execution:** Small, non-overlapping GitHub issues designed for autonomous local AI agents.

## Source of truth

1. [Product narrative](docs/NARRATIVE.md) — why URP exists and what it promises.
2. [Technical blueprint](docs/BLUEPRINT.md) — architecture, domain boundaries, runtime contracts, and delivery roadmap.
3. [Engineering standards](docs/ENGINEERING_STANDARDS.md) — non-negotiable implementation and quality rules.
4. [AI agent playbook](docs/AI_AGENT_PLAYBOOK.md) — how autonomous agents select, implement, test, and close tasks.
5. [Roadmap](docs/ROADMAP.md) — scope and phase exit criteria.
6. [Issue map](docs/ISSUE_MAP.md) — all epic/task ranges and execution order.
7. GitHub Issues — the executable work queue. An issue may implement only its explicitly stated scope.

When documents conflict, stop implementation and propose a focused documentation correction before changing code. Do not invent architecture in an implementation issue.

## Project status

The repository was initialized from a greenfield baseline. Nothing is production-ready until the relevant issue acceptance criteria, automated checks, security review, and release gates have passed.

## Quick links

- [Narrative](docs/NARRATIVE.md)
- [Blueprint](docs/BLUEPRINT.md)
- [Engineering standards](docs/ENGINEERING_STANDARDS.md)
- [AI agent playbook](docs/AI_AGENT_PLAYBOOK.md)
- [Roadmap](docs/ROADMAP.md)
- [Issue map](docs/ISSUE_MAP.md)
- [Issue task template](.github/ISSUE_TEMPLATE/implementation-task.md)

## Non-goals

- Claiming feature parity with every Odoo edition or third-party addon.
- Importing or copying Odoo source code, proprietary assets, or non-open specifications.
- Starting with microservices or a distributed deployment topology.
- Treating dynamic metadata as a substitute for typed financial and inventory invariants.
- Shipping a visually radical interface at the expense of accessibility, clarity, or operational trust.

**Licensing:** No redistribution license has been selected yet. Do not publish or distribute binaries/source bundles until [issue #2](https://github.com/bjo163/urp/issues/2) is approved and the decision is recorded.
