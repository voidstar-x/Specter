// Copyright (c) 2026 Specter contributors. Licensed under AGPL-3.0-only.
import { describe, expect, it } from 'vitest'
import { i18n } from './i18n.svelte'
import type { Locale } from '$lib/types/user'

describe('English-only i18n store', () => {
  it('starts on English with the en dictionary', () => {
    expect(i18n.locale).toBe('en')
    expect(i18n.t('Auth.unlock')).toBe('Unlock') // real key from en.json
  })

  it('normalises a stale persisted non-English locale back to en', () => {
    // Simulates a stale persisted locale ("it") that survived the
    // catalogue shrink: setLocale must never leave the store off en.
    i18n.setLocale('it' as unknown as Locale)
    expect(i18n.locale).toBe('en')
    expect(i18n.t('Auth.unlock')).toBe('Unlock')
  })

  it('does not adopt a browser language outside the catalogue', () => {
    i18n.detectFrom('it-IT')
    expect(i18n.locale).toBe('en')
    i18n.detectFrom('fr')
    expect(i18n.locale).toBe('en')
    i18n.detectFrom(undefined)
    expect(i18n.locale).toBe('en')
  })

  it('falls back to the raw key for unknown keys with placeholder support', () => {
    expect(i18n.t('NoSuch.key')).toBe('NoSuch.key')
    expect(i18n.t('NoSuch.key', { name: 'Han' })).toBe('NoSuch.key')
  })
})
