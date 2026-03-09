import { describe, expect, test } from 'vitest'
import { formatStorageSize, formatVideoCount } from './scraperFormatting'

describe('scraper formatting', () => {
  test('formats kilobytes into megabytes and gigabytes for dialog display', () => {
    expect(formatStorageSize(null)).toBe('Unavailable')
    expect(formatStorageSize(512)).toBe('512 KB')
    expect(formatStorageSize(2_048)).toBe('2.0 MB')
    expect(formatStorageSize(3_145_728)).toBe('3.0 GB')
  })

  test('formats estimated video counts with comma separators', () => {
    expect(formatVideoCount(null)).toBe('Unavailable')
    expect(formatVideoCount(12)).toBe('12')
    expect(formatVideoCount(1234)).toBe('1,234')
    expect(formatVideoCount(9876543)).toBe('9,876,543')
  })
})
