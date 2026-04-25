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

  test('wires the dialog plugin for page-local file selection', () => {
    const packageJsonPath = resolve(__dirname, '../../../package.json')
    const cargoTomlPath = resolve(__dirname, '../../../src-tauri/Cargo.toml')
    const libPath = resolve(__dirname, '../../../src-tauri/src/lib.rs')
    const packageJson = readFileSync(packageJsonPath, 'utf8')
    const cargoToml = readFileSync(cargoTomlPath, 'utf8')
    const libSource = readFileSync(libPath, 'utf8')

    expect(packageJson).toContain('"@tauri-apps/plugin-dialog": "^2.2.0"')
    expect(cargoToml).toContain('tauri-plugin-dialog = "2"')
    expect(libSource).toContain('.plugin(tauri_plugin_dialog::init())')
    expect(libSource).toContain('.plugin(tauri_plugin_shell::init())')
  })

  test('registers storage info and preflight estimate tauri commands', () => {
    const libPath = resolve(__dirname, '../../../src-tauri/src/lib.rs')
    const source = readFileSync(libPath, 'utf8')

    expect(source).toContain('commands::get_storage_info')
    expect(source).toContain('commands::get_sync_preflight_estimate')
  })
})
