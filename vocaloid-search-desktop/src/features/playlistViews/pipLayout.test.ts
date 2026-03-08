import { describe, expect, test } from 'vitest'
import { getPipLayout } from './pipLayout'

describe('pipLayout', () => {
  test('returns PiP content order with fixed sidebar controls and right column sections as header, player, details', () => {
    expect(getPipLayout()).toEqual({
      sidebar: ['watchLater', 'previous', 'playPause', 'next'],
      content: [
        {
          section: 'header',
          videoMetaPanelMode: 'header',
        },
        {
          section: 'player',
        },
        {
          section: 'details',
          videoMetaPanelMode: 'details',
        },
      ],
    })
  })
})
