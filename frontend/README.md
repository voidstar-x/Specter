# Specter frontend

Clean-room Svelte 5 rewrite of the Specter desktop UI.

- **Inception:** 2026-05-15
- **Stack:** Tauri 2 · Svelte 5 (runes) · TypeScript · Tailwind CSS v4 · Vite 6
- **License:** AGPL-3.0-only (see [LICENSE](LICENSE))
- **Plan:** [../docs/specter-ui-rewrite-plan.md](../docs/specter-ui-rewrite-plan.md) v2.1

## Status

Fase 0 scaffold. The app currently boots, discovers the axum backend port
via the Tauri `api_base_url` command, and renders `/healthz` as a smoke
test.

## Anti-contamination

This frontend is a **clean-room rewrite**. It does NOT derive from the
upstream "Mike" AGPL project. The legacy frontend (kept at
`../frontendMike/` as a working reference during migration) MUST NOT be
read while developing this one — only screenshots of the rendered UI
and Specter commit messages are admissible sources. See plan §21 for
the full anti-contamination rules.

## Develop

From the **repo root** (`c:\Progetti\Specter`):

```pwsh
# 1. Install dependencies (once)
pnpm --dir frontend install

# 2. Launch the Tauri shell against the NEW Svelte frontend
cargo tauri dev --config src-tauri/tauri.svelte.conf.json
```

The default `cargo tauri dev` (without `--config`) still launches the
legacy Next.js frontend from `../frontendMike/`. The two configurations
are intentionally parallel during migration; the legacy one and its
config will be deleted in Fase 8 once parity is reached.

## Scripts (`pnpm <script>` inside this directory)

| Script           | Action                                  |
|------------------|-----------------------------------------|
| `dev`            | Vite dev server on `127.0.0.1:5173`     |
| `build`          | Type-check + production build to `dist` |
| `preview`        | Serve `dist` for local preview          |
| `typecheck`      | `svelte-check` (CI)                     |
| `lint`           | ESLint on `src/`                        |
| `format`         | Prettier write                          |
| `test`           | Vitest unit suite                       |
| `test:watch`     | Vitest watch mode                       |
| `test:e2e`       | Playwright suite                        |
| `license-audit`  | Reject non-permissive transitive deps   |

## Layout

See plan §3 for the full directory tree. High-level:

```
frontend/
├── src/
│   ├── lib/
│   │   ├── api/        ← HTTP wrappers for /auth /chat /document …
│   │   ├── components/ ← UI primitives + feature components
│   │   ├── stores/     ← Svelte 5 runes state (one file per resource)
│   │   ├── tauri/      ← invoke wrappers (api_base_url, open_external_url)
│   │   ├── types/      ← TS mirrors of Rust serde structs
│   │   └── utils/      ← format, markdown, sse, download, …
│   ├── routes/         ← Boot / Setup / Unlock / Assistant / …
│   ├── App.svelte
│   ├── app.css
│   └── main.ts
├── locales/            ← i18n bundle (en canonical + it/fr/de/es/pt)
├── public/             ← static assets bundled as-is
└── tests/              ← unit (Vitest) + e2e (Playwright)
```

## Backend contract

The frontend is a pure HTTP client of the axum backend documented in
plan §6. There are only **two Tauri commands**: `api_base_url` (boot)
and `open_external_url` (system browser routing). Everything else is
fetched from `http://127.0.0.1:<discovered-port>`.
