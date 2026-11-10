# URP — Delivery Roadmap

The roadmap is capability-based, not calendar-based. Phase boundaries describe prerequisites and exit conditions; GitHub issue dependencies determine what can run in parallel.

| Phase | Area | Exit condition |
|---|---|---|
| P00 | Governance and specification | Narrative, blueprint, standards, agent playbook and task taxonomy approved |
| P01 | Workspace and developer experience | Reproducible Rust/Next.js workspace; format, lint, typecheck, test and CI baseline |
| P02 | Kernel and storage | IDs, errors, context, PostgreSQL/SQLite adapters, migrations and test fixtures |
| P03 | API and contracts | Versioned API skeleton, OpenAPI artifact, generated TypeScript client and error contract |
| P04 | Identity and organization | Users, sessions, tenant/company/branch context, membership and authorization primitives |
| P05 | ORM and metadata | Model/field registry, relations, domain AST/compiler, lifecycle, compute and migration plan |
| P06 | Module runtime | Manifest validation, dependency resolver, install/upgrade/disable lifecycle and module SDK |
| P07 | Radical UI foundation | Tokens, UI primitives, workspace shell, navigation, forms, lists, details and accessibility |
| P08 | CRM and commercial | Contacts, CRM, sales, procurement and end-to-end commercial workflow |
| P09 | Inventory and accounting | Stock ledger/movements, valuation, chart of accounts, journal posting, invoices and reconciliation baseline |
| P10 | Workforce and operations | HR, expenses, projects, timesheets, manufacturing, quality and maintenance foundations |
| P11 | Business experience | Documents, approvals, website, ecommerce, POS, helpdesk, subscriptions and customer operations |
| P12 | Intelligence and integrations | Reporting, import/export, workflow runtime, webhooks, integrations and governed AI |
| P13 | Production readiness | Security review, performance, observability, backup/restore, upgrades, localization and release gates |

## Release slices

### Internal Alpha
- One tenant and multiple companies/branches.
- Authentication and scoped permissions.
- Contacts, CRM and one sales flow.
- PostgreSQL deployment.
- Basic UI shell and audit trail.
- CI, Docker Compose and backup/restore baseline.

### Business Beta
- Procurement, inventory, accounting and invoicing baseline.
- Full tenant/company isolation tests.
- Module install/upgrade lifecycle.
- Both supported database profiles tested within declared capabilities.
- Critical E2E tests and operational metrics.

### Production Candidate
- HR/operations and selected advanced modules.
- Security and performance gates.
- Upgrade/restore rehearsals.
- Documentation for operators and implementers.
- Localization scope explicitly reviewed and signed off.

## Dependency policy

- Kernel and context precede storage/application modules.
- Identity and tenant context precede all tenant-owned business APIs.
- ORM registry and module runtime precede dynamic metadata screens.
- API contract generation precedes shared frontend domain screens.
- Inventory posting precedes manufacturing and stock valuation integrations.
- Accounting posting precedes payments, taxes and financial reports.
- Event outbox precedes reliable cross-module automations and external webhook delivery.
- UI primitives precede module-specific screen implementation.

## Scope control

The catalog in the blueprint is the full product ambition. The GitHub task list decomposes the first implementation path and its extensibility points. A missing domain is not permission to add unrelated work to an active issue: create a new scoped issue under the correct domain epic and link the relevant blueprint section.
