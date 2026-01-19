# SIGNIA Interface Service

Deterministic help layer for the SIGNIA repository.
It indexes docs/schemas/examples and answers common questions without relying on an LLM.

## Run
```bash
npm install
npm run dev
```

## Env
- `PORT` (default 7070)
- `SIGNIA_REPO_ROOT` path to monorepo root (default: process.cwd())
