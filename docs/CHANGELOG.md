# Specter — Change Log & Fork Notes

Original upstream: **MikeRust** (github.com/SemplificaAI/MikeRust, AGPL-3.0). This repository is the **Specter** fork (github.com/voidstar-x/Specter), maintained by Han (SG-based commercial / corporate lawyer, APAC + Australia focus).

This file records the substantive changes made on top of upstream. It is the canonical reference for what Specter does differently.

---

## Release — Specter 0.7.4 APAC / common-law build (2026-09-07)

### Added

- **25 workflow presets** in `config/workflow-presets/legal/` (tabular review), English-language and SG/common-law focused:
  - 7 core presets: `statutory-analysis` (jurisdiction-agnostic), `statutory-analysis-sg`, `statutory-analysis-au`, `nda-review-sg`, `saas-supply-review-sg`, `gdpr-pdpa-compliance`, `content-online-safety`.
  - 18 new regulatory/AI-governance presets: `ai-governance-assessment` (+`-sg`/`-au`), `ai-act-conformity-eu`, `dpia-cross-border` (+`-sg`/`-au`), `horizon-scan-obligation-map` (+`-sg`/`-au`), `consultation-response-sg`/`-au`, `privacy-notice-consent-sg`/`-au`, `cloud-saas-dpa-review`, `oss-licence-review`, `online-safety-code-sg`/`-au`. Authored via legal-counsel (Astra) and validated against the Specter wire schema.
- **8 APAC official-government corpus connectors** (`config/corpora-plugins/`): `sg-statutes` (Singapore), `my-lom` (Malaysia), `id-peraturan` (Indonesia), `kr-lawinfo` (Korea), `vn-legal` (Vietnam), `th-royalgazette` (Thailand), `au-federalregister` (Australia), `jp-egov` (Japan).
- **DirectPdf** fetch shape + **gzip** adapter support in `src/corpora/manifest_adapter.rs` and `src/corpora/plugin.rs` (needed for the PDF-only / gzip official sources).
- **DeepSeek V4 Flash Vision Exp** model entry in `config/model.json` under the `local` (OpenAI-compatible) provider, matching the local engine (`VLLM_MAIN_MODEL`).
- `examples/apac_probe.rs` and `scripts/verify-msi-config.ps1` (checks the built MSI file table against the source config).

### Changed

- **Default assistant/content language is now English.** locale default flipped to `"en"` in `src/routes/chat.rs` (was `"it"`); `FALLBACK_LOCALES` in `src/presets/system_prompt.rs` reordered to English-first; `config/system-prompts/en/` is the sole canonical prompt set (English-only resolution). Hard-coded Italian user-facing strings translated (GEMMA system prompt, "Secure local mode" preambles, document-summary prompt, corpus import status labels, test fixtures).
- API server now binds **`0.0.0.0`** (`.env` `PORT=3001`) so it is reachable on the LAN (was loopback).
- `config/column-presets/insurance/*` and `legal/*` field names/labels translated Italian → English.
- **Documentation** aligned to Specter: `README.md` rewritten (fork lineage, differences table, no upstream assets); `NOTICE.md` re-attributed to this project with corrected logo path, no dead `semplifica.ai` link, and the third-party trademark list updated to the APAC official sources; `CORPORA.md` rewritten from the EU/Danish corpus survey to the APAC corpora; `WORKFLOWS.md` built-in count updated (14 → 25) and Italian strings removed.

### Removed (upstream Italian/EU, not relevant to APAC/common-law)

- EU/EUR-Lex & European corpus code/routes/UI: `src/corpora/{eurlex,fedlex,dila_bulk,italian_legal,limits}.rs`, `src/routes/{eurlex,italian_legal}.rs`, `config/corpora.json`, `frontend/src/lib/components/settings/EurlexSection.svelte`.
- Non-English locale files (`frontend/locales/{de,es,fr,it,pt}.json`) and the `fill-i18n` generator (Specter is English-only).
- Italian DOCX templates (`config/docx-templates/it/*`, `config/docx-templates/compliance/{macchine-*,nis2-*,procedura-iso-sgi}`).
- Italian fiscal column presets (`config/column-presets/fiscale/*`), upstream dev-plan/session docs (`PLAN.md`, `PLAN_FONTI_INTERNAZIONALI.md`, `docs/specter-ui-rewrite-plan.md`), orphaned upstream screenshots (`docs/images/*`), and `tests/insurance_diffida_e2e.rs`.
- `Cargo.toml`/`Cargo.lock` deps for the removed EU corpus crates.

### Fixed

- **Installed preset lookup regression.** `src/presets/mod.rs` now treats executable-adjacent `config/` as authoritative after explicit environment overrides (working-directory lookup remains only as a dev fallback), and `scripts/build-release.ps1` purges stale release-config staging before Tauri recopies resources. This resolved the live app reporting **119 workflows** (94 stale upstream presets were being loaded from build staging) — it now serves exactly the 25 installed presets with no `builtin-*` IDs.
- `frontend/pnpm-workspace.yaml` `allowBuilds: esbuild` placeholder fixed to `esbuild: true` (the pnpm build had been failing on `ERR_PNPM_IGNORED_BUILDS`).
- `HISTORY.md` dangling cross-references to removed docs corrected and terse Italian neutralised.

### Build & verification

- `cargo check`, `pnpm install`, `pnpm build`, and `scripts/build-release.ps1 -Target x64` all exit 0. Library tests (420+), frontend tests (61), and DOCX integration tests (3) pass. The full `cargo test` remains blocked by a pre-existing `tests/embedding_perf.rs` fastembed incompatibility (non-exhaustive struct / removed fields).
- Fresh MSI rebuilt (`dist/Specter_0.7.4_x64.msi`); MSI admin-extraction verified exact hash matches and runtime paths for **25 workflows, 8 corpora, 30 columns, 12 English prompts**, no DOCX sidecars.
- Running installed app verified: `GET /workflow` returns exactly **25** unique IDs (zero `builtin-*`), health reports `workflows=25, columns=30, docx_templates=0`, the corpus API lists the 8 APAC sources, and a live chat prompt returned a correct English legal answer via the local engine.
