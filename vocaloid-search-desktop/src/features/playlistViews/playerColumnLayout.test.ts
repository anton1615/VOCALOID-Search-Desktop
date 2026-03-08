import { describe, expect, test } from 'vitest'
import { getPlayerColumnLayout } from './playerColumnLayout'

describe('playerColumnLayout', () => {
  test('returns the right column sections in header, player, controls, details order', () => {
    expect(getPlayerColumnLayout()).toEqual([
      {
        section: 'header',
        videoMetaPanelMode: 'header',
      },
      {
        section: 'player',
      },
      {
        section: 'controls',
      },
      {
        section: 'details',
        videoMetaPanelMode: 'details',
      },
    ])
  })
})
