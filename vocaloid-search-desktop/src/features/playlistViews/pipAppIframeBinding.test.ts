import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'

describe('PipApp iframe binding', () => {
  test('binds the NicoNico embed src declaratively on the iframe', () => {
    const pipAppPath = resolve(__dirname, '../../PipApp.vue')
    const source = readFileSync(pipAppPath, 'utf8')

    expect(source).toContain(':src="`https://embed.nicovideo.jp/watch/${currentVideo.id}?jsapi=1&playerId=1`"')
  })
})
