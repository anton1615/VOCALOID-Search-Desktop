import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'

describe('Watch Later removal confirmation contract', () => {
  test('WatchLaterView gates list-item removal behind a confirmation dialog', () => {
    const watchLaterViewPath = resolve(__dirname, '../../views/WatchLaterView.vue')
    const source = readFileSync(watchLaterViewPath, 'utf8')

    expect(source).toContain('showRemoveConfirm')
    expect(source).toContain('pendingRemovalVideo')
    expect(source).toContain("t('watchLater.confirmRemoveTitle')")
    expect(source).toContain("t('watchLater.confirmRemoveMessage')")
    expect(source).toContain('@click="promptRemove(video)"')
    expect(source).toContain('@click="confirmRemove"')
    expect(source).toContain('class="btn-secondary modal-btn"')
    expect(source).toContain('class="btn-danger modal-btn modal-btn-danger"')
    expect(source).toContain('.modal-btn {')
    expect(source).toContain('min-width: 110px;')
    expect(source).toContain('box-shadow: 0 10px 24px rgba(0, 0, 0, 0.16);')
    expect(source).toContain('.modal-btn-danger {')
  })

  test('WatchLaterButton keeps immediate toggle removal without confirmation state', () => {
    const watchLaterButtonPath = resolve(__dirname, '../../components/WatchLaterButton.vue')
    const source = readFileSync(watchLaterButtonPath, 'utf8')

    expect(source).toContain('await api.removeFromWatchLater(props.videoId)')
    expect(source).not.toContain('showRemoveConfirm')
    expect(source).not.toContain('pendingRemovalVideo')
    expect(source).not.toContain('window.confirm')
  })
})
