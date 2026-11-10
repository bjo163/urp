# URP Support Matrix

**Baseline date:** 2026-11-11. This matrix distinguishes upstream-supported versions from combinations already exercised by URP CI. A selected production target is not considered URP-validated until its integration lane passes.

## Runtime matrix

| Component | Selected baseline | URP validation status |
|---|---|---|
| Rust | 1.99.1 stable | Pinned in `rust-toolchain.toml`; Rust CI passed on the foundation branch |
| Axum | 0.8.9 | Pinned in Cargo workspace dependencies |
| SQLx | 0.9.0 | PostgreSQL and bundled SQLite drivers enabled; adapter tests are part of foundation/P02 |
| Node.js | 26.x Active LTS | Selected primary runtime; web CI is being moved to Node 26 |
| Node.js compatibility | 24.x Maintenance LTS | Secondary compatibility target; do not make new production installs depend on a superseded line |
| Next.js | 16.4.6 App Router | Resolved by the committed web dependency manifest; web CI has passed on Node 24 |
| React | 19.3.1 | Paired with the selected Next.js release |
| TypeScript | 7.0.2 | Strict mode |
| pnpm | 12.17.0 | Pinned in root `package.json` and CI |

Node.js 26 entered Active LTS on 2026-10-28; Node.js 24 moved to Maintenance LTS on 2026-10-20. URP selects 26 as its primary runtime and retains 24 only as a compatibility target. See the [Node.js release schedule](https://github.com/nodejs/Release#release-schedule) and [Node.js release status](https://nodejs.org/en/about/previous-releases).

Next.js 16.4.6 is the pinned App Router release in the current foundation dependency set. See the [Next.js 16.4 release notes](https://nextjs.org/blog/next-16-4) and [installation requirements](https://nextjs.org/docs/app/getting-started/installation).

## Database matrix

| Database | Selected role | Validation status |
|---|---|---|
| PostgreSQL 18 | Production default target for multi-user and multi-node deployments | Storage connection adapter is validated by P01 CI; migration/upgrade conformance is completed in P02 |
| PostgreSQL 17 | Compatibility target | P02 migration workflow uses a dedicated test database on PostgreSQL 17 |
| SQLite (SQLx bundled driver) | Embedded / single-host deployment only | P02 migration and transaction tests cover the file-backed adapter |

### PostgreSQL

PostgreSQL is the required database for multi-node, HA, high-write and multi-user production workloads. The P01 local Compose profile and current P02 migration CI use PostgreSQL 17 so foundation checks remain reproducible. Before claiming PostgreSQL 18 as URP-validated, the persistence/migration conformance suite must also run on PostgreSQL 18.

PostgreSQL 18 and 17 are supported upstream release lines. PostgreSQL 19 remains a release candidate on the baseline date and is excluded from production support. See the [PostgreSQL versioning policy](https://www.postgresql.org/support/versioning/) and [PostgreSQL 19 RC announcement](https://www.postgresql.org/about/news/postgresql-19-rc-1-released-3409/).

### SQLite

SQLite is supported for embedded/local and single-host deployments only.

- Enable foreign keys and WAL journaling with a bounded busy timeout.
- Keep the database, WAL and SHM files on local durable storage.
- Never share a WAL database over a network filesystem or between application hosts.
- PostgreSQL is mandatory for HA and multi-node deployments.
- Backend capability differences must be explicit; never silently claim SQLite has PostgreSQL semantics.

## Operating systems and browsers

| Target | Status |
|---|---|
| Linux x86_64 | Primary CI and container target |
| Linux arm64/aarch64 | Compatibility target; requires its own build/runtime matrix |
| macOS | Developer workstation |
| Windows | Developer workstation via WSL2; native Windows production deployment is not targeted |
| Browsers | Current stable Chrome, Edge, Firefox and Safari; accessibility and keyboard tests are required |

## Update policy

1. Review Rust, Node.js, Next.js, PostgreSQL, SQLite and package-manager updates monthly.
2. Pin toolchain and dependency versions; commit lockfiles produced by CI and reviewed with the dependency manifests.
3. Upgrade one major runtime/database line at a time.
4. Run formatting, lint, typecheck, tests, builds, persistence conformance and migration tests after an update.
5. Do not mark a compatibility target supported until its relevant CI lane is green.
6. Record exceptions with owner, rationale and expiry.

## Required CI lanes

- Rust 1.99.1 on Linux x86_64.
- Node.js 26.x + pnpm 12.17.0 + Next.js 16.4.6.
- PostgreSQL 17 migration/test database while P02 storage tests are being established.
- PostgreSQL 18 conformance before claiming production database validation.
- SQLite bundled/file-backed migration and transaction tests.
