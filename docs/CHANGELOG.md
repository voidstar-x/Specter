# Specter — Change Log & Fork Notes

Original upstream: **MikeRust** (github.com/SemplificaAI/MikeRust, AGPL-3.0). This repository is the **Specter** fork (github.com/voidstar-x/Specter).
This repository is the **Specter** fork maintained by Han (SG-based commercial /
corporate lawyer, APAC + Australia focus).

This file records the substantive changes made on top of upstream and how they
are configured. It is the canonical reference for what Specter does differently.

---

## 1. Default content/assistant language: ENGLISH (replaces Italian)

**Problem:** Specter answered in Italian because the assistant/content language
defaulted to `"it"` regardless of the UI locale, loading
`config/system-prompts/it/*.md` (Italian) for the per-domain system prompt.

**Setting that controls it:** the assistant/content language is driven by
`user_settings.locale` (migration `0005_user_locale.sql`). It is read in
`src/routes/chat.rs` at two points (the HyDE path and the domain prologue) and
resolved through `crate::presets::system_prompt::resolve()` / `assemble_prologue()`
in `src/presets/system_prompt.rs`, which walks a locale fall-back chain.

**What we changed:**
- `src/routes/chat.rs` — the locale default in both locations is now `"en"`
  (`unwrap_or("it")` → `unwrap_or("en")`), so a user with no stored preference
  gets English.
- `src/presets/system_prompt.rs` — `FALLBACK_LOCALES` reordered from
  `["it", "en"]` to `["en", "it"]`, so English is the first fall-back locale
  and no non-English content leaks for unknown locales. Module docs updated to
  state `en` is canonical.
- `config/system-prompts/` — **`en/` is now the sole canonical prompt set.**
  The `it/`, `fr/`, `de/`, `es/`, `pt/` directories were removed so English is
  always used (their removal is what makes the English guarantee hold even if a
  stored `user_settings.locale` is `it`). The `en/*` bodies already instruct
  "Default working language: English" and "Default jurisdiction: Singapore /
  ask which applies".
