# URP — Product Narrative

## The idea

URP is a **Universal Radical Platform**: a modern business operating system that unifies the day-to-day work of an organization in one coherent, extensible ERP.

It takes inspiration from the breadth, modularity, and model-driven workflows associated with Odoo, then rebuilds the foundation as an independent product: Rust for dependable business execution, Next.js for a fast and expressive workspace, PostgreSQL for serious multi-user operations, SQLite for constrained single-host deployments, and a first-party ERP engine designed around explicit contracts.

URP is not a port of Odoo. It is not an attempt to reproduce every legacy screen. It is a new platform that aims to make complex business operations feel understandable, connected, composable, and auditable.

## The promise

URP should let an organization:

1. Start with a small set of operational apps and add capabilities without replacing the platform.
2. Share one consistent identity, company structure, permission model, data vocabulary, and audit trail.
3. Turn business objects into connected workflows rather than isolated CRUD screens.
4. Adapt forms, fields, views, approvals, and automations through governed metadata.
5. Keep financial, inventory, and other high-integrity operations explicit, typed, transactional, and testable.
6. Run on infrastructure the organization controls, with understandable backup, restore, upgrade, and deployment procedures.
7. Use AI as a controlled assistant and operator, never as an unaccountable authority over business data.

## Product identity

**Category:** Modular ERP / business operations platform.

**Working product name:** URP.

**Technical identity:** Rust + Next.js, clean-room and Odoo-inspired.

**Experience identity:** Radical Enterprise — expressive, high-contrast, industrial, grunge/neobrutalist, and unmistakably modern, while remaining calm enough for long work sessions and precise business decisions.

**Delivery identity:** Small, verifiable capabilities. Every increment has a defined contract, tests, observable behavior, and a clear completion checklist.

## Who it serves

- Small and mid-sized organizations that want an integrated operational system without a heavyweight implementation process.
- Multi-company organizations that need shared governance and separated legal/operational contexts.
- Operators in sales, purchasing, stock, finance, people operations, projects, production, service, and management.
- Implementation partners and developers who extend the platform through documented modules and contracts.
- Self-hosters who need clear control over their data and deployment lifecycle.

The first release is not assumed to satisfy every industry, country, or regulatory regime. Local requirements are delivered as explicit localization modules and validated with qualified domain review.

## Product principles

### 1. One coherent platform, many focused apps
Modules share platform identity and contracts. They must not create parallel user systems, permission engines, or incompatible UI conventions.

### 2. Transactions before decoration
A polished workflow is not complete until its business invariants, authorization, failure behavior, audit events, and tests are correct.

### 3. Typed core, flexible edges
Critical first-party domains use typed Rust models and explicit relational schemas. Metadata provides extensibility for custom models, fields, views, and workflows without turning every business rule into untyped configuration.

### 4. Explicit beats magical
Dependencies, permissions, company context, lifecycle transitions, side effects, and failure states must be visible in code and documentation. Avoid hidden global state and implicit cross-module writes.

### 5. Modularity without microservice theatre
Begin as a modular monolith with enforceable boundaries. Split a component into a separate service only after measurable scaling, isolation, or operational needs justify it.

### 6. Radical visual language, disciplined usability
The interface may be bold, playful, industrial, and expressive. It must still be readable, keyboard accessible, responsive, predictable, and efficient under dense real-world data.

### 7. Auditable automation
Every privileged or consequential operation needs identity, authorization, a traceable reason, idempotency where appropriate, and an audit trail.

### 8. Self-hosting is a product feature
Install, configure, upgrade, back up, restore, and recover must be first-class workflows—not undocumented operator knowledge.

### 9. AI is governed, not omnipotent
AI agents use the same APIs, permissions, validation, approvals, and audit records as human users. High-risk actions require policy checks and, when configured, human approval.

### 10. Deliver in small, complete slices
A feature is not complete because its screen exists. It is complete when the API, domain behavior, security, UX states, persistence, tests, and documentation meet the issue's acceptance criteria.

## What “Odoo-inspired” means

URP adopts broad product concepts that are useful in modern ERP systems:

- A registry of business models and fields.
- Modules with declared dependencies and installation lifecycle.
- Relational fields and reusable business records.
- Multiple views over the same model.
- Declarative domains, filters, and actions.
- Access control at model, action, field, and record level.
- Extensible workflows, automation, and reporting.
- Connected apps sharing the same platform context.

These are product inspirations, not compatibility claims.

URP does **not** promise compatibility with Odoo's internal ORM, Python modules, XML data files, RPC contracts, database schema, binary addons, or upgrade mechanisms. Migration/import adapters may be built as separate, explicitly scoped tools. Any interoperability must be based on documented external interfaces and customer-owned data.

## The experience

URP should feel like an operational cockpit rather than a collection of old-fashioned admin pages.

- A consistent workspace shell with contextual navigation, global search, app launcher, activity inbox, and user/company context.
- Dense but legible list and detail workspaces.
- Fast keyboard and command interactions where they improve real tasks.
- Clear status, ownership, history, and next action on business records.
- Strong empty, loading, validation, permission-denied, offline, and error states.
- Dark and light themes with the same semantic contrast and accessibility.
- A distinctive visual system built from reusable tokens and components, not one-off decoration.

Visual character must never obscure data meaning. Color cannot be the only way to communicate status. Motion must respect reduced-motion preferences. Forms must be accessible and usable without a pointer.

## Product boundaries

URP's platform owns shared capabilities: identity, tenants, companies, permissions, module registry, metadata, API contracts, eventing, audit, navigation, localization foundations, and application shell.

Business modules own their business rules and data: CRM owns pipeline semantics; Sales owns quotations and orders; Inventory owns stock movements and reservations; Accounting owns journals and ledgers; HR owns employee lifecycle; Manufacturing owns production orders and bills of materials.

A module may consume another module's public contract, but must not reach into its private tables or bypass its domain commands.

## Definition of success

URP succeeds when a team can install it, configure an organization, enable a focused set of apps, complete an end-to-end business workflow, understand every permission and side effect, recover from operational failure, and safely extend the platform—without needing to understand its entire source tree.

The long-term ambition is broad ERP coverage. The delivery promise is incremental: no capability is declared complete until its own evidence and acceptance criteria exist.
