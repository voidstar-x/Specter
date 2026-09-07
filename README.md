# Specter

**Specter** is a sovereign, local-first AI legal document assistant. It runs entirely on your machine — no cloud database, no external auth provider, no S3 bucket — and keeps your documents and embeddings on your own disk.

Specter is a **fork of [SemplificaAI/MikeRust](https://github.com/SemplificaAI/MikeRust)**, which is itself a Rust + Tauri port of the original open-source **Mike** legal assistant by [Will Chen](https://github.com/willchen96/mike). The lineage runs **Mike → MikeRust → Specter**. We built Specter on MikeRust's proven local-first architecture and repurposed it toward the jurisdiction and workflow we actually work in.

> Branding note: the code is AGPL-3.0; the "Specter" name and mark are our own. See [NOTICE.md](NOTICE.md) for the brand-vs-code separation.

---

## How Specter differs from MikeRust

MikeRust ships with a European/Italian orientation — Italian language defaults, EU (EUR-Lex, Fedlex, etc.) and world legal corpora, and Italian-informed (civil-law) workflow templates. **Specter is re-oriented to Singapore/common-law practice in the APAC region**, with an emphasis on **internet, technology, content and AI regulation**.

Concretely, the differences are:

| Area | MikeRust | Specter |
| --- | --- | --- |
| **Language** | Italian-first; multi-locale (en/it/fr/de/es/pt) | **English only** — English is the sole assistant/content language; non-English prompt and UI locales are removed |
| **Corpora** | EU/world legal sources (EUR-Lex, Fedlex, etc.) | **APAC official-government sources**: Singapore, Malaysia, Indonesia, South Korea, Vietnam, Thailand, Australia, Japan (`config/corpora-plugins/`) |
| **Workflows** | Italian/EU legal presets | **SG/common-law + jurisdiction-agnostic** presets, including a dedicated **statutory analysis & interpretation** workflow for internet/tech/content/AI regulation |
| **Scope of review** | Broad, civil-law-flavoured | Statutory interpretation, gap analysis, and risk-focused review for regulated tech/content/data areas |
| **Model** | Any provider | Pre-configured to a **local OpenAI-compatible inference engine** (e.g. a local DeepSeek cluster) via `.env`; other providers still selectable |
| **Binding** | Localhost by default | API bound to `0.0.0.0:3001` so the backend is reachable on your LAN |

Specter retains the upstream architecture described below, with the cleanup and APAC-specific changes recorded in [CHANGELOG](docs/CHANGELOG.md).

---

## Architecture (the same stack as MikeRust)

The technical foundation is unchanged from MikeRust and is **not** re-implemented here:

- **Backend:** a **Rust + axum** implementation, SQLite (via `sqlite-vec`) for storage and vector search, **local ONNX** embeddings (INT8-quantized multilingual-e5-base, with optional DirectML / QNN execution providers), and pure-Rust extraction for **PDF / DOCX / RTF / XLSX** — no LibreOffice process spawn.
- **Frontend:** a clean-room **Svelte 5 + Vite + Tailwind CSS v4** rewrite. No source code from the original Mike project remains in the tree.
- **Shell:** a **Tauri** desktop app with no server-side dependency.

### What you can do

- **Chat with citations** — the assistant answers and cites the underlying document; numbered citation pills (`[1]`, `[gN]`, `[pN]`) open the source in a side viewer (PDF.js text search highlights the quoted passage).
- **Projects & documents** — organise work into projects and attach documents for review.
- **Workflows & tabular reviews** — run structured review templates that extract and assess specific fields clause-by-clause.
- **DOCX templates** — create or import templates to generate editable `.docx` documents through the pure-Rust docx engine. Upstream Italian report templates are not bundled.
- **Corpora** — query official legal sources (our APAC set, see above) via the corpus plugin system; `config/corpora-plugins/*.json` are manifest-driven and extensible.

### Model providers

Select the active provider in **Settings → LLM models** — Anthropic, Google, OpenAI, Mistral, or a **local OpenAI-compatible endpoint** (the default for Specter). Only providers with a saved API key are selectable.

### Configuration

Configuration lives under `config/`:
- `config/corpora-plugins/` — the APAC official-government corpus manifests.
- `config/workflow-presets/` — the SG/common-law workflow presets.
- `config/system-prompts/en/` — the (English-only) assistant persona prompts per practice area.
- `config/model.json` — registered models, including the local engine entry.
- `.env` — runtime config: `PORT`, `VLLM_BASE_URL`, `VLLM_MAIN_MODEL`, `VLLM_API_KEY`, `JWT_SECRET`.

The API server binds to `0.0.0.0:3001` (LAN-reachable) when `PORT=3001` is set.

---

## Built in the open

Fixes, new corpus plugins, jurisdiction-specific feedback, design ideas and half-formed proposals are all welcome. Open an [issue](https://github.com/voidstar-x/Specter/issues) to discuss a direction, or send a pull request when you have something concrete.

---

## License

Specter is a fork of AGPL-3.0 projects and ships under **AGPL-3.0**. See [LICENSE](LICENSE).

## Thanks

- [SemplificaAI/MikeRust](https://github.com/SemplificaAI/MikeRust) — the Rust/Tauri port we forked.
- [willchen96/mike](https://github.com/willchen96/mike) — the original open-source Mike project.
