# URP — Engineering Standards

These rules apply to every implementation issue and pull request. A task-specific issue may add constraints but may not silently weaken these standards.

## 1. Architecture and scope

- Preserve the modular-monolith boundary. Do not introduce microservices, brokers, caches, or infrastructure without a measured requirement and approved issue.
- Keep business rules in Rust domain/application code, not Axum handlers or Next.js components.
- Keep UI state and presentation in Next.js; all authoritative business state changes pass through Rust APIs.
- Use module public contracts for cross-module behavior. Never import another module's private repository or query its private tables.
- One issue owns one bounded outcome. Do not opportunistically implement adjacent features.
- If implementation requires a blueprint change, stop and create/update a documentation decision before proceeding.

## 2. Rust standards

- Use the repository toolchain and pinned dependency policy.
- Prefer explicit types, typed IDs, enums for finite states, and domain-specific errors.
- Avoid `unwrap()`/`expect()` in production paths; explain any unavoidable invariant assertion.
- No `unsafe` without an explicit approved issue and safety rationale.
- Do not use floating-point arithmetic for money, tax, stock quantity or other exact business values.
- Use parameterized SQL only. Never interpolate user data, field names or sort expressions into SQL.
- Keep transactions explicit in application use cases.
- Do not hide authorization inside optional convenience methods.
- Avoid unnecessary generic frameworks, premature abstraction and global mutable state.
- Public types/functions require meaningful docs when their contract is not self-evident.
- Errors must preserve internal diagnostics without leaking secrets or PII to clients.

## 3. Database and migration rules

- Every schema change is versioned, namespaced and tested on PostgreSQL and SQLite where the feature claims support.
- Migrations are explicit deployment steps; application startup does not mutate production schema.
- Add foreign keys, uniqueness, check constraints and indexes where the invariant belongs.
- Destructive migrations require a staged plan, data-impact explanation, backup/restore evidence and explicit acceptance.
- Use UTC instants, explicit timezone IDs, exact decimal/money representation and declared rounding rules.
- Tenant-owned tables include tenant scope and query paths require authenticated tenant context.
- Avoid database-specific features in shared code unless guarded by a documented capability contract.
- Verify pooled-connection session context cannot leak between tenants.

## 4. ORM and metadata rules

- Metadata is untrusted input and must be schema validated before registration.
- Technical model/field names are stable, namespaced and validated against a strict grammar.
- Domain filters compile from a typed AST; never accept raw SQL or arbitrary expression strings.
- Query compilation binds all values as parameters and allowlists identifiers/operators.
- Computed fields must have declared dependencies and deterministic evaluation.
- Detect inheritance and computed-field dependency cycles before activating a registry.
- Schema plans must be previewable and versioned; no silent data loss on field removal or type changes.
- Dynamic metadata must not bypass typed accounting/inventory invariants.
- Metadata read/write APIs require administrative permissions and audit events.

## 5. API and contracts

- All public API routes are versioned under `/api/v1`.
- Request/response DTOs are explicit; never serialize persistence structs directly.
- Validate all request bodies, query filters, pagination, sorting, file names and content types.
- Use stable machine-readable error codes, field paths and request IDs.
- Generate the TypeScript client from the canonical OpenAPI output.
- Breaking contract changes require a versioning/deprecation decision and contract test.
- Use idempotency keys for retry-prone business commands and inbound webhooks.
- Do not use Next.js Server Actions as the only public API contract.

## 6. Security and privacy

- Treat tenant/company IDs from clients as requested context, not proof of access.
- Authorize every command, query, aggregate, export, file read and background job.
- UI hiding is not security.
- Apply model, action, field and record rules consistently.
- Use secure HttpOnly cookies, CSRF protection for cookie-authenticated mutations, restrictive CORS and safe headers.
- Never log passwords, tokens, secrets, session cookies, payment data or unnecessary personal data.
- All secrets are injected from deployment configuration or a secret manager.
- Sensitive operations produce audit records.
- File uploads require size/type validation, generated storage keys and access checks on every download.
- Add security tests for every new privileged endpoint or tenant-scoped data path.

## 7. Frontend and design rules

- Use Server Components by default; isolate interactive behavior in Client Components.
- Frontend uses the generated API client; no direct DB access.
- Reuse design tokens and `packages/ui`; no one-off visual systems per module.
- No hard-coded color/spacing/type values in feature components when a token exists.
- Radical visual identity must never reduce readability or accessible interaction.
- Support dark and light themes, keyboard navigation, visible focus, semantic markup and reduced motion.
- Color is never the only status indicator.
- Design loading, empty, error, validation, denied, success and offline states.
- Large collections require search/filter/pagination and bounded rendering.
- Forms prevent accidental loss of unsaved changes and show actionable validation errors.
- Every icon-only action has an accessible name and tooltip where appropriate.

## 8. Testing rules

Each issue identifies tests appropriate to its scope. At minimum:
- Domain logic: unit tests for success, boundary and failure cases.
- Persistence: integration tests against the claimed database engines.
- API: handler/contract tests including denied and invalid requests.
- Security: cross-tenant and unauthorized access tests when applicable.
- UI: component/accessibility tests and E2E for critical workflows.
- Migrations: empty database and prior-version upgrade tests.
- Async jobs/events: retry, dedupe, idempotency and terminal failure tests.

Do not mock away the behavior the issue is supposed to verify. Use deterministic clocks, IDs and fixtures when appropriate.

## 9. Quality and maintainability

- Run format, lint, typecheck, test and build commands relevant to changed areas.
- Keep functions and modules cohesive; avoid massive files and catch-all modules.
- No dead code, unexplained TODOs, placeholder success paths, fake persistence or silent error swallowing.
- Add structured tracing at important boundaries without logging sensitive payloads.
- Update docs, examples and generated contracts when their source changes.
- Keep the dependency surface small; every new dependency needs a reason and license review.
- Do not add a library merely to avoid a few simple lines of well-tested code.

## 10. Radical enterprise design language

- Visual signature: industrial editorial typography, bold hierarchy, structured grids, controlled contrast, intentional borders and selective high-energy accents.
- Work surfaces must be calm enough for repetitive operational use.
- Prioritize table density, data alignment, filters, keyboard access, clear actions and contextual record history.
- Avoid noisy textures behind dense data, low-contrast grunge, excessive animation and decorative HUD elements that obscure information.
- Do not use text as an image for essential content.
- Visual regression fixtures must include both themes and at least one narrow viewport for shared components.

## 11. Pull request and completion rules

A PR must:
- Link exactly the issue(s) it implements.
- Stay within each issue's scope.
- Include a concise summary and implementation notes.
- Record test commands and results.
- Include screenshots/video for visible UI changes where useful.
- Identify migrations, config/env changes, security implications and rollback concerns.
- Update relevant docs.
- Have all issue acceptance criteria checked with evidence.

Close an issue only when its checkboxes are demonstrably complete. If a criterion is not applicable, explain why in the issue/PR rather than silently omitting it.
