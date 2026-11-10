# URP — Technical Blueprint

> Status: Baseline architecture for greenfield implementation. This document is normative until changed through a documented architecture issue.
>
> Product scope: full ERP platform, Odoo-inspired concepts, independent implementation, radical enterprise UX.

## 1. Executive architecture

URP is a **modular monolith first**. The first production topology contains one Rust API process, one Rust worker process, one Next.js web application, and one database. Realtime delivery is initially part of the API/worker boundary; it is not a separately deployed service until operational evidence justifies one.

```text
People / Browser / Integrations / Governed AI Agents
                     |
               HTTPS / WSS
                     |
       Next.js App Router (apps/web)
  Workspace • Navigation • Views • Forms • Dashboards
                     |
            Generated/typed SDK
                     |
          Versioned HTTP API (/api/v1)
                     |
          Rust Application Boundary
       Axum • Auth • Context • DTO/Contracts
                     |
        Application / Domain Modules
 CRM • Sales • Purchase • Stock • Finance • HR • MRP
                     |
     Shared Platform Kernel and Registries
 Identity • Tenant • Company • Module • Metadata
 Policy • Audit • Events • Localization • Jobs
                     |
            Storage Adapters (SQLx)
               /             \
       PostgreSQL           SQLite
       production           single-host
```

The web application is a presentation client. Rust is the system of record for authorization, business invariants, transactions, workflow transitions, and durable side effects. Next.js server components may read through the typed API/SDK, but they do not implement a second business backend.

### Fixed decisions

| Concern | Decision |
|---|---|
| Workspace | Monorepo |
| Frontend | Next.js App Router, React, TypeScript |
| Frontend runtime | Node.js-compatible deployment; choose and pin supported runtime in bootstrap issue |
| Backend | Rust stable toolchain, Tokio, Axum |
| Architecture | Modular monolith with explicit module boundaries |
| SQL access | SQLx, parameterized queries and transactions |
| ERP ORM | First-party metadata/model/query engine above SQLx |
| Database | PostgreSQL production default; SQLite supported target |
| API | Versioned REST/JSON, OpenAPI contract |
| Realtime | Outbox/event-driven WebSocket/SSE delivery, initially co-deployed |
| Background jobs | Rust worker, durable job table/queue abstraction |
| Frontend API client | Generated/typed TypeScript SDK from OpenAPI |
| Deployment | Docker Compose reference deployment |
| UI | First-party design system, radical enterprise visual identity |
| Testing | Unit + integration + contract + property + E2E + security + migration tests |

