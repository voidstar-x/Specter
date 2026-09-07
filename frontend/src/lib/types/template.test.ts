// Copyright (c) 2026 Specter contributors. Licensed under AGPL-3.0-only.
import { describe, expect, it } from 'vitest'
import { TEMPLATE_NAME_LOCALES, blankUserTemplate, templateDisplayName } from './template'

describe('English-only DOCX template defaults', () => {
  it('edits display names for English only', () => {
    expect([...TEMPLATE_NAME_LOCALES]).toEqual(['en'])
  })

  it('creates blank templates with an English locale', () => {
    const tpl = blankUserTemplate()
    expect(tpl.locale).toBe('en-SG')
  })

  it('creates blank templates with English style names', () => {
    const tpl = blankUserTemplate()
    expect(tpl.style_map_baseline).toEqual({
      body_text: 'Body Text',
      section_heading: 'Section Heading',
      citation: 'Citation',
      footnote: 'Footnote',
    })
  })

  it('still resolves legacy Italian display names and falls back to en then id', () => {
    const legacy = blankUserTemplate()
    legacy.id = 'user/nda'
    legacy.display_name = { it: 'Accordo di riservatezza', en: 'NDA' }
    expect(templateDisplayName(legacy, 'en')).toBe('NDA')
    expect(templateDisplayName(legacy, 'it-IT')).toBe('Accordo di riservatezza')
    // No matching entry anywhere → raw id.
    const bare = blankUserTemplate()
    bare.id = 'user/x'
    expect(templateDisplayName(bare, 'en')).toBe('user/x')
  })

  it('keeps legacy stored schema field names on the blank template', () => {
    const tpl = blankUserTemplate()
    // v0.x schema field names preserved for round-tripping with old sidecars.
    expect(Object.keys(tpl)).toContain('uso_bollo' as never)
    expect(tpl).toHaveProperty('style_map_baseline')
    expect(tpl).toHaveProperty('section_numbering', 'manual')
  })
})
