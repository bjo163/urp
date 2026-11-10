# Contributing to URP

## Before you start

Read the issue's parent epic, the applicable sections of:

- `docs/NARRATIVE.md`
- `docs/BLUEPRINT.md`
- `docs/ENGINEERING_STANDARDS.md`
- `docs/AI_AGENT_PLAYBOOK.md`

Choose one unblocked GitHub task. Do not implement an epic as one coding task.

## Development rules

- Rust owns business invariants, authorization and persistence.
- Next.js is a typed API client and presentation layer; never connect it directly to the database.
- Keep modules inside their declared public contracts.
- Do not use floating-point values for money or exact quantities.
- Do not add arbitrary SQL or executable metadata expressions.
- Do not introduce a second implementation of an existing shared primitive.
- Keep pull requests limited to their linked issue scope.

## Pull requests

Every pull request must link the implemented issue, list validation commands/results, identify schema/security impact, and provide screenshots for meaningful UI changes. Close tasks only when all issue acceptance criteria have evidence.