- Hard-coded Italian strings that surfaced to users were translated to English:
  - `src/llm/ollama_manager.rs` — local-model GEMMA system prompt &
    "Secure local mode" no-think preamble.
  - `src/llm/local.rs` — "Secure local mode active: …" error messages & tests.
  - `src/routes/documents.rs` — document-summary system prompt ("You are an
    assistant that produces technical and concise summaries…") and the
    "Summary (max 700 characters)" prompt.
  - `src/routes/corpora.rs` & `src/routes/eurlex.rs` — "Indexing completed but
    N chunks created…" import status messages.
  - `src/corpora/dila_bulk.rs` — "Local indexing…" progress label.
  - `src/llm/builtin_tools.rs` — test fixture strings ("You are an
    assistant…", "You are a lawyer…").

## 2. APAC / official-government corpus connectors

- Added a **DirectPdf** fetch shape + **gzip** adapter support in
  `src/corpora/manifest_adapter.rs`, `src/corpora/plugin.rs` so corpus manifests
  can declare PDF-direct and gzip-compressed fetches (needed for the APAC
  official-gov sources, several of which serve raw PDFs or gzip).
- Added/updated the following **APAC official-government** corpus manifests in
  `config/corpora-plugins/`:
  `sg-statutes` (Singapore), `my-lom` (Malaysia), `id-peraturan` (Indonesia),
  `kr-lawinfo` (Korea), `vn-legal` (Vietnam), `th-royalgazette` (Thailand),
  `au-federalregister` (Australia), `jp-egov` (Japan) — all 8 APAC sources.
- **Removed** the EU/world corpus plugins that are not relevant to the
  APAC/common-law focus (eurlex, eu-curia, it-normattiva, fr-legifrance,
  de-gesetze, uk-legislation, us-courtlistener, etc.) to keep the catalogue
  minimal. The builtin EUR-Lex / Italian-Legal routes remain in code.

## 3. SG / common-law workflow presets

- Added new **tabular review** workflow presets for Singapore and common-law
  practice in `config/workflow-presets/legal/`:
  - `statutory-analysis.json` (jurisdiction-agnostic statutory analysis)
  - `statutory-analysis-sg.json` (Singapore statutory analysis)
  - `statutory-analysis-au.json` (Australia (Cth) statutory analysis)
  - `nda-review-sg.json` (NDA review, SG & common law)
  - `saas-supply-review-sg.json` (SaaS / supply & services review, SG)
  - `gdpr-pdpa-compliance.json` (data-protection gap analysis, SG PDPA 2012)
  - `content-online-safety.json` (content / online-safety / AI regulation, SG
    with AU comparators)
- These are English-language, wide-column tabular reviews covering jurisdiction
  identification, instruments, obligations, exemptions, enforcement, penalties
  and SG-common-law case law where relevant.

## 4. LLM model catalogue

- `config/model.json` — added a **DeepSeek V4 Flash Vision Exp** entry to the
  `local` (OpenAI-compatible) provider's model list, matching the local engine
  configured via `VLLM_MAIN_MODEL=deepseek-v4-flash-vision-exp`. This is the
  local inference engine used for the auxiliary/side-job work.

## 5. Serving / environment

- `src/lib.rs` — the API server now binds **`0.0.0.0`** instead of loopback so
  it is reachable on the LAN (`.env` `PORT=3001`).
- `.env` (git-ignored; see `.env.example` for the documented keys) configures:
  `PORT=3001`, `VLLM_BASE_URL`, `VLLM_API_KEY`, `VLLM_MAIN_MODEL` (the local
  DeepSeek-v4 vit language model), and `JWT_SECRET`.

## 6. Example & tooling

- `examples/apac_probe.rs` — a probe binary used to exercise the APAC corpus
  adapters / DirectPdf fetch shape against the live sources.
- `frontend/.npmrc` & `frontend/pnpm-workspace.yaml` — pnpm build configuration
  for the Svelte frontend (`node-linker=hoisted`, esbuild as the only built
  dependency).

## [Unreleased] - 2026-09-07
### Added
- 18 new SG/common-law & jurisdiction-aware tabular-review workflow presets (config/workflow-presets/legal/): i-governance-assessment (+-sg/-au), i-act-conformity-eu, dpia-cross-border (+-sg/-au), horizon-scan-obligation-map (+-sg/-au), consultation-response-sg/-au, privacy-notice-consent-sg/-au, cloud-saas-dpa-review, oss-licence-review, online-safety-code-sg/-au. Authored with legal-counsel (Astra); validated against the Specter wire schema and deployed to both the install dir and this repo.

## [Unreleased] - docs cleanup
### Removed
- Removed upstream Italian/EU domain & plan docs (macchine*, piano_*, nis2-prompts, pa-prompts, TEMPLATE_PRONTUARIO, Toolkit_Prompt_Commercialista, EURLEX_REGISTRATION, PLAN_FONTI_INTERNAZIONALI, PLAN_MISTRAL, insurance-workflows-plan, PLAN/session recaps) not relevant to Specter.s APAC/common-law focus.
### Changed
- NOTICE.md: Specter trademarks now attributed to this project (not upstream); logo path corrected; dead semplifica.ai link removed; third-party trademark list updated to the APAC official legal sources.
- CORPORA.md: rewritten from the EU/Danish corpus survey to document the 8 bundled APAC official-government corpora.
- WORKFLOWS.md: built-in preset count updated (14 -> 25) and Italian UI strings replaced with English.

## [Unreleased] - docs cleanup
### Removed
- Removed upstream Italian/EU domain & plan docs (macchine*, piano_*, nis2-prompts, pa-prompts, TEMPLATE_PRONTUARIO, Toolkit_Prompt_Commercialista_Bilanci, EURLEX_REGISTRATION, PLAN_FONTI_INTERNAZIONALI, PLAN_MISTRAL, insurance-workflows-plan, PLAN.md, SESSION_RECAP) not relevant to Specter APAC/common-law focus.
### Changed
- WORKFLOWS.md: built-in preset count updated (14 -> 25) and Italian UI strings replaced with English.
