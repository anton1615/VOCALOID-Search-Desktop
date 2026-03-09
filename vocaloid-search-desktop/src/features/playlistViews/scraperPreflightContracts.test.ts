import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'

describe('scraper preflight contracts', () => {
  test('exposes structured storage info and preflight estimate through the frontend API', () => {
    const apiPath = resolve(__dirname, '../../api/tauri-commands.ts')
    const source = readFileSync(apiPath, 'utf8')

    expect(source).toContain('export interface StorageInfo')
    expect(source).toContain('data_directory: string')
    expect(source).toContain('database_size_kb: number | null')
    expect(source).toContain('export interface SyncPreflightEstimate')
    expect(source).toContain('estimated_video_count: number | null')
    expect(source).toContain('estimated_database_size_kb: number | null')
    expect(source).toContain('free_space_kb: number | null')
    expect(source).toContain('getStorageInfo: async (): Promise<StorageInfo> =>')
    expect(source).toContain("return invoke('get_storage_info')")
    expect(source).toContain('getSyncPreflightEstimate: async (): Promise<SyncPreflightEstimate> =>')
    expect(source).toContain("return invoke('get_sync_preflight_estimate')")
  })

  test('registers storage info and preflight estimate tauri commands', () => {
    const libPath = resolve(__dirname, '../../../src-tauri/src/lib.rs')
    const source = readFileSync(libPath, 'utf8')

    expect(source).toContain('commands::get_storage_info')
    expect(source).toContain('commands::get_sync_preflight_estimate')
  })
})
