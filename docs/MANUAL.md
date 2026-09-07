# Specter — Configuration and user guide

This English guide replaces the obsolete Italian manual for the removed
React/Next.js interface. Specter uses the Svelte desktop interface inherited
from MikeRust. For project background and configuration locations, see the
[README](../README.md); for release-era behaviour and upstream attribution,
see [HISTORY](../HISTORY.md) and the [upstream sync record](UPSTREAM_SYNC.md).
The former `PLAN.md` and `docs/mike-upstream/README_SEMPLIFICA.md` are historical
documents removed from this fork, not current setup guides.

## 1. Main concepts

- **Profile:** your local identity and security settings.
- **Document:** an uploaded source file or a generated document.
- **Project:** a workspace for related documents, conversations and reviews.
- **Chat:** a conversation with the assistant, standalone or inside a project.
- **Workflow:** a reusable assistant prompt or tabular extraction definition.
- **Tabular review:** structured extraction across documents and columns.

The application and database are local. A remote model provider or MCP server
can receive information when you use it; local storage alone does not make a
remote model call private. Check the selected model, endpoint and document
scope before sending confidential material.

## 2. Configure a model

Open **Settings → LLM models**. Configure the provider you intend to use:
Anthropic, Google, OpenAI, Mistral, or a local OpenAI-compatible endpoint.
Use the exact model identifier supported by that endpoint. Cloud providers
require their own credentials; a local endpoint may or may not require a key.
Save the settings and select the intended model in the chat composer.

Model settings are managed through the backend; the removed React manual's
`specter_llm_settings` browser-storage recipe is not the current configuration
contract. See the [model settings store](../frontend/src/lib/stores/models.svelte.ts)
and [model settings panel](../frontend/src/lib/components/settings/ModelsSection.svelte).
Do not put real API keys into checked-in documentation or preset files.

Specter uses English interface and assistant content. Official source documents
may be in another language; the source language is not an interface locale.

## 3. Work with documents and citations

1. Start a chat or open a project.
2. Attach the documents needed for the question. Inside a project, check that
   the picker is showing the intended project and domain.
3. State the task, jurisdiction, relevant dates and desired output format.
4. Inspect cited passages in the side viewer. Verify the source and page rather
   than treating a generated citation as proof by itself.
5. Use the chat's document list to return to uploaded, generated or referenced
   files. Review generated DOCX content before accepting or sharing it.

Extraction and preview depend on the file format and model capabilities.
Scanned PDFs and images may require a vision-capable model. A successful upload
does not establish that every page was read correctly. See [DOCX extraction](DOCX.md)
for tracked changes and [attachment caching](CACHE.md) for cache lifecycle.

## 4. Projects and scope

Use **Projects** to group work by client, matter or task. Select the appropriate
domain and isolation setting. Strict project scope and shared/global documents
are different retrieval choices: check the scope before attaching or querying
sensitive documents. Domain labels help filter content; they are not a substitute
for confidentiality controls.

Project export uses the `.mikeprj` format. Share an export only with the intended
recipient and review what it contains. Release-era export limitations and fixes
are recorded in [HISTORY](../HISTORY.md); the current format implementation is in
[`src/mikeprj/`](../src/mikeprj/).

## 5. Workflows and tabular reviews

An **assistant workflow** supplies reusable instructions to a chat. A **tabular
workflow** defines columns and extraction prompts for a review. Built-in presets
are read-only; duplicate a preset to make an editable version for your matter.

For a tabular review, select a workflow, attach the source documents, inspect
the columns and start generation. Check extracted cells against the underlying
source. When information is absent, prefer an explicit "Not stated" response
over an inferred answer. Re-run only the scope you intend to replace.

See [WORKFLOWS](WORKFLOWS.md) for authoring, column formats and preset locations.
The English EU AI Act workflow is intentional legal-counsel content; removing
the legacy EUR-Lex connector does not remove or invalidate that workflow.

## 6. Sources and retrieval

Open **Settings → Data sources** to inspect available sources. The bundled
connector catalogue targets official APAC sources; availability, search modes,
identifiers and document languages vary. A manifest or successful lookup does
not establish that a corpus is complete or current.

- [CORPORA](CORPORA.md): source survey and scope.
- [CORPUS_PLUGINS](CORPUS_PLUGINS.md): manifest structure and extension guidance.
- Retrieval settings: review optional behaviours and their model-call costs
  before enabling them.
- MCP settings: connect only servers you trust and understand what their tools
  can read or change.

## 7. Troubleshooting and maintenance

- **Model error:** check the selected provider, endpoint, model identifier,
  credentials and the actual error message. A retry does not fix an invalid key.
- **Missing citation source:** check whether the backing document was removed
  or moved. Re-upload or re-index only the intended source.
- **Unexpected retrieval:** check the current project, domain and retrieval
  settings before broadening access.
- **Slow first use:** model downloads and local embedding initialization can
  take time; inspect progress and logs rather than repeatedly starting jobs.
- **Deletion or reset:** back up material you need first. Document, chat and
  project deletion can remove associated records and cached files.

AI output requires professional review. Verify jurisdiction, commencement,
amendments, authority and quotations against the applicable primary sources.
