# Authoritative legal corpora

Plan and per-source survey for ingesting public legal sources into
Specter's RAG store, configurable per-corpus from
**Settings → Local documents / Corpora**.

Specter is oriented to APAC / common-law jurisdictions. The bundled
corpus plugins each target an **official government source** for a
jurisdiction; sources are manifest-driven and live under
`config/corpora-plugins/*.json`, so adding or changing a source is a
config-only change (no code). Manifest shape is documented in
[`CORPUS_PLUGINS.md`](CORPUS_PLUGINS.md).

## Goals

1. Let the user opt-in to one or more authoritative corpora — they're
   large, so they default off.
2. Per-corpus controls:
   - **Enabled** toggle (powers sync on/off).
   - **Reference language** picker (for multilingual sources).
   - **Search by law number / identifier** — e.g. an Act number or
     gazette reference.
   - **Search by keyword** — full-text search routed to the source's
     native search endpoint (not RAG).
3. Ingestion is opt-in *per document*: search returns hits, the user
   picks which to add to their personal index. We don't bulk-mirror
   national gazettes — that's gigabytes and most of it is irrelevant
   to any one user.
4. Ingested documents land in the same `sqlite-vec` partition as
   folder-synced docs (scope = `global` by default), so retrieval
   treats them uniformly.

## Bundled APAC source survey

Official government sources only. Some sources require a user-agent
or a direct-PDF fetch; the `DirectPdf` fetch shape handles the
PDF-only sources.

### Singapore — Singapore Statutes Online
- **Site**: https://sso.agc.gov.sg
- **Notes**: Attorney-General's Chambers consolidated statutes; needs a
  browser user-agent; body selector `#legisContent`.

### Malaysia — LOM (Laws of Malaysia)
- **Site**: https://lom.agc.gov.my
- **Notes**: Attorney General's Chambers; PDF/HTML act detail pages.

### Indonesia — Peraturan
- **Site**: https://peraturan.go.id / https://peraturan.bpk.go.id
- **Notes**: bpk.go.id is a scripted/Cloudflare-protected portal; uses
  the DirectPdf adapter.

### South Korea — Korea Law Information Center
- **Site**: https://law.go.kr
- **Notes**: JS-rendered pages; uses the DirectPdf adapter.

### Vietnam — Legal Portal
- **Site**: https://phapluat.gov.vn / https://vanban.chinhphu.vn
- **Notes**: Government legal documents portal.

### Thailand — Royal Gazette
- **Site**: https://ratchakitcha.soc.go.th / https://krisdika.go.th
- **Notes**: Uses the DirectPdf adapter.

### Australia — Federal Register of Legislation
- **Site**: https://legislation.gov.au
- **Notes**: Official consolidated Australian legislation.

### Japan — e-Gov
- **Site**: https://e-gov.go.jp
- **Notes**: Official government portal for Japanese laws.

## Adding a new source

1. Copy an existing manifest in `config/corpora-plugins/` and adapt
   `id`, `name`, `url`, `fetch_shape` (one of `http` / `direct_pdf` /
   `js`), and the extraction selectors.
2. Set `available: true`.
3. Restart Specter; the corpus appears in Settings → Local documents.
