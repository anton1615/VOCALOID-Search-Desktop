import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'

function readLocale(name: string): string {
  return readFileSync(resolve(__dirname, `../../locales/${name}.json`), 'utf8')
}

describe('scraper i18n keys', () => {
  test.each(['en', 'ja', 'zh-TW'])('%s includes new sync-preflight keys', (locale) => {
    const source = readLocale(locale)

    expect(source).toContain('"storageTitle"')
    expect(source).toContain('"dataDirectory"')
    expect(source).toContain('"storageDescription"')
    expect(source).toContain('"categoryNone"')
    expect(source).toContain('"categoryAnimal"')
    expect(source).toContain('"categoryNature"')
    expect(source).toContain('"categoryCooking"')
    expect(source).toContain('"categoryTravel"')
    expect(source).toContain('"categoryVehicle"')
    expect(source).toContain('"categorySports"')
    expect(source).toContain('"categorySocial"')
    expect(source).toContain('"categoryTechnical"')
    expect(source).toContain('"categoryLecture"')
    expect(source).toContain('"categoryRadio"')
    expect(source).toContain('"updateAvailableTitle"')
    expect(source).toContain('"emptyDatabaseTitle"')
    expect(source).toContain('"confirmClearReplace"')
    expect(source).toContain('"estimatedVideos"')
    expect(source).toContain('"estimatedDatabaseSize"')
    expect(source).toContain('"availableDiskSpace"')
    expect(source).toContain('"insufficientStorageTitle"')
    expect(source).toContain('"insufficientStorageMessage"')
    expect(source).toContain('"estimateUnavailable"')
    expect(source).toContain('"diskSpaceUnavailable"')
  })
})
