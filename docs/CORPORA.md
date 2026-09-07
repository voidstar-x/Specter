# Authoritative legal corpora

Specter's bundled source catalogue targets official APAC government sources.
The manifests live in [`config/corpora-plugins/`](../config/corpora-plugins/) and
are surfaced through **Settings → Data sources**. See [CORPUS_PLUGINS](CORPUS_PLUGINS.md)
for the schema, capability flags and extension procedure.

## Scope and limits

- Index documents selected for the user's work; the bundled connectors are
  not complete offline mirrors of national legislation or case law.
- Availability and initial enabled state come from each manifest. They do not
  prove coverage, legal currency or successful retrieval of every identifier.
- Fetching by identifier is distinct from keyword search. Only offer search
  when the manifest and endpoint actually support it.
- Source-document language is independent of Specter's English-only interface.
  Do not silently present translated wording as authoritative original text.
- Preserve official source URLs, publisher attribution and redistribution terms.
  Check jurisdiction, version, commencement and amendments before relying on law.

## Bundled manifest survey

The table below describes the checked-in configuration, not a fresh network
verification. Consult each manifest for its exact URL, selectors, availability
and capability flags; publishers may change their sites independently.

| Corpus | Display name | Fetch response shape | Source languages | Identifier input |
|---|---|---|---|---|
| [`au-federalregister`](../config/corpora-plugins/au-federalregister.json) | AU / Federal Register | `rest-html` | en | Register ID |
| [`id-peraturan`](../config/corpora-plugins/id-peraturan.json) | Indonesia / National Regulations (Peraturan) | `direct-pdf` | id | Full official PDF URL |
| [`jp-egov`](../config/corpora-plugins/jp-egov.json) | JP / e-Gov | `rest-html` | ja | LawId |
| [`kr-lawinfo`](../config/corpora-plugins/kr-lawinfo.json) | Korea / Law Information Center (law.go.kr) | `rest-html` | ko | Law serial (lsiSeq) |
| [`my-lom`](../config/corpora-plugins/my-lom.json) | Malaysia / Laws of Malaysia (LOM) | `direct-pdf` | en, ms | Full official LOM PDF URL (processFile.php?token=... harvested from the site) |
| [`sg-statutes`](../config/corpora-plugins/sg-statutes.json) | Singapore / Statutes (Singapore Statutes Online) | `rest-html` | en | Act short code (e.g. IA1965) |
| [`th-royalgazette`](../config/corpora-plugins/th-royalgazette.json) | Thailand / Official Law Library (PRD, Government of Thailand) | `direct-pdf` | th | Full official law.prd.go.th PDF URL |
| [`vn-legal`](../config/corpora-plugins/vn-legal.json) | Vietnam / Government Legal Documents (Chinh Phu) | `rest-html` | vi | 18-digit publication id (from xaydungchinhsach.chinhphu.vn URL) |

## Direct PDF handling

`direct-pdf` is a response shape under `strategy.kind: "http-fetch-per-id"`.
It downloads an official PDF and extracts its text layer through Pdfium. The
backend must be built with the `pdf` feature and have the native Pdfium runtime.
Image-only/scanned PDFs without extractable text are not OCR'd by this path.
An HTML portal or anti-bot challenge is not a substitute for a PDF URL.

## Add or update a source

1. Copy a manifest with the appropriate response shape.
2. Set `id`, English `display_name`, official `homepage`, language fields and
   `strategy.search_by_id` (`url_template`, `shape`, selectors).
3. Explicitly set capabilities, keeping `bulk_import: false`. Keep a new source
   unavailable until its fetch and error paths have been verified.
4. Restart the backend, inspect `GET /corpora`, and exercise a real document
   through the UI/API. Check the extracted body, not just the HTTP status.

There is no `fetch_shape: js` browser-execution mode. Sources requiring login,
interactive challenges or unsupported formats need a different implementation,
not invented manifest fields.

## Historical connectors

EUR-Lex, Italian Legal, Fedlex and DILA bulk connectors are retired from this
fork. Their historical engineering work and attribution remain in
[HISTORY](../HISTORY.md). The English EU AI Act workflow is intentional legal
content and remains separate from the removed connector infrastructure.
