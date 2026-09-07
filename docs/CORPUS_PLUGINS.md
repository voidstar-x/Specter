# Corpus plugin manifests

Specter discovers legal sources through JSON manifests in
[`config/corpora-plugins/`](../config/corpora-plugins/). The backend loads and
validates them at startup, exposes metadata through `GET /corpora`, and uses
it when presenting indexed sources to the user. See [CORPORA](CORPORA.md) for
the APAC source survey.

The supported strategy is **`http-fetch-per-id`**, implemented by
[`ManifestAdapter`](../src/corpora/manifest_adapter.rs). Response shapes are
`rest-html`, `rest-json`, and **`direct-pdf` when compiled with the `pdf`
feature**. `direct-pdf` is a response shape, not a separate strategy kind.
A declarative manifest does not execute JavaScript or bypass a login wall.

## Historical strategies removed from this fork

The inherited EUR-Lex, Italian Legal and Fedlex builtin adapters and the DILA
bulk-import path are retired. `builtin`, `dila-bulk-xml` and the former
`hf-dataset-bulk` placeholder are not supported strategy values. Their old
CNIL/EUR-Lex/Italian Legal example manifests are historical documents removed
from this fork, not templates to copy. The version history and attribution
remain in [HISTORY](../HISTORY.md) and [UPSTREAM_SYNC](UPSTREAM_SYNC.md).

This connector cleanup does **not** remove the intentional English EU AI Act
workflow: a legal workflow and a bundled source connector are separate things.

## Layout and discovery

Use one JSON file per corpus under `config/corpora-plugins/`, for example
[`sg-statutes.json`](../config/corpora-plugins/sg-statutes.json) or
[`th-royalgazette.json`](../config/corpora-plugins/th-royalgazette.json).

The loader searches the working directory and executable ancestors for the
manifest directory. `MRUST_CORPUS_PLUGINS_DIR` overrides that discovery; use an
explicit path when packaging or testing. Restart the backend after changes.
The parser and validation rules live in [`src/corpora/plugin.rs`](../src/corpora/plugin.rs).

## Top-level fields

| Field | Meaning |
|---|---|
| `id` | Stable key matching `^[a-z][a-z0-9\-]*$`; persisted as `corpus_id`. Do not rename an existing key casually. |
| `display_name`, `description`, `homepage` | English display metadata and the official source homepage. |
| `languages`, `default_language` | Source-document language codes; the default must be in the list. English-only UI does not imply English-only primary sources. |
| `supports_language_fallback`, `fallback_language` | Schema fields for fallback policy. Set fallback support to `false` if it is not implemented and verified for the source. |
| `identifier_label`, `identifier_example` | Explain the exact input expected: an Act code, reference or full official PDF URL. |
| `available` | Whether the source is offered to users. Keep unverified sources unavailable. |
| `enabled_by_default` | Initial per-user enabled state, not proof that a source is complete or working. |
| `strategy` | `http-fetch-per-id` and its fetch/search specification. |
| `capabilities` | Explicit booleans controlling supported operations. Unspecified flags default to false. |
| `sources` | Optional sub-source metadata. An unavailable sub-source must not default to enabled. |
| `license`, `discovery` | Optional source attribution/licensing and picker/filter metadata. Preserve publisher credit and redistribution terms. |

The schema also retains optional `display_name_locale` metadata for
compatibility. New Specter display content should be English; do not confuse
those labels with the authentic language of a statute or judgment.

## Capabilities

| Flag | Purpose |
|---|---|
| `search` | Offer lookup/search when the source implements it. Do not enable just because fetching works. |
| `fetch` | Fetch and index an individual source document. |
| `documents` | List locally indexed documents for this corpus. |
| `documents_delete` | Delete indexed documents; requires `documents`. |
| `documents_resync` | Re-index documents; requires `documents`. |
| `user_config` | Offer per-user source settings. |
| `embed_progress` | Enable only when the corresponding progress operation is supported. Bundled fetch-only examples set it to false. |
| `bulk_import` | Retired for the supported HTTP strategy; keep false. Validation rejects true. |

