# Specter v0.6.5 — Mistral "Fast" profile preset

Adds a third Mistral profile preset alongside the existing
"Equilibrato" and "Premium" buttons in
**Settings → Modelli LLM → Mistral AI**.

## The new "Veloce" (Fast) preset

| Role | Model | Price (per Mtok) |
|---|---|---|
| `main_model` | `mistral-small-latest` (Small 4) | $0.1 in / $0.3 out |
| `title_model` | `ministral-3b-latest` (Ministral 3B) | $0.1 in / $0.1 out |
| `tabular_model` | `mistral-small-latest` (Small 4) | $0.1 in / $0.3 out |

### Why these picks

* Mistral Small 4 is roughly **3-5× faster** than Large 3 on
  typical legal queries — ideal for triage flows or quick
  lookups where the user wants a snappy response rather than
  the absolute best answer.
* Pricing is **5× lower** on input and **5× lower on output**
  vs the Balanced profile's Large 3 — useful for tabular runs
  where each cell is a focused extraction (filename, dates,
  parties) rather than deep reasoning.
* Small 4 keeps **vision + function calling + 128K context**, so
  the chat composer doesn't lose capabilities — only the quality
  ceiling on complex multi-step reasoning.

The profile picker now uses a 3-column grid: **Veloce →
Equilibrato → Premium** (left-to-right, cheap-to-expensive). The
"Attivo" badge surfaces whichever profile matches the user's
current role assignments; `Personalizzato` is the implicit
fall-through when none match.

## Tests

Frontend-only change; svelte-check 0 errors. No backend changes
— v0.6.4's rate limiter + retry stack carries over unchanged.

## Downloads

Pre-built MSIs for Windows:

- `Specter_0.6.5_x64.msi` — Windows x86_64
- `Specter_0.6.5_arm64.msi` — Windows ARM64, Snapdragon X Elite

Drop-in replacement for v0.6.4.

## License

Specter is distributed under **AGPL-3.0-only**. The Specter
wordmark and logo are trademarks; see `NOTICE.md`. The full
licence text is available in-app under **Settings → Licenza**.
