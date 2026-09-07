// Copyright (c) 2026 Specter contributors. Licensed under AGPL-3.0-only.
import { describe, expect, it } from 'vitest'
import { DEFAULT_LOCALE, LOCALES, LOCALE_LABELS, isLocale } from './user'

describe('English-only locale catalogue', () => {
  it('exposes exactly the English locale', () => {
    expect([...LOCALES]).toEqual(['en'])
    expect(DEFAULT_LOCALE).toBe('en')
  })

  it('labels only English', () => {
    expect(Object.keys(LOCALE_LABELS)).toEqual(['en'])
    expect(LOCALE_LABELS.en).toBe('English')
  })

  it('accepts en and rejects retired locales', () => {
    expect(isLocale('en')).toBe(true)
    for (const retired of ['it', 'fr', 'de', 'es', 'pt']) {
      expect(isLocale(retired)).toBe(false)
    }
    expect(isLocale('EN')).toBe(false)
    expect(isLocale(null)).toBe(false)
    expect(isLocale(42)).toBe(false)
  })
})
