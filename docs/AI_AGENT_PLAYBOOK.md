# URP — Autonomous AI Agent Playbook

This repository is designed for small local models and autonomous coding agents. Optimize work for narrow context, deterministic steps, explicit contracts, and verifiable outcomes.

## 1. Source-of-truth hierarchy

1. `docs/NARRATIVE.md` defines product intent and non-goals.
2. `docs/BLUEPRINT.md` defines architecture, domain ownership and delivery phases.
3. `docs/ENGINEERING_STANDARDS.md` defines global design and quality rules.
4. A GitHub issue defines the only implementation scope for one task.
5. Existing code and tests define current behavior, unless they conflict with approved documentation.

If a conflict is discovered, do not guess. Report the exact conflict and propose the smallest documentation correction. Implementation waits until the conflict is resolved.

## 2. Selecting work

- Pick only an open issue with all dependencies closed.
- Prefer the lowest-numbered ready issue in the earliest active phase unless a maintainer explicitly prioritizes another issue.
- Do not start an epic as if it were an implementation task. Epics organize work; child tasks contain executable scope.
- Do not select two tasks with overlapping file/domain ownership in the same agent run.
- Read the parent epic, this playbook, standards, and only the blueprint sections referenced by the task.
- Inspect the existing code before editing. Never assume a planned directory or API already exists.
- If the task's prerequisites are missing, stop and report the blocker instead of building a duplicate foundation.

## 3. Required execution loop

1. **Understand:** restate the issue outcome, scope, exclusions, dependencies and acceptance criteria.
2. **Inspect:** read relevant files, tests, manifests and nearby module conventions.
3. **Plan:** write a short task plan with expected files and validation commands.
4. **Implement:** make the smallest complete change within the issue scope.
5. **Test:** run targeted tests first, then relevant workspace checks.
6. **Review:** inspect the diff for security, scope creep, duplication, migration impact and generated-file drift.
7. **Document:** update docs only when the issue requests it or the implementation changes a documented contract.
8. **Report:** summarize changes, tests and any unresolved acceptance criteria.
9. **Close:** close only after all issue checkboxes have evidence and the PR is merged according to repository policy.

## 4. Issue completion contract

An issue is complete only when:
- Every acceptance checkbox is satisfied.
- Required tests exist and pass.
- Format/lint/typecheck/build checks relevant to the scope pass.
- No unrelated files or behavior were changed.
- Public contracts and docs are updated where required.
- Security and tenant context are preserved.
- The PR links the issue using `Closes #N` when complete.

If blocked, leave the issue open and provide:
- The exact blocker.
- The missing dependency or conflicting contract.
- Evidence (file path, test output, failing command).
- The smallest suggested follow-up issue.

Never mark a task complete because code compiles if its acceptance criteria describe runtime behavior.

## 5. Atomicity and anti-overlap

- One task = one bounded deliverable, ideally one module boundary or one cohesive layer.
- Do not expand scope to adjacent modules.
- Do not recreate shared infrastructure inside a feature module.
- Do not duplicate DTOs, API clients, permissions, design tokens, IDs, errors, audit, tenant resolution or migration systems.
- Before adding a new abstraction, search the repository for an existing equivalent.
- A module may consume another module only through the public contract named in the blueprint.
- If one issue appears to require work owned by another issue, stop and link the dependency; do not implement both.
- When a shared file is concurrently owned, coordinate rather than overwriting another agent's change.

## 6. Rules for small models

- Keep prompts task-specific; do not paste the entire blueprint into every run.
- Use the issue's "Relevant blueprint sections" and "Files in scope" as context boundaries.
- Avoid broad refactors, speculative abstractions and style-only rewrites.
- Prefer simple explicit code over clever generic code.
- Implement only one vertical slice per issue.
- Use existing commands and fixtures; do not invent tooling.
- Ask for human review when an issue touches authentication cryptography, tenant isolation, financial posting, stock valuation, destructive migrations, payment handling, or production secrets.
- Never fabricate test results, benchmark numbers or compliance claims.

## 7. Database safety

- Never edit an applied migration; add a new migration.
- Never run destructive commands against production.
- Use local/test databases only.
- Do not make schema changes outside the migration workflow.
- Do not weaken tenant predicates or authorization to make tests pass.
- Verify PostgreSQL and SQLite behavior when issue scope claims both.
- For destructive data changes, require the explicit migration/back-up issue and human approval.

## 8. API and frontend safety

- The Rust API owns business rules.
- Next.js must not access database credentials or bypass Rust authorization.
- Generate API clients from canonical OpenAPI; do not hand-maintain duplicate types.
- Keep Server/Client Component boundaries deliberate.
- UI controls reflect permissions but never replace server authorization.
- No raw HTML, SQL, script or arbitrary expression execution from metadata.

## 9. Agent output format

Every implementation report must include:

```text
ISSUE: #<number> — <title>
RESULT: DONE | BLOCKED | PARTIAL
SCOPE:
- <what changed>
FILES:
- <path>: <purpose>
VALIDATION:
- [command] — PASS/FAIL
ACCEPTANCE:
- [x] <criterion> — <evidence>
- [ ] <criterion> — <blocker>
RISKS:
- <migration/security/compatibility notes or none>
FOLLOW-UP:
- <issue link or none>
```

## 10. Parallel execution

Parallelize only tasks whose file ownership and dependencies are disjoint. Shared foundation contracts, manifests, root workspace files, API schemas and design tokens are integration hotspots; serialize their changes or assign a single integrator. Merge small PRs frequently. Do not let multiple agents invent different versions of the same shared contract.

## 11. Stop conditions

Stop implementation and report a blocker when:
- The issue conflicts with the current blueprint or standards.
- A required dependency is open or missing.
- The existing code has an unresolved security flaw in the path being changed.
- Acceptance criteria cannot be tested with available fixtures/tools.
- The change would require a destructive migration not included in scope.
- The task would expand beyond one bounded issue.