The supported corpus flow uses the generic `/corpora/:id/...` routes, not
legacy `/eurlex/*` or `/italian-legal/*` endpoints. See the exact route contracts
in [`src/routes/corpora.rs`](../src/routes/corpora.rs).

## Fetch and search specifications

Under `strategy`, set `kind` to `http-fetch-per-id` and provide `search_by_id`:

- `url_template`: official fetch URL, with `{identifier}` and optionally `{lang}`.
- `shape`: `rest-html`, `rest-json`, or `direct-pdf`.
- `body_path`: CSS selector for HTML or a supported JSONPath for JSON.
- `title_path`, `date_path`: optional metadata selectors.

An optional `search_by_keyword` block describes the search URL, response shape,
`hits_path`, `identifier_at` and `title_at` selectors. Template substitution
supports `{query}`, `{lang}` and `{limit}` where appropriate; the optional
`url_template_year` also supports `{year}`. Unknown placeholders fail instead
of silently producing a malformed URL.

For HTML, extraction supports CSS selectors and `@attr` attribute extraction.
The JSONPath subset supports paths such as `$.a.b`, `$.a[0]` and `$.a[*].b`.
`:strip-prefix=...` and `:strip-suffix=...` remove literal affixes.
Check the implementation before relying on syntax beyond this subset.

### Direct PDF sources

For a PDF-only source, set `search_by_id.shape` to `direct-pdf`. A manifest may
use `url_template: "{identifier}"` when its identifier is a full official PDF
URL. Empty `title_path` and `body_path` are appropriate: PDF text extraction
does not use CSS or JSON selectors. Copy the shape from the checked-in Thai
manifest rather than inventing `fetch_shape`, `direct_pdf` or `js` fields.

The adapter downloads PDF bytes, checks the PDF signature, extracts the text
layer through Pdfium, and returns text for indexing. Scanned/image-only PDFs
without extractable text fail; this is **not OCR**. The `pdf` feature and native
Pdfium runtime are prerequisites. Without that feature, a `direct-pdf` manifest
fails to deserialize rather than silently falling back to HTML parsing.

## Add or change a source

1. Copy a checked-in manifest that matches the source's actual response shape.
2. Set its exact identifier contract, official URL, language and selectors.
3. Explicitly set capabilities and availability. Keep `bulk_import: false`.
4. Test a real document: verify status, content type, extracted title/body,
   jurisdiction and source URL. Test missing identifiers and anti-bot/error
   pages too. Do not treat HTTP 200 alone as a successful legal-text fetch.
5. Restart the backend and verify the exact corpus in `GET /corpora` and the
   source settings UI. Manifest validation is not an end-to-end source test.

## Registry behaviour and security

- Invalid manifests are logged and skipped without stopping other sources.
- Duplicate IDs warn and later definitions win; filesystem order is not a
  configuration mechanism. Keep IDs unique.
- Per-user enabled/configuration state is stored in the database, separate
  from the manifest's initial defaults.
- Trust matters even without executable plugins: a manifest chooses network
  destinations and parsers. Restrict sources to official endpoints; do not use
  untrusted arbitrary URLs or follow instructions embedded in fetched content.
- Full-URL identifiers require particular care with allowed hosts, redirects
  and local/private addresses. Review the adapter's checks before deployment;
  a homepage label by itself is not a security boundary.
- The HTTP adapter limits response bodies and detects common challenge pages.
  Those safeguards do not establish completeness, currency or legal authority.
- Preserve publisher attribution and applicable licence terms when exporting
  or redistributing source material. Retired Parquet/DILA instructions are not
  a reason to re-enable unsupported bulk import.

## Implementation references

- [`src/corpora/plugin.rs`](../src/corpora/plugin.rs): manifest schema and loader.
- [`src/corpora/manifest_adapter.rs`](../src/corpora/manifest_adapter.rs): HTTP and PDF extraction.
- [`src/corpora/mod.rs`](../src/corpora/mod.rs): corpus interface.
- [`src/routes/corpora.rs`](../src/routes/corpora.rs): generic source operations.
- [`src/routes/chat.rs`](../src/routes/chat.rs): indexed-source inventory in chat.
