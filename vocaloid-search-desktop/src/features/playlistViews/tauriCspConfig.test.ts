import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'

type CspConfig = Record<string, string | string[]>

function readTauriCspConfig(): CspConfig | string | null {
  const tauriConfigPath = resolve(__dirname, '../../../src-tauri/tauri.conf.json')
  const source = JSON.parse(readFileSync(tauriConfigPath, 'utf8'))
  return source.app?.security?.csp ?? null
}

function getDirectiveValues(csp: CspConfig, directive: string): string[] {
  const value = csp[directive]
  if (Array.isArray(value)) return value
  return typeof value === 'string' ? value.split(/\s+/).filter(Boolean) : []
}

describe('tauri CSP baseline', () => {
  test('does not leave desktop WebView CSP disabled', () => {
    expect(readTauriCspConfig()).not.toBeNull()
  })

  test('allows current dev and playback origins', () => {
    const csp = readTauriCspConfig()

    expect(csp).not.toBeNull()
    expect(typeof csp).toBe('object')

    const directives = csp as CspConfig

    expect(getDirectiveValues(directives, 'default-src')).toEqual(expect.arrayContaining([
      "'self'",
      'customprotocol:',
      'asset:',
      'http://localhost:1420',
      'http://127.0.0.1:1420',
    ]))
    expect(getDirectiveValues(directives, 'script-src')).toEqual(expect.arrayContaining([
      "'self'",
      'http://localhost:1420',
      'http://127.0.0.1:1420',
      "'unsafe-eval'",
    ]))
    expect(getDirectiveValues(directives, 'connect-src')).toEqual(expect.arrayContaining([
      "'self'",
      'ipc:',
      'http://ipc.localhost',
      'http://localhost:1420',
      'http://127.0.0.1:1420',
      'ws://localhost:1420',
      'ws://127.0.0.1:1420',
      'https://embed.nicovideo.jp',
    ]))
    expect(getDirectiveValues(directives, 'img-src')).toEqual(expect.arrayContaining([
      "'self'",
      'asset:',
      'http://asset.localhost',
      'data:',
      'blob:',
      'https://secure-dcdn.cdn.nimg.jp',
      'https://nicovideo.cdn.nimg.jp',
      'https://image.nicovideo.jp',
    ]))
    expect(getDirectiveValues(directives, 'frame-src')).toEqual(expect.arrayContaining([
      "'self'",
      'https://embed.nicovideo.jp',
    ]))
    expect(getDirectiveValues(directives, 'style-src')).toEqual(expect.arrayContaining([
      "'self'",
      "'unsafe-inline'",
      'http://localhost:1420',
      'http://127.0.0.1:1420',
    ]))
  })
})
