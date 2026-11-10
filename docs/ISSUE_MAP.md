# URP — GitHub Issue Map

This file maps the product blueprint to its executable GitHub backlog. Each issue contains its own scope, specification, design rules, dependencies, acceptance checklist and validation plan.

**Current baseline:** 14 phase epics, 186 open implementation tasks, and 1 closed duplicate issue (201 total issues: 200 open, 1 closed). The repository is greenfield; the tasks are planned work, not completed functionality.

## Phase index

| Phase | Epic | Tasks | Count | Scope |
|---|---:|---|---:|---|
| P00 | [#1 Governance](https://github.com/bjo163/urp/issues/1) | [#2–#4](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP00%5D) | 3 | License, threat model, supported matrix |
| P01 | [#5 Workspace](https://github.com/bjo163/urp/issues/5) | [#6–#13](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP01%5D) | 8 | Rust/Next.js workspace, CI, local development |
| P02 | [#14 Kernel/API](https://github.com/bjo163/urp/issues/14) | [#15–#20, #22–#28, #74](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP02%5D) | 14 | IDs, errors, context, storage, migrations, API, OpenAPI, SDK, outbox |
| P03 | [#29 Identity/Security](https://github.com/bjo163/urp/issues/29) | [#30–#38, #154](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP03%5D) | 10 | Users, sessions, tenant/org, policies, audit, service credentials |
| P04 | [#39 ORM](https://github.com/bjo163/urp/issues/39) | [#40–#52](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP04%5D) | 13 | Metadata, registry, relations, query AST/compilers, CRUD, lifecycle, compute, schema plans, policy integration |
| P05 | [#53 Module Runtime](https://github.com/bjo163/urp/issues/53) | [#54–#62](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP05%5D) | 9 | Manifest, dependencies, install/upgrade/disable, registrations, SDK |
| P06 | [#63 Radical UI](https://github.com/bjo163/urp/issues/63) | [#64–#73, #177–#188](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP06%5D) | 22 | Tokens, components, shell, API client, tables, forms, details, accessibility |
| P07 | [#75 Contacts/CRM/Sales](https://github.com/bjo163/urp/issues/75) | [#76–#84, #219–#220](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP07%5D) | 11 | Contacts, CRM, catalog/pricing, quotation/order, confirmation event |
| P08 | [#85 Procurement/Inventory](https://github.com/bjo163/urp/issues/85) | [#86–#97, #189, #221–#224](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP08%5D) | 16 | Vendors, RFQ/PO, warehouse, movement ledger, reservation, receipts, delivery, traceability, costing, barcode |
| P09 | [#98 Accounting](https://github.com/bjo163/urp/issues/98) | [#99–#111, #190–#192, #225](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP09%5D) | 17 | Money/currency, chart, journal, posting, AR/AP, payments, tax, reports, valuation bridge, assets |
| P10 | [#112 HR/Operations](https://github.com/bjo163/urp/issues/112) | [#113–#127, #193–#194, #226](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP10%5D) | 18 | HR, recruitment, attendance, leave, expenses, projects, timesheets, MRP, quality, maintenance, fleet, payroll, field service, planning |
| P11 | [#128 Customer Experience](https://github.com/bjo163/urp/issues/128) | [#129–#141, #195–#196](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP11%5D) | 15 | Documents, approvals, helpdesk, CMS, ecommerce, payments, POS, subscriptions, marketing, knowledge, signatures, portal |
| P12 | [#142 Automation/AI](https://github.com/bjo163/urp/issues/142) | [#143–#153, #155–#160, #197–#198](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP12%5D) | 19 | Jobs, worker, outbox, workflows, notifications, webhooks, import/export, search, reports, dashboards, AI, scheduled reports |
| P13 | [#161 Production Readiness](https://github.com/bjo163/urp/issues/161) | [#162–#176, #199–#201](https://github.com/bjo163/urp/issues?q=is%3Aissue+is%3Aopen+%5BP13%5D) | 18 | Compose, backup/restore, observability, secrets, upgrades, SBOM, performance, security review, E2E, localization, release and runbooks |

## Closed duplicate

- [#21](https://github.com/bjo163/urp/issues/21) was closed as a duplicate of [#20](https://github.com/bjo163/urp/issues/20). Keep #20 as the single PostgreSQL migration runner task.

## Execution order

1. Complete P00 governance decisions and P01 reproducible developer workspace.
2. Complete P02 kernel/storage/API contracts.
3. Complete P03 identity, tenant context and authorization.
4. Complete P04 ORM and P05 module runtime.
5. Build P06 shared UI primitives and shell.
6. Deliver P07 Contacts/CRM/Sales and P08 Purchase/Inventory as connected vertical slices. Parallel work is allowed only where issue dependencies and file ownership are disjoint.
7. Deliver P09 Accounting against the already-established sales, purchasing and stock contracts.
8. Deliver P10 operations and P11 customer experience according to their explicit dependency links.
9. Deliver P12 automation/reporting/AI after outbox, jobs, policies and approval contracts.
10. Complete P13 release gates before calling any deployment production-ready.

## Agent rules

- Pick a task only when all "Blocked by" issues are closed.
- Read the parent epic, the issue, and only the relevant blueprint/standard sections.
- Implement one issue per PR unless a maintainer explicitly groups strictly related tasks.
- Do not treat epic issues as implementation tasks.
- Do not duplicate shared infrastructure or expand a task to adjacent modules.
- Close only when every acceptance checkbox has evidence and tests pass.
- If a blueprint or dependency is wrong, stop and request a focused documentation/backlog correction before coding.

## Counting convention

Counts include open task issues only. Phase epics are not counted as tasks. The active backlog contains 186 task issues plus 14 phase epics (200 open issues). Closed issue #21 is excluded from the active task count. P06 and later phases include supplemental UI/domain tasks with non-contiguous issue numbers. Issue #21 is a closed duplicate; issue numbers are immutable and gaps are intentional.
