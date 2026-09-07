// Copyright (c) 2026 Specter contributors. Licensed under AGPL-3.0-only.

import { api } from './client'

/** Wrappers for local sync and manifest-driven corpus routes. */

// ── Local folder sync ────────────────────────────────────────────────

export interface SyncFolder {
  id: string
  path: string
  label: string | null
  recursive: boolean
  enabled: boolean
  last_scan_at: string | null
  project_id: string | null
}

export interface ScanStatus {
  status: 'idle' | 'running' | 'done' | 'failed'
  total?: number
  processed?: number
  indexed?: number
  skipped?: number
  failed?: number
  current_file?: string | null
  current_step?: string | null
  last_error?: string | null
}

export interface SyncedFile {
  path: string
  status: string
  document_id: string | null
  skip_reason: string | null
  size_bytes: number
  chunk_count: number
  indexed_at: string | null
  mtime: string | null
}

export type ModelStatus =
  | { state: 'idle' | 'loading' | 'ready' | 'unavailable' }
  | { state: 'downloading'; downloaded: number; total: number; file: string }
  | { state: 'failed'; error: string }

/**
 * GLiNER2 PII engine bootstrap snapshot. Polled by the chat
 * composer while at least one attached file has PII protection on
 * so the user sees an indeterminate "Loading PII model…" stripe
 * rather than a silent multi-minute wait the first time the model
 * is downloaded from HuggingFace.
 */
export type NerStatus =
  | { state: 'idle' | 'loading' | 'ready' | 'unavailable' }
  | { state: 'downloading'; downloaded: number; total: number | null; file: string }
  | { state: 'failed'; error: string }

export const syncApi = {
  listFolders: () => api<SyncFolder[]>('/sync/folders'),

  addFolder: (body: { path: string; recursive?: boolean; label?: string; project_id?: string }) =>
    api<{ id: string }>('/sync/folders', { method: 'POST', body }),

  deleteFolder: (id: string) =>
    api<{ ok: boolean }>(`/sync/folders/${encodeURIComponent(id)}`, { method: 'DELETE' }),

  startScan: (id: string) =>
    api<{ started?: boolean; already_running?: boolean }>(
      `/sync/folders/${encodeURIComponent(id)}/scan`,
      { method: 'POST' },
    ),

  scanStatus: (id: string) =>
    api<ScanStatus>(`/sync/folders/${encodeURIComponent(id)}/status`),

  listFiles: (id: string) =>
    api<SyncedFile[]>(`/sync/folders/${encodeURIComponent(id)}/files`),

  modelStatus: () => api<ModelStatus>('/sync/model-status'),

  /** GLiNER2 PII engine bootstrap snapshot. Returns
   *  `{ state: "unavailable" }` outside the `ner-pii` build, so the
   *  frontend can poll uniformly. */
  nerStatus: () => api<NerStatus>('/sync/ner-status'),
}

export interface CorpusHit {
  identifier: string
  title: string
  date: string | null
  url: string
  languages_available: string[]
}

// ── Corpora registry ─────────────────────────────────────────────────

export interface CorpusCapabilities {
  search: boolean
  fetch: boolean
  documents: boolean
  documents_delete: boolean
  documents_resync: boolean
  embed_progress: boolean
  bulk_import: boolean
  user_config: boolean
}

export interface CorpusSourceItem {
  id: string
  display_name: string
  subtitle?: string | null
  description?: string | null
  available: boolean
  default_enabled: boolean
  status_label?: string | null
}

/** Discovery metadata for the data-sources filter UI and badges. */
export interface CorpusDiscovery {
  jurisdiction?: string | null
  doc_types: string[]
  auth?: string | null
  search_mode?: string | null
  fetch_format?: string | null
}

export interface CorpusItem {
  id: string
  display_name: string
  description: string
  homepage: string
  languages: string[]
  default_language: string
  identifier_label: string
  identifier_example: string
  enabled_by_default: boolean
  /** Manifest kill switch — false retires a non-working connector. */
  available: boolean
  runnable: boolean
  capabilities: CorpusCapabilities
  sources: CorpusSourceItem[]
  discovery?: CorpusDiscovery | null
}

/** Per-user enable/disable (+ language) for any corpus. */
export interface CorpusConfig {
  enabled: boolean
  language?: string
  fallback_en?: boolean
}

export const corporaApi = {
  /** All registered corpus plugins (from config/corpora-plugins/*.json). */
  list: () => api<{ corpora: CorpusItem[] }>('/corpora'),
}

export interface CorpusDocument {
  id: string
  filename: string
  corpus_identifier: string | null
  corpus_date?: string | null
  size_bytes: number
  created_at: string
  status: string
}

// ── Generic corpus (declarative plugins, /corpora/{id}/* routes) ─────

/** API surface for a manifest-defined APAC corpus. */
export function genericCorpusApi(id: string) {
  const base = `/corpora/${encodeURIComponent(id)}`
  return {
    search: (query: string) =>
      api<{ hits: CorpusHit[]; note: string | null }>(`${base}/search`, {
        method: 'POST',
        body: { query },
      }),
    fetch: (identifier: string, opts?: { signal?: AbortSignal; date?: string }) =>
      api<unknown>(`${base}/fetch`, {
        method: 'POST',
        body: { identifier, date: opts?.date },
        signal: opts?.signal,
      }),
    /** Read-only preview — fetches the document body without persisting
     *  or chunking. Backed by `GET /corpora/<id>/preview?identifier=…`. */
    preview: (
      identifier: string,
      opts?: { signal?: AbortSignal; language?: string },
    ) => {
      const params = new URLSearchParams({ identifier })
      if (opts?.language) params.set('language', opts.language)
      return api<{
        identifier: string
        title: string
        source_url: string
        text: string
      }>(`${base}/preview?${params.toString()}`, {
        method: 'GET',
        signal: opts?.signal,
      })
    },
    documents: () => api<{ documents: CorpusDocument[] }>(`${base}/documents`),
    deleteDocument: (docId: string) =>
      api<{ ok: boolean }>(`${base}/documents/${encodeURIComponent(docId)}`, {
        method: 'DELETE',
      }),
    resyncDocument: (docId: string, opts?: { signal?: AbortSignal }) =>
      api<unknown>(`${base}/documents/${encodeURIComponent(docId)}/resync`, {
        method: 'POST',
        signal: opts?.signal,
      }),
    getConfig: () => api<CorpusConfig>(`${base}/config`),
    setConfig: (body: CorpusConfig) =>
      api<CorpusConfig>(`${base}/config`, { method: 'PUT', body }),
  }
}
