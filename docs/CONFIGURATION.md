# URP — Configuration Contract

## Local setup

1. Copy `.env.example` to `.env`.
2. Keep the example values local only; never use them in shared or production environments.
3. Start PostgreSQL with `docker compose --env-file .env -f infra/compose/dev.yml up -d`.
4. Start the API with `cargo run -p urp-api`.
5. Start the web app with `pnpm --filter @urp/web dev`.

## Environment variables

| Variable | Consumer | Required | Default / example | Secret |
|---|---|---:|---|---|
| `URP_API_BIND` | API | No | `0.0.0.0:8080` | No |
| `URP_LOG` | API | No | `info` | No |
| `URP_DATABASE_URL` | Storage/API (storage task pending) | Yes when DB adapter is enabled | Local development URL in `.env.example` | **Yes** |
| `URP_STORAGE_PATH` | Document storage (implementation pending) | No for current skeleton | `./var/storage` | Path is private configuration |
| `NEXT_PUBLIC_API_BASE_URL` | Browser bundle | No in local dev | `http://127.0.0.1:8080/api/v1` | No; must not contain credentials |
| `POSTGRES_DB` | Dev Compose | Yes for Compose | `urp_dev` | No |
| `POSTGRES_USER` | Dev Compose | Yes for Compose | `urp_dev` | No |
| `POSTGRES_PASSWORD` | Dev Compose | Yes for Compose | Development-only placeholder | **Yes** |
| `POSTGRES_PORT` | Dev Compose | No | `5432` | No |
| `URP_WORKER_CONCURRENCY` | Worker (future) | No | `4` | No |

## Production rules

- Production startup must fail when required credentials or encryption/session keys are absent; no insecure default is allowed.
- Inject secrets through deployment secrets or an operator-managed secret store.
- Do not place database URLs, passwords, signing keys or provider credentials in `NEXT_PUBLIC_*` variables.
- Do not log environment dumps or raw connection strings.
- The API currently validates its bind address and log filter. Database, object-storage and worker validation become required when their runtime components are introduced.
- Do not enable unimplemented environment variables merely because they appear in the example file; reserved variables are documented for future tasks.

## Change policy

New variables must be added to this table and `.env.example` in the same change. A variable must declare consumer, required/default behavior, secret classification and production validation policy. Remove obsolete variables only through a dedicated change with migration guidance.
