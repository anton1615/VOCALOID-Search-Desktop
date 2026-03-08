import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'
import { getVideoMetaPanelLayout } from './videoMetaPanelLayout'

describe('videoMetaPanelLayout', () => {
  test('returns both sections for full mode', () => {
    expect(getVideoMetaPanelLayout('full')).toEqual({
      showHeader: true,
      showDetails: true,
    })
  })

  test('returns only header for header mode', () => {
    expect(getVideoMetaPanelLayout('header')).toEqual({
      showHeader: true,
      showDetails: false,
    })
  })

  test('returns only details for details mode', () => {
    expect(getVideoMetaPanelLayout('details')).toEqual({
      showHeader: false,
      showDetails: true,
    })
  })

  test('uses the planned full-width bar button style for the description toggle', () => {
    const videoMetaPanelPath = resolve(__dirname, '../../components/VideoMetaPanel.vue')
    const source = readFileSync(videoMetaPanelPath, 'utf8')

    expect(source).toContain('.expand-btn {')
    expect(source).toContain('display: block;')
    expect(source).toContain('width: 100%;')
    expect(source).toContain('margin-top: var(--space-sm);')
    expect(source).toContain('padding: var(--space-xs);')
    expect(source).toContain('border-radius: 4px;')
    expect(source).toContain('background: transparent;')
    expect(source).toContain('color: var(--color-accent-primary);')
    expect(source).toContain('font-size: var(--font-size-sm);')
    expect(source).toContain('font-weight: 600;')
    expect(source).toContain('text-align: center;')
    expect(source).toContain('.expand-btn:hover')
  })
})
