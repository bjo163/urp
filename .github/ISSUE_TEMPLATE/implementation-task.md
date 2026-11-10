---
name: Implementation task
about: Small, autonomous, testable implementation work
title: "[Pxx] [TASK] "
labels: []
assignees: []
---

## Outcome
One sentence describing the observable result.

## Scope
- Included:
- Excluded:

## Specification
Describe exact behavior, data contract, state transitions, constraints and failure behavior.

## Design rules
- Follow `docs/ENGINEERING_STANDARDS.md`.
- Follow the relevant `docs/BLUEPRINT.md` section.
- Do not duplicate shared platform capabilities.
- Keep changes within this issue's ownership boundary.

## Files / ownership
- Expected paths:
- Shared files requiring coordination:

## Dependencies
- Parent epic:
- Blocked by:
- May run in parallel with:

## Acceptance criteria
- [ ] Observable behavior is implemented.
- [ ] Validation and failure behavior are covered.
- [ ] Authorization / tenant scope is enforced where applicable.
- [ ] Required tests pass.
- [ ] Docs/contracts are updated where required.
- [ ] No unrelated scope or duplicate abstraction was introduced.

## Test plan
List exact commands and scenarios, including failure/security cases.

## Definition of done
- PR links this issue.
- Acceptance checkboxes have evidence.
- CI is green.
- No unresolved blocker or unexplained TODO remains.
