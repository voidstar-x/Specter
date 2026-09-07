// Copyright (c) 2026 Specter contributors. Licensed under AGPL-3.0-only.

/**
 * English-only UI dictionary. Legacy persisted locale values are
 * normalised to English at the runtime boundary.
 */

import { DEFAULT_LOCALE, isLocale, type Locale } from '$lib/types/user'
import en from '../../../locales/en.json'

type Dict = Record<string, unknown>

const dict: Dict = en

/** Resolve a dotted key path (e.g. `Domains.values.legal`) to a string. */
function resolve(dict: Dict, path: string): string | undefined {
  let cur: unknown = dict
  for (const seg of path.split('.')) {
    if (cur && typeof cur === 'object') {
      cur = (cur as Dict)[seg]
    } else {
      return undefined
    }
  }
  return typeof cur === 'string' ? cur : undefined
}

function createI18n() {
  let locale = $state<Locale>(DEFAULT_LOCALE)

  return {
    get locale() {
      return locale
    },

    setLocale(loc: unknown) {
      locale = isLocale(loc) ? loc : DEFAULT_LOCALE
    },

    /** Pick the best initial locale from a BCP-47 tag (e.g. navigator.language). */
    detectFrom(tag: string | undefined) {
      const short = (tag ?? '').slice(0, 2).toLowerCase()
      this.setLocale(short)
    },

    /**
     * Translate a dotted key. Missing keys fall back to the raw key.
     * `{name}`-style placeholders are filled
     * from `params`.
     */
    t(key: string, params?: Record<string, string | number>): string {
      let raw = resolve(dict, key) ?? key
      if (params) {
        for (const [k, v] of Object.entries(params)) {
          raw = raw.replaceAll(`{${k}}`, String(v))
        }
      }
      return raw
    },
  }
}

export const i18n = createI18n()
