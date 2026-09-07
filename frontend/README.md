# Specter frontend

English-only Svelte 5 frontend for Specter's local-first legal assistant.
Inherited from the SemplificaAI/MikeRust Svelte rewrite; see the repository
[credits](../README.md#thanks), [LICENSE](LICENSE) and [NOTICE](NOTICE).

## Develop

From the repository root on Windows:

```powershell
pnpm --dir frontend install
cargo tauri dev --config src-tauri/tauri.svelte.conf.json
```

Use the explicit Svelte configuration above. The legacy default
`src-tauri/tauri.conf.json` still targets the retired Next.js frontend and
is not the supported development entry point.

For frontend-only development, run `pnpm dev` in this directory against
a running axum backend. Tauri discovers its backend via `api_base_url`;
see `src/lib/tauri/` for the desktop command wrappers.

## Verification

Run in this directory:

```powershell
pnpm install
pnpm build
pnpm test
```

`build` runs Svelte/TypeScript checks followed by Vite's production build
to `dist/`. Other scripts: `dev`, `preview`, `typecheck`, `lint`, `format`,
`test:watch`, `test:e2e` (Playwright) and `license-audit`.

## Layout

- `src/lib/api/`: HTTP wrappers for the backend.
- `src/lib/components/`: shared controls and feature panels.
- `src/lib/stores/`: Svelte 5 reactive state.
- `src/lib/tauri/`: desktop IPC wrappers.
- `src/lib/types/`: TypeScript backend contracts.
- `src/routes/`: application screens.
- `locales/en.json`: the sole UI dictionary.
- `tests/`: end-to-end tests; unit tests also live alongside source.

APAC corpora use the generic `/corpora/{id}/*` APIs. Local folder indexing
uses `/sync/*`. See [CORPORA](../docs/CORPORA.md) and
[CORPUS_PLUGINS](../docs/CORPUS_PLUGINS.md) for supported connectors.