The official Next.js App Router uses Server Components by default and Client Components where state, events, or browser APIs are required; URP follows that division instead of making the whole application client-rendered. See [Next.js App Router](https://nextjs.org/docs/app) and [Server and Client Components](https://nextjs.org/docs/app/getting-started/server-and-client-components).

## 2. Repository structure

The following is the target structure. Issues may introduce directories gradually; do not create empty placeholders for every planned capability.

```text
urp/
├── apps/
│   ├── web/                         # Next.js user experience
│   ├── api/                         # Rust Axum HTTP application
│   └── worker/                      # Rust durable jobs and outbox consumers
├── crates/
│   ├── kernel/                      # Shared IDs, errors, context, primitives
│   ├── domain/                      # Domain contracts, no transport/database types
│   ├── application/                 # Use cases and transaction boundaries
│   ├── api-contract/                # DTO/OpenAPI generation boundary
│   ├── auth/                        # Authentication/session primitives
│   ├── policy/                      # RBAC, record and field authorization
│   ├── tenancy/                     # Tenant/company/branch context
│   ├── audit/                       # Immutable audit event contract
│   ├── events/                      # Domain event and outbox contracts
│   ├── jobs/                        # Durable job model and scheduling
│   ├── orm/                          # Metadata-driven model/query runtime
│   ├── orm-derive/                  # Optional typed-model derive macros
│   ├── module-runtime/              # Module manifest, registry, lifecycle
│   ├── storage/                     # SQLx pools, transactions, dialect adapters
│   ├── localization/                # Locale, currency, timezone, numbering
│   ├── reporting/                   # Report definitions and query boundary
│   └── test-support/                # Factories, fixtures, test DB utilities
├── modules/
│   ├── base/
│   ├── contacts/
│   ├── crm/
│   ├── sales/
│   ├── purchase/
│   ├── inventory/
│   ├── accounting/
│   ├── expenses/
│   ├── hr/
│   ├── payroll/
│   ├── projects/
│   ├── manufacturing/
│   ├── quality/
│   ├── maintenance/
│   ├── documents/
│   ├── website/
│   ├── ecommerce/
│   ├── point-of-sale/
│   ├── helpdesk/
│   └── localization-*/
├── packages/
│   ├── ui/                          # First-party component system
│   ├── design-tokens/
│   ├── api-client/                  # Generated client
│   ├── contracts/                   # Shared schemas and API artifacts
│   ├── eslint-config/
│   └── typescript-config/
├── docs/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   └── workflows/
├── infra/
│   ├── docker/
│   ├── compose/
│   └── scripts/
├── Cargo.toml
├── Cargo.lock
├── package.json
├── pnpm-workspace.yaml
├── rust-toolchain.toml
└── README.md
```

### Boundary rules

- `apps/*` are composition roots. They wire dependencies; they do not own domain logic.
- `crates/kernel` may contain shared primitives but must not become a dumping ground for business features.
- `crates/domain` must not import Axum, SQLx, Next.js, or database-specific types.
- `modules/<name>` owns that module's domain behavior and persistence definitions.
- Modules may depend only on declared public contracts of other modules.
- `packages/ui` is a reusable visual system; app-specific pages stay in `apps/web`.
- Generated code is labeled and reproducible. Agents must not hand-edit generated artifacts.

## 3. Rust backend architecture

### 3.1 Layering

Each business module follows a consistent vertical structure:

1. **Domain** — entities/value objects, invariants, domain services, domain events.
2. **Application** — command/query use cases, authorization checks, transaction orchestration.
3. **Ports** — repository, clock, ID, event, and external-service interfaces where abstraction is useful.
4. **Adapters** — SQLx repositories, external API clients, message delivery.
5. **Transport** — Axum handlers and DTO mapping, kept at the app/API edge.

Do not create abstractions without a real boundary. Simple internal functions are preferred over generic frameworks. The rule is dependency direction: domain behavior must not depend on transport or storage implementation.

### 3.2 API process

The API process owns:
- HTTP routing and versioning.
- Request IDs, tracing, error mapping and validation.
- Authentication/session extraction.
- Tenant, company, branch, locale, timezone and actor context.
- Authorization policy evaluation.
- Application use-case invocation.
- Rate limits, idempotency keys, pagination and response envelopes.
- OpenAPI generation and health/readiness endpoints.

Use explicit DTOs; never expose database rows or internal domain structs as public API by accident. API handlers should be thin and deterministic.

### 3.3 Worker process

The worker owns:
- Outbox delivery.
- Durable scheduled jobs.
- Retry/backoff and dead-letter handling.
- Notifications and webhook delivery.
- Import/export processing.
- Long-running reports and document tasks.
- Automation actions that are safe to run asynchronously.

Jobs must have stable IDs, attempt counters, next-run timestamps, deduplication keys where needed, and observable terminal failure. At-least-once delivery is assumed; handlers must be idempotent or explicitly guarded.

### 3.4 Eventing

Domain events are emitted inside the same database transaction as the state change via an outbox record. The worker publishes/delivers after commit. Do not pretend database writes and external calls form one atomic transaction.

Event envelopes include event ID, event type/version, tenant context, aggregate ID/version, actor, occurred-at timestamp, correlation ID, and a schema payload. Consumers validate versions and record idempotent processing.

## 4. Database and persistence strategy

### 4.1 Database roles

**PostgreSQL is the production default** for concurrent multi-user, multi-company and multi-node workloads.

**SQLite is supported** for embedded, local, evaluation, and single-host deployments. Enable and verify WAL mode where appropriate, configure foreign keys, busy timeout, and transaction behavior. SQLite WAL improves reader/writer concurrency but all WAL participants must be on the same host; it is not a shared network-filesystem database. See [SQLite WAL](https://www.sqlite.org/wal.html).

SQLite is not marketed as an equivalent multi-node HA database. A deployment requiring multiple application nodes, high write concurrency, or managed failover must use PostgreSQL.

### 4.2 SQLx role

SQLx is the low-level async SQL toolkit and database driver layer, not the ERP ORM. It supports PostgreSQL and SQLite, typed query APIs, connection pools, and transactions; the URP ORM supplies model metadata, domain queries, relations, lifecycle hooks and access-aware behavior above it. See [SQLx documentation](https://docs.rs/sqlx/latest/sqlx/) and [SQLx crate overview](https://docs.rs/crate/sqlx/latest).

Do not hide dialect differences behind an untested lowest-common-denominator abstraction. Use explicit PostgreSQL/SQLite adapters and capability tests. Avoid relying on runtime `Any` where it weakens compile-time driver selection or obscures database-specific behavior.

### 4.3 Schema ownership

- First-party platform and business tables use versioned, reviewed migrations.
- Every schema change is additive and reversible where practical; destructive changes require a staged data migration and backup plan.
- Module migrations are namespaced, ordered, checksummed, and recorded.
- Application startup must not silently mutate production schemas.
- Production schema changes run through an explicit migration command/release step.
- Indexes, unique constraints, foreign keys, check constraints and deletion behavior are designed with the domain model, not added as afterthoughts.

### 4.4 ORM model storage

URP uses a hybrid model:

- **First-party core models:** normalized relational tables, Rust-typed domain models, explicit migrations.
- **Custom metadata models:** metadata registry creates validated model storage through a controlled schema plan. Each model has a stable technical name, field metadata, ownership scope, and schema version. It is not a free-form SQL execution facility.
- **Custom fields on core models:** registered with explicit type, owner, indexing/search policy, security metadata, and migration state. A field is not queryable merely because it appears in a form.
- **Computed fields:** pure registered Rust functions for trusted first-party behavior; expressions use a constrained AST/DSL, never arbitrary `eval` or generated code execution.
- **Views/actions:** metadata describes presentation and allowed operations, not hidden business invariants.

A metadata schema update must be previewable, validated for compatibility, recorded, and recoverable. Removing a field is not a silent data deletion.

### 4.5 IDs, money and time

- IDs are opaque, stable, non-sequential externally; choose a UUID/ULID strategy in the foundation issue and keep it consistent.
- Money and quantities use exact decimal arithmetic or integer minor units with explicit currency/precision rules. Never use binary floating point for financial values.
- Store instants in UTC; preserve timezone identifiers for schedules and display.
- Use explicit currency, unit-of-measure and rounding policies.
- Number sequences are transactional or reservation-based with documented gap behavior; never assume a gapless sequence unless legally required and independently designed.

### 4.6 Tenant data model

- **Tenant** is the isolation and subscription boundary.
- **Company** is a legal/accounting entity within a tenant.
- **Branch/site** is an operating location or organizational subdivision; it is not automatically a separate legal entity.
- **Warehouse/location** is an inventory structure and is not interchangeable with a branch.
- **Department/team** is an organizational grouping.
- Every tenant-owned row carries a tenant scope. Company scope is present only where business ownership requires it.
- Cross-company sharing is explicit and policy controlled.

## 5. URP ORM and metadata engine

The ORM is a product capability, not a generic replacement for SQL. It provides the model contract used by the module runtime, dynamic views, permissions, and custom models.

### 5.1 Core subsystems

1. **Model registry** — stable technical model names, module ownership, model version and lifecycle.
2. **Field registry** — type, required, nullable, default, label, relation, indexing, compute, storage, copy, tracking and security metadata.
3. **Relations** — many-to-one, one-to-many and many-to-many with explicit inverse fields, on-delete policy and tenant constraints.
4. **Domain AST** — typed predicate tree for AND/OR/NOT, comparisons, membership, null, relation traversal and safe operators.
5. **Query compiler** — parameterized SQL generation for PostgreSQL and SQLite, with explicit capability errors.
6. **Repository/record API** — CRUD and search operations with explicit transaction and request context.
7. **Lifecycle pipeline** — validation, before/after hooks, computed dependency updates, domain events and audit.
8. **Computed fields** — dependency graph, cycle detection, deterministic evaluation, cache invalidation.
9. **Metadata migration planner** — safe DDL/data transitions for model and field changes.
10. **Access integration** — model/action/field/record rules applied before data access and mutation.
11. **Registry cache** — versioned and invalidated after module/schema changes.
12. **Introspection** — model and field metadata exposed only to authorized administrative APIs.

### 5.2 Field type baseline

Initial scalar types: text, long text, boolean, integer, decimal, date, datetime, JSON (restricted), UUID/ULID reference, selection/enum, binary reference, and monetary amount.

Relation types: many-to-one, one-to-many, many-to-many.

Field metadata also supports required/nullable, default policy, label/help, index strategy, unique constraint, read/write permission, stored/computed, dependency list, tracking, copy policy and on-delete behavior.

Do not add a type without defining serialization, validation, database mapping, query operators, UI editor, import/export, and migration semantics.

### 5.3 Query and record semantics

- Domain expressions are parsed into an AST and validated against the authorized model registry.
- All values are bound as parameters; field names/operators are resolved only through registry allowlists.
- Sorting and grouping are allowlisted; raw SQL never comes from client-supplied metadata.
- Pagination has maximum limits and stable ordering.
- Record rules are applied to search, count, read, write and delete paths, including relation traversal and aggregate queries.
- Batch operations are explicit and transactional where business invariants require it.
- A direct repository escape hatch is allowed for performance-critical typed queries, but it must preserve tenant, permission, audit and transaction requirements.

### 5.4 ORM non-goals

- Full Python/Odoo ORM compatibility.
- Arbitrary user-defined executable methods.
- Arbitrary SQL through view metadata.
- Automatic silent schema mutation.
- Using dynamic records for accounting ledgers or stock valuation as a substitute for typed domain models.

## 6. Module runtime

Each module has a manifest such as `urp.module.toml` with:
- Technical ID and display metadata.
- Module version and supported platform API range.
- Declared dependencies and optional integrations.
- Model and field registrations.
- Migration list and checksum.
- Permissions and record rule declarations.
- Menus, actions, views and navigation metadata.
- Event subscriptions and job handlers.
- API route registration.
- Assets/translations.
- Installation and upgrade hooks with restricted lifecycle contracts.
- Demo/test data references.

### Lifecycle requirements

1. Discover manifests without executing module code.
2. Validate schema, identity, version, dependency and permission metadata.
3. Resolve dependencies deterministically with cycle detection.
4. Build a dry-run plan.
5. Apply migrations and registry changes transactionally where supported.
6. Record installed module version and checksums.
7. Activate routes, jobs, views and event handlers only after successful commit.
8. Upgrade through versioned migration steps.
9. Disable a module without deleting customer data.
10. Uninstall only through an explicit policy and data-impact preview.

Modules must not mutate another module's private tables. Cross-module operations go through public application commands, domain events or declared interfaces.

## 7. Security, identity and tenancy

### 7.1 Authentication

Initial release:
- Local email/username + password authentication with a vetted password-hashing algorithm and safe parameters.
- Server-side sessions or opaque refreshable tokens; browser authentication uses secure, HttpOnly cookies.
- Session rotation and revocation.
- Password reset tokens with single-use and expiry.
- Optional OIDC integration through a standard provider boundary.
- MFA is a subsequent security capability, not a reason to invent a custom cryptographic protocol.

Never store plaintext passwords, session secrets, API keys, or refresh tokens in source control or logs.

### 7.2 Authorization

Authorization is layered:
- Tenant membership.
- Company/branch context.
- Role and permission grants.
- Model CRUD permissions.
- Action-level permissions.
- Field-level read/write controls.
- Record-level rules.
- Sensitive-operation policies and approvals.

Client-side hidden buttons are usability only, never authorization. Every API command and query is checked server-side.

### 7.3 Tenant isolation

Tenant identity is resolved from the authenticated principal and validated membership; a client-supplied tenant ID is merely a requested context, never trusted authority.

All tenant queries carry a typed context. Query compilation and repository APIs require scoped context for tenant-owned data. PostgreSQL Row-Level Security may be used as defense in depth, with transaction-scoped context and tests for pooled-connection leakage. Application-level enforcement remains mandatory so SQLite has equivalent behavior. PostgreSQL RLS must be tested with the actual application role, not a superuser or table owner that bypasses policies.

### 7.4 Security operations

- CSRF protection for cookie-authenticated state-changing requests.
- CORS allowlists.
- Input validation and output encoding.
- Rate limits on authentication and sensitive operations.
- Secrets from environment/secret store.
- TLS at ingress and encrypted backup storage.
- Audit logging for auth, permission changes, data exports, financial postings, stock adjustments, module install/upgrade, and privileged administration.
- Dependency and container vulnerability scanning.
- Threat model and security review before production.
- No sensitive content in logs by default.

## 8. Core shared platform domains

| Domain | Ownership |
|---|---|
| Identity | Users, credentials, sessions, service identities |
| Organization | Tenant, company, branch, department, teams |
| Contacts | People, organizations, addresses, communication channels |
| Access | Roles, permissions, record rules, policy checks |
| Module registry | Module manifests, dependencies, lifecycle |
| Metadata | Models, fields, views, actions, menus |
| Audit | Actor/action/target/change trail |
| Events | Domain event schema and outbox |
| Jobs | Durable background execution |
| Localization | Language, timezone, currency, units, number formats |
| Documents | File metadata, attachments, access policy and storage references |
| Notifications | In-app and external delivery preferences |
| Search | Permission-aware indexing and search |
| Reporting | Read models, exports, dashboards and report definitions |

## 9. Functional ERP domain catalog

This is the complete product ambition, not a claim that every domain ships in the first milestone. Each domain is delivered as a separate module with its own domain invariants, API, tests, UI and documentation.

### 9.1 Foundation and organization
- Users, invitations, credentials, sessions, teams and service accounts.
- Tenants, companies, branches, departments, locations and operating calendars.
- Contacts, people, organizations, addresses, tags and communication channels.
- Roles, permission sets, record rules, delegated access and approval authority.

### 9.2 CRM and commercial operations
- Leads, lead sources, qualification, assignment and scoring.
- Opportunities, pipelines, stages, forecasts, activities and customer history.
- Customer and vendor master data.
- Quotations, pricing rules, pricelists, discounts, taxes and sales orders.
- Delivery commitments, order confirmation, returns and credit notes.
- Subscriptions, recurring billing, renewals and contract lifecycle.
- Commissions and sales team targets as later commercial extensions.

### 9.3 Procurement and supplier operations
- Vendor master data and supplier catalogs.
- Purchase requests and approval workflow.
- Request for quotation, bid comparison and purchase orders.
- Blanket agreements, reorder policies and purchase replenishment.
- Goods receipt, quality inspection and vendor returns.
- Vendor bills, matching and payment handoff to accounting.

### 9.4 Inventory and logistics
- Warehouses, zones, locations, routes and putaway rules.
- Products, variants, categories, attributes, units of measure and packaging.
- Receipts, deliveries, internal transfers and adjustments.
- Reservations, availability and replenishment.
- Lots, batches, serial numbers and expiration dates.
- Barcode operations, picking, packing and shipping integrations.
- Returns, scrap, consignment and multi-warehouse transfers.
- Inventory valuation policies and ledger reconciliation.

Stock must be movement-ledger based. On-hand values are derived or maintained as rebuildable projections. Every movement has source/destination, quantity/UoM, product, company/warehouse context, actor, timestamp, reason and idempotency reference where applicable.

### 9.5 Accounting and finance
- Chart of accounts and account groups.
- Journals, sequences, fiscal years and accounting periods.
- Balanced journal entries and immutable posted moves.
- Customer invoices, vendor bills, credit/debit notes.
- Receipts, payments, bank statements and reconciliation.
- Taxes, tax groups, fiscal positions and withholding support.
- Multi-currency rates and realized/unrealized FX.
- Accounts receivable/payable aging.
- Cash/bank, fixed assets, depreciation and disposals.
- Analytic accounting, budgets, cost centers and financial statements.
- Consolidation and intercompany accounting as advanced scope.

Financial rules:
- Every posted journal entry balances exactly by currency/company rounding policy.
- Posted entries are immutable; correction is by reversal/adjustment with references.
- Use exact decimal arithmetic; never `f32`/`f64` for money.
- Posting is transactional and idempotent.
- Period locks, permissions and audit are mandatory.
- Tax behavior is localized and reviewed; the generic core must not claim jurisdiction-specific compliance.

### 9.6 People and HR
- Employee profiles, contracts and org chart.
- Recruitment, applicants and onboarding.
- Attendance, shifts, work calendars and overtime.
- Leave/time-off requests, approvals and accruals.
- Expenses and reimbursements.
- Payroll rules, payslips, deductions and payment export as jurisdiction-specific modules.
- Skills, appraisals, training, equipment and offboarding.
- Workforce planning and timesheet integration.

Payroll and employment compliance require explicit country localization and qualified review.

### 9.7 Projects and services
- Projects, milestones, tasks, dependencies, Kanban and Gantt read models.
- Timesheets, billing rules, budgets and profitability.
- Resource planning and capacity.
- Field service scheduling, dispatch, worksheets and parts usage.
- Helpdesk tickets, SLAs, queues, escalation and knowledge links.
- Maintenance requests and service history.

### 9.8 Manufacturing and quality
- Bills of materials and revisions.
- Work centers, routing, operations and production calendars.
- Manufacturing orders, component consumption and finished goods.
- Work-in-progress and scrap.
- Subcontracting and planning.
- Quality points, inspections, nonconformance and corrective action.
- Maintenance plans, equipment, preventive schedules and downtime.
- PLM/change orders as advanced scope.

Manufacturing must consume and produce inventory through explicit stock movements and preserve cost traceability.

### 9.9 Commerce and customer experience
- Website content and page composition.
- Product catalog, variants, pricing and availability.
- Shopping cart, checkout, order lifecycle and fulfillment handoff.
- Payment provider adapters; secrets and payment data handled only through compliant provider flows.
- Point of sale, sessions, cash control, refunds and offline synchronization scope.
- Promotions, coupons, loyalty and gift cards.
- Marketing campaigns, segments, consent and communication history.
- Rental, bookings and e-signature as separately scoped extensions.

### 9.10 Documents, approvals and knowledge
- Document metadata, storage adapters, preview and retention.
- Access-controlled attachments and version history.
- Approval requests, delegation, quorum and audit.
- Signatures through provider integration or legally reviewed signing implementation.
- Knowledge articles, categories and permission-aware search.

### 9.11 Analytics, automation and integrations
- Saved filters, pivot/grouping, dashboards and scheduled reports.
- Import/export with preview, validation, error report and idempotent batch handling.
- Webhooks and API credentials with scopes and rotation.
- Workflow definitions, trigger conditions, actions, retries and execution history.
- Notification routing and user preferences.
- AI assistant/agents that call approved tools and use normal authorization and workflow.
- Integration adapters for accounting, banking, shipping, payment, email, identity and external business systems.

### 9.12 Localization
Localization is a separate module family for country-specific chart of accounts, tax, invoice numbering, fiscal reporting, payroll, bank formats, and electronic government reporting. A locale module must declare jurisdiction, effective dates, supported versions, source references, tests, and validation owner. Indonesia localization is a planned module, not a universal assumption in the core.

## 10. Cross-domain business invariants

- A confirmed sales order creates downstream delivery/invoice actions only through explicit application commands/events.
- A confirmed purchase order can create receipt expectations; receipts record actual accepted quantities.
- Inventory cannot become negative unless an explicit company policy permits it; policy is audited.
- Stock reservation, release, delivery, return and valuation are separate transitions.
- Accounting entries are posted only after validation, balanced exactly, and cannot be silently edited.
- Business records have explicit lifecycle states and allowed transition graphs.
- Every external callback is authenticated, replay protected and idempotent.
- Every import has a dry-run/validation report before commit.
- Every automation action is authorized as the acting user/service principal and recorded.
- Deleting master data referenced by posted transactions is restricted; archive/deactivate is preferred.

## 11. Frontend architecture and radical UX

### 11.1 Structure

- Next.js App Router with route groups for auth, workspace, administration and public content.
- Server Components by default for layout and server-side data reads.
- Client Components only for interactive widgets, local state, browser APIs and rich editors.
- API calls through the typed URP SDK; no direct database access from the frontend.
- Shared design tokens and primitives in `packages/design-tokens` and `packages/ui`.
- Module screens compose shared primitives and module-specific workflows.
- Forms derive validation and metadata from authorized API contracts, not arbitrary frontend-generated SQL semantics.

### 11.2 Visual system

**Direction:** radical enterprise / industrial neobrutalist / grunge, not a generic SaaS dashboard.

Design rules:
- Strong typography hierarchy, editorial labels, clear numeric hierarchy.
- Deliberate grid, visible structure, confident borders and controlled asymmetry.
- Restrained accent palette; semantic status colors remain consistent.
- Dark and light themes must both satisfy contrast and focus visibility.
- Dense lists and tables optimize scanning, alignment, sorting, filtering and bulk actions.
- Motion is purposeful, short, and disabled/reduced when requested.
- No decoration may hide business state, warnings, permissions or totals.
- Use tokens for colors, spacing, radius, elevation, typography, motion and z-index.
- No hard-coded colors in business feature components.
- All interactive elements have focus states, labels, keyboard behavior and accessible names.
- Empty/loading/error/success/permission-denied/offline states are designed, not left blank.
- Mobile layouts prioritize critical tasks; desktop is the primary high-density workspace.

### 11.3 Core UI patterns
- Application launcher and workspace shell.
- Global command/search palette.
- Breadcrumbs, contextual side navigation and saved favorites.
- List/table with filters, search, grouping and bulk actions.
- Record detail with activity/history and related records.
- Form sections, relational picker, inline validation and unsaved-change guard.
- Kanban for supported lifecycle workflows.
- Calendar/timeline/planning views where domain semantics justify them.
- Dashboard tiles, charts and drill-down reports.
- Approval inbox and activity stream.
- Admin metadata/module/security screens.

## 12. API and contract strategy

- Prefix public application APIs with `/api/v1`.
- Use REST resources and explicit command endpoints for lifecycle transitions.
- Publish OpenAPI as a build artifact.
- Generate a TypeScript client from the API schema; do not manually duplicate DTOs.
- Use stable error codes, validation paths, request IDs and correlation IDs.
- Use cursor pagination for large/volatile collections; bounded offset pagination only where justified.
- Use idempotency keys for payment, posting, import, webhook and other retry-prone commands.
- Version breaking API changes intentionally.
- Keep internal module events separate from public webhook schemas.
- Never expose database table names or ORM internals as public compatibility promises.

## 13. Security and privacy architecture

Security is a cross-cutting acceptance requirement, not a final hardening phase.

Required controls:
- Authentication and session lifecycle.
- Server-side authorization for every query and command.
- Tenant/company context validation.
- Record and field-level access.
- CSRF/CORS/rate-limit controls.
- Secrets management and log redaction.
- Audit trails and data export tracking.
- Input validation, safe file handling and content type checks.
- Dependency/container scanning and SBOM output.
- Security headers and TLS termination guidance.
- Backup encryption, least-privilege runtime DB role and separate migration role.
- Threat modeling for authentication, tenancy, module loading, automation and integrations.

## 14. Testing and quality gates

### Rust
- Unit tests for pure domain invariants and state transitions.
- Integration tests against real PostgreSQL and SQLite adapters.
- Contract tests for API DTOs and OpenAPI generation.
- Property tests for money rounding, quantity conversion, domain query compilation and state machines.
- Migration tests from empty and previous supported schema versions.
- Tenant isolation and authorization matrix tests.
- Concurrency/idempotency tests for posting, reservation and job execution.

### Frontend
- Typecheck, lint and build.
- Component tests for design primitives and complex field widgets.
- Accessibility checks for keyboard, labels, focus, contrast and reduced motion.
- Visual regression for core shell and high-value workflows.
- E2E flows for login, company context, CRUD, approvals and key ERP transactions.

### Cross-system
- Generated SDK drift check.
- OpenAPI compatibility check.
- Docker Compose smoke test.
- Backup/restore rehearsal.
- Performance tests for list/search, posting, stock movement, imports and dashboards.
- Security test suite before release.

A green unit test suite alone does not establish production readiness.

## 15. Deployment and operations

### Reference production topology
- Reverse proxy / TLS ingress.
- Next.js web container.
- Rust API container.
- Rust worker container.
- PostgreSQL with durable volume and backup policy.
- Object storage for documents/attachments (local disk only for controlled single-host setups).
- Optional cache only when measured need exists; not a required dependency for first boot.

### SQLite deployment
Single-host profile with one application host and local durable disk. Do not put the SQLite database or WAL files on network filesystems or share them between app nodes.

### Operational capabilities
- `/health/live` and `/health/ready`.
- Structured JSON logs with request and correlation IDs.
- Tracing across API, DB and job execution.
- Metrics for request latency/error, job queue depth/age/failures, DB pool saturation and business processing lag.
- Graceful shutdown and worker lease recovery.
- Scheduled and on-demand backup.
- Restore command and documented recovery steps.
- Explicit migration command and version compatibility check.
- Rolling app updates only when schema compatibility permits.
- Config validation before startup.
- No default admin password in production; first-run bootstrap token or one-time setup flow.

## 16. Performance and reliability budgets

Budgets are targets to benchmark and refine, not unverified claims:
- No unbounded list endpoint; every collection is paginated and capped.
- Avoid N+1 relation loading in standard views.
- Use indexes that match actual filter/order patterns.
- Keep interactive reads bounded and stream long-running exports.
- Queue heavy imports, report generation, bulk notifications and large document tasks.
- Measure p50/p95/p99 for critical paths on documented hardware/data fixtures.
- Use cache only with explicit invalidation and tenant-aware keys.
- Prefer rebuildable read projections for dashboards; source-of-truth transactions remain authoritative.

## 17. Delivery roadmap

### Phase 0 — Governance and product contract
Narrative, blueprint, design rules, license decision, issue execution protocol, threat model and initial architecture constraints.

### Phase 1 — Monorepo foundation
Rust workspace, Next.js app, package manager, lint/typecheck/format, CI, local dev and Docker skeleton.

### Phase 2 — Kernel and persistence
IDs, errors, clock, request context, PostgreSQL/SQLite adapters, migrations, transaction boundaries and test support.

### Phase 3 — Identity and organization
Users, sessions, tenant/company/branch model, membership, policy primitives and audit foundations.

### Phase 4 — ORM and metadata engine
Model registry, fields, relations, query AST/compiler, lifecycle hooks, computed fields and schema migration planner.

### Phase 5 — Module runtime
Manifest, dependency graph, install/upgrade/disable lifecycle, module permissions, menus, views and module SDK.

### Phase 6 — UI foundation
Design tokens, component library, app shell, navigation, API SDK, form/list/detail patterns and responsive accessibility.

### Phase 7 — Essential ERP
Contacts, CRM, sales, procurement, inventory, accounting/invoicing/payments with complete end-to-end tests.

### Phase 8 — Operational ERP
HR, payroll localization, projects, timesheets, manufacturing, quality, maintenance, fleet and field service.

### Phase 9 — Business experience
Website, ecommerce, POS, documents, approvals, helpdesk, subscriptions, marketing and integration adapters.

### Phase 10 — Intelligence and automation
Workflow builder/runtime, durable jobs, reporting, import/export, webhooks, AI assistant/agents, governance and evaluation.

### Phase 11 — Production readiness
Security review, performance, observability, backup/restore, upgrade rehearsals, deployment guides, localization validation and release process.

Phases may overlap only when their dependencies are satisfied and module ownership is disjoint. An issue's dependency list is authoritative for execution order.

## 18. Release gates

No public production release until:
1. Core CI and reproducible builds are green.
2. Migrations are tested from clean and supported prior versions.
3. Tenant isolation and authorization tests pass.
4. Backup and restore have been rehearsed.
5. Critical business flows pass E2E tests.
6. No known critical/high exploitable vulnerability is unresolved.
7. Logging, metrics, health checks and failure alerts are present.
8. Setup, upgrade, rollback/recovery and operator docs are complete.
9. Supported database/runtime matrix is published.
10. License, notices and third-party dependency obligations are reviewed.

## 19. Reference documentation

- [Next.js App Router](https://nextjs.org/docs/app)
- [Next.js Server and Client Components](https://nextjs.org/docs/app/getting-started/server-and-client-components)
- [Next.js Backend-for-Frontend guidance](https://nextjs.org/docs/app/guides/backend-for-frontend) — URP uses Next.js as a frontend/BFF only where useful; Rust remains the business backend.
- [Axum documentation](https://docs.rs/axum/latest/axum/)
- [SQLx documentation](https://docs.rs/sqlx/latest/sqlx/)
- [SQLite Write-Ahead Logging](https://www.sqlite.org/wal.html)
- [PostgreSQL Row Security Policies](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
