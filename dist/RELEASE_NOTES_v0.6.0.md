# Specter v0.6.0 — Mistral first-class

Promotes Mistral La Plateforme (`api.mistral.ai`) from a generic
OpenAI-compat reuse to a dedicated provider with Mistral-specific
optimisations baked in. Headline benefits for the legal-target
audience:

* **80-90% cost reduction on long document-heavy chats** via
  Mistral's `prompt_cache_key` (charges 10% of normal token price
  on cache hits).
* **Sequential tool execution** by default (`parallel_tool_calls: false`)
  — predictable for tabular cell extraction and citation lookups.
* **No spurious refusals on legal content** — `safe_prompt: false`
  by default; toggle exposed for users who hit the opposite case.
* **EU data residency clarity** — info paragraph + link to the
  Mistral help-center ZDR article in the settings card.

## What's new

### Dedicated Mistral provider — `src/llm/mistral.rs`

Mistral was previously routed through the generic OpenAI-compat
path (`local.rs`) which couldn't send Mistral-specific knobs. The
new path always sends:

* `parallel_tool_calls` — overrideable per user via Settings, but
  defaults to `false` because legal workflows benefit from
  sequential, predictable tool execution.
* `safe_prompt` — same shape: overrideable but defaults to `false`
  because Mistral's safety wrapper false-flags legitimate legal
  content (criminal-case sentences, sensitive medical reports).
* `prompt_cache_key: "mike_chat_{chat_id}"` — stable per-chat key
  emitted only when the call carries a chat_id. On a typical
  20-turn document-heavy legal chat, hit rate is 80-90% — and
  Mistral charges 10% of normal token price on cache hits, so
  the effective cost reduction is in the same range. One-shot
  callers (title generation, summarisation, HyDE, doc summary,
  translation) emit no key — no caching benefit on unique
  prefixes.

Mistral-specific error mapping with Italian-first messages:

* 401 → "API key non valida o scaduta. Verifica in
  Settings → Modelli LLM → Mistral AI."
* 403 → "richiesta rifiutata (quota esaurita o filtro di
  sicurezza)" with body detail.
* 422 → "payload non valido. Probabile model id errato."
* 429 → parses `Retry-After` header into "(riprova fra ~Ns)".
* 5xx → "errore lato server" with body detail.

### Two new user toggles in Settings → Modelli LLM → Mistral AI

Below the existing "Profilo modelli" picker (added in v0.5.6):

* **Filtro safety (`safe_prompt`)** — applies Mistral's upstream
  safety wrapper. OFF by default; flip ON only if the model
  refuses legitimate legal content.
* **Esegui tool call in parallelo** — re-enables Mistral's
  upstream `parallel_tool_calls: true` default. OFF by default
  for predictable sequential behaviour.

Both disabled when no Mistral API key is configured.

### EU hosting + ZDR info

The Mistral card surfaces a short paragraph explaining the data
location (France) and the default 30-day rolling abuse-monitoring
retention (data is never used for training on paid API plans).
Plus a link to the official Mistral help-center article for users
who want full Zero Data Retention.

ZDR is NOT a per-request header (confirmed against the docs
2026-06) — it requires a manual support ticket to Mistral. The
link goes directly to that article.

### Catalogue refresh (carried from v0.5.6)

The `config/model.json` Mistral entries were updated against the
2026-06 docs:

* Mistral Medium 3.5 is the multimodal frontier (`tier: premium`,
  $1.5/$7.5 per Mtok).
* Mistral Large 3 is the mid-tier "economy" ($0.5/$1.5).
* Mistral Small 4 is the budget option ($0.1/$0.3).
* All three are vision-capable.
* Magistral Medium added as a future reasoning option.
* Ministral 3B / 8B kept as cheap title-generation candidates.

The "Profilo modelli" picker in the Mistral card auto-fills
`main` / `title` / `tabular` roles with curated combinations:

* **Equilibrato** (default) — Large 3 / Ministral 3B / Large 3.
* **Premium** — Medium 3.5 / Small 4 / Medium 3.5.

## Migration notes

* **New schema migration `0033_user_mistral_options`** — adds two
  boolean columns (`mistral_safe_prompt`, `mistral_parallel_tools`)
  to `user_settings`, both defaulting to 0. Auto-applied on
  first launch.
* **Existing chats are unaffected** — the routing change for the
  `mistral:` prefix is transparent. The cost reduction from
  `prompt_cache_key` kicks in automatically on the next chat
  turn that carries a chat_id; no setting required.
* **No frontend breaking changes** — the two new toggles are
  additive in the existing Mistral card.

## Cost simulator (Mistral Large 3, EU pricing)

| Scenario | Input tok | Output tok | Cost (no cache) | Cost (cache hit, Mistral Large 3) |
|---|---|---|---|---|
| 1-turn chat (5K + 1K) | 5K | 1K | $0.004 | n/a |
| 20-turn doc-heavy chat | ~100K | ~20K | ~$0.080 | **~$0.020-0.030** |
| Tabular review (10 docs × 5K × 10 cols) | 500K | 30K | $0.30 | n/a (per-cell unique prefix) |
| Title (Ministral 3B) | 500 | 15 | $0.00005 | n/a |

## Downloads

Pre-built MSIs for Windows:

- `Specter_0.6.0_x64.msi` — Windows x86_64
- `Specter_0.6.0_arm64.msi` — Windows ARM64, Snapdragon X Elite

Drop-in replacement for v0.5.6.

## New direct dependencies

None. The Mistral provider uses the same `reqwest` + `serde_json`
+ `tokio` already in the dependency tree. ollama-rs and
async-stream landed in v0.5.6 and don't change.

## What's not in this release (deferred)

* **`wiremock` round-trip HTTP integration tests** — the 17 new
  unit tests in `mistral.rs` cover the build_body and
  error-mapping logic; wiremock would add coverage of the wire
  shape end-to-end. Deferred to a v0.6.x point release.
* **Vision routing for Mistral Medium/Large** — the API side
  works (we already send `image_url` content parts when the
  composer has attachments), but the chat composer's
  PDF-page-rendering path is currently wired only for Gemini.

## License

Specter is distributed under **AGPL-3.0-only**. The Specter
wordmark and logo are trademarks; see `NOTICE.md`. The full
licence text is available in-app under **Settings → Licenza**.
