# AGENTS.md

本文件為 AI 代理提供專案指南。請在執行任務前詳閱。

## 工作目錄

**重要：** 主專案位於 `vocaloid-search-desktop/` 子目錄。所有 npm/cargo 命令必須在此目錄執行。

```bash
cd vocaloid-search-desktop
```

## 建構命令

### 前端 (Vue 3 + Vite + TypeScript)

```bash
cd vocaloid-search-desktop

# 開發模式（注意：Niconico 嵌入播放器在 localhost 下無法運作）
npm run dev

# 類型檢查
npx vue-tsc --noEmit

# 建構前端
npm run build

# 執行測試
npm run test

# 執行單一測試檔案
npx vitest run src/features/playlistViews/searchViewInteractions.test.ts

# 執行特定測試名稱
npx vitest run -t "restores saved playlist item"
```

### 後端 (Rust + Tauri)

```bash
cd vocaloid-search-desktop

# 檢查 Rust 程式碼
cd src-tauri && cargo check

# 執行 Rust 測試
cd src-tauri && cargo test

# 執行單一測試
cd src-tauri && cargo test accepts_search_load_more

# Clippy lint
cd src-tauri && cargo clippy

# 建構完整應用程式（正式版）
npm run tauri build

# 建構完整應用程式（除錯版，較快）
npm run tauri build -- --debug
```

## 專案架構

```
vocaloid-search-desktop/
├── src/                    # Vue 3 前端
│   ├── api/               # Tauri 命令封裝
│   ├── composables/       # Vue composables (共享邏輯)
│   ├── features/          # 功能模組
│   │   └── playlistViews/ # 播放清單視圖邏輯與測試
│   ├── stores/            # Pinia stores
│   ├── views/             # Vue 路由視圖
│   └── main.ts            # 主入口點
├── src-tauri/             # Rust 後端
│   └── src/
│       ├── commands.rs    # Tauri 命令處理
│       ├── state.rs       # 應用程式狀態管理
│       ├── models.rs      # 資料模型
│       ├── database.rs    # SQLite 操作
│       └── scraper.rs     # Niconico API 抓取器
└── vite.config.ts         # Vite 配置（含測試）
```

## 程式碼風格指南

### TypeScript/Vue

**匯入順序：**
1. Node.js 內建模組
2. 第三方套件
3. 專案內部模組（使用 `@/` 別名）

```typescript
import { readFileSync } from 'node:fs'
import { describe, expect, test } from 'vitest'
import type { Video } from '../../api/tauri-commands'
import { toggleSortDirection } from './searchViewInteractions'
```

**命名規範：**
- 檔案：`kebab-case.ts` 或 `PascalCase.vue`（元件）
- 函數/變數：`camelCase`
- 類型/介面：`PascalCase`
- 常數：`SCREAMING_SNAKE_CASE` 或 `camelCase`（視語意而定）
- Vue 元件：`PascalCase.vue`
- 測試檔案：`*.test.ts`

**類型定義：**
- 使用 `interface` 定義物件結構
- 使用 `type` 定義聯合類型或工具類型
- 明確標註 `| null` 表示可為空值

```typescript
export interface Video {
  id: string
  title: string
  thumbnail_url: string | null
  is_watched: boolean
}

export type PlaylistType = 'Search' | 'History' | 'WatchLater'
```

**錯誤處理：**
- API 呼叫使用 `try/catch`
- Tauri 命令返回 `Result<T, String>`
- 前端使用 `Result<T, String>` 處理錯誤

### Rust

**匯入順序：**
1. 標準庫
2. 外部 crate
3. 內部模組

```rust
use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::models::{Video, ListContextId};
```

**命名規範：**
- 函數/變數：`snake_case`
- 類型/結構體：`PascalCase`
- 常數：`SCREAMING_SNAKE_CASE`
- 模組：`snake_case`

**錯誤處理：**
- 使用 `Result<T, String>` 作為 Tauri 命令返回值
- 使用 `Option<T>` 表示可空值
- 錯誤訊息使用英文

```rust
#[tauri::command]
pub async fn get_video(video_id: String) -> Result<Option<Video>, String> {
    // ...
}
```

**狀態管理：**
- 使用 `Arc<RwLock<T>>` 共享狀態
- 讀取用 `.read()`，寫入用 `.write()`
- 避免長時間持有鎖

## 測試規範

### TypeScript 測試

- 測試檔案與源碼同目錄：`*.test.ts`
- 使用 Vitest：`describe`, `test`, `expect`
- 測試名稱使用英文描述

```typescript
describe('playlistViewState shared logic', () => {
  test('restores saved playlist item only when playlist type matches', () => {
    expect(result).toEqual({ selectedIndex: 1, selectedVideo: results[1] })
  })
})
```

### Rust 測試

- 測試寫在 `#[cfg(test)] mod tests { ... }` 區塊
- 測試函數標註 `#[test]`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_matching_search_load_more_requests() {
        assert!(should_accept_list_load_more(
            PlaylistType::Search,
            PlaylistType::Search,
            4,
            4,
        ));
    }
}
```

## Git Commit 規範

- 使用 Conventional Commits 格式
- 常見前綴：`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`

## 重要注意事項

1. **Niconico 嵌入播放器限制**：`npm run dev` 無法測試嵌入播放器，必須使用 `tauri build` 建構完整應用程式

2. **狀態同步**：主視窗與 PiP 視窗透過 Rust AppState 同步狀態，修改時需考慮雙視窗情境

3. **版本控制**：`ListContext.version` 用於防止並發請求造成資料損壞，修改相關邏輯時請特別注意

4. **路徑別名**：前端使用 `@/*` 對應 `./src/*`

5. **測試驗證**：修改後請執行相關測試確保不破壞現有功能

## 常見陷阱與已知問題

### Niconico 嵌入播放器限制

- **問題**：Niconico 嵌入播放器僅在 `tauri://` 協議下運作
- **影響**：`npm run dev` 無法測試嵌入播放器功能
- **解決**：測試播放功能必須使用 `npm run tauri build` 或 `npm run tauri dev`

### 競態條件：load_more 與 search

- **問題**：快速切換排序/篩選時，load_more 可能使用錯誤的參數
- **根因**：`reserve_list_context_version()` 與 `finalize_list_context_search()` 之間的時間窗口
- **解決**：已透過原子更新瀏覽參數修復（2026-03-13）
- **注意**：修改 state.rs 時務必確保版本控制邏輯正確

### 狀態同步：主視窗與 PiP

- **問題**：主視窗與 PiP 需要保持狀態一致
- **解決**：Rust 作為唯一真值來源，前端不維護狀態副本
- **注意**：
  - 避免在前端維護與 Rust 衝突的狀態
  - 修改狀態相關邏輯需考慮雙視窗情境
  - 參見 `playlist-context-management` 規格

### ListContext 版本控制

- **目的**：防止並發請求造成資料損壞
- **行為**：
  1. search 呼叫 `reserve_version()` → 版本 +1，清除 items
  2. API 回傳後呼叫 `finalize()` → 寫入結果
  3. load_more 需驗證版本號一致才會執行
- **注意**：修改 `state.rs` 中的 `reserve_list_context_version()` 需確保原子更新所有相關參數

### CSP 配置

- **位置**：`vocaloid-search-desktop/src-tauri/tauri.conf.json`
- **關鍵設定**：
  - `frame-src` 必須包含 `https://embed.nicovideo.jp`
  - `connect-src` 必須包含 `https://embed.nicovideo.jp`
  - `img-src` 必須包含 Niconico 圖片 CDN 網域

## 關鍵架構決策

### 為什麼 Rust 統一狀態管理？(2026-03-06)

**問題**：狀態分散在 Vue 前端與 Rust 後端，主視窗與 PiP 視窗同步問題不斷。每次修復一個同步 bug，又會發現新的邊緣案例。

**根因**：「狀態有兩份副本需要同步」本身就是問題所在。

**決策**：Rust 成為唯一真值來源（Single Source of Truth），前端（主視窗與 PiP）變成純顯示層。

**效益**：
- 徹底解決同步問題
- PiP 可獨立運作，不依賴主視窗
- 更容易除錯（狀態全部在 Rust）
- 更容易擴展（未來新增視窗不需要同步邏輯）

**參見**：`openspec/changes/archive/2026-03-06-unified-rust-state/`

---

### 為什麼使用 ListContext 版本控制？(2026-03-12)

**問題**：browsing state、playback state、paginated result updates 可以重疊而沒有嚴格的身份或失效模型。這導致排序、還原視圖、載入更多結果可能混合不同查詢狀態的結果。

**決策**：每個 list（Search, History, WatchLater）有自己的 context，包含：
- 唯一識別身份（ListContextId）
- 版本號（version）
- 瀏覽參數（query, sort, filters, exclude_watched）
- 載入的 items 與分頁狀態

**行為**：
- 只有 active playback reference 指向的 list 才能控制播放
- 切換視窗不會切換 playback
- 版本不一致的 load_more 會被拒絕

**參見**：`openspec/specs/playlist-context-management/spec.md`

---

### 為什麼統一播放器架構？(2026-03-14)

**問題**：主視窗（`PlayerColumn.vue`）與 PiP 視窗（`PipApp.vue`）有重複的播放器邏輯。這導致 bug：搜尋條件變更時，主視窗播放器重置但 PiP 播放器沒有。

**決策**：建立統一的播放器架構：
- `usePlayerCore.ts` - 共享狀態管理、事件處理、播放控制
- `UnifiedPlayer.vue` - 支援 full（主視窗）與 compact（PiP）模式
- 透過 shared composable 處理 `active-playback-cleared` 事件

**效益**：
- 單一真值來源控制播放器行為
- 未來 bug 修復自動套用到雙視窗
- 更容易新增功能（如鍵盤快捷鍵）

**參見**：`openspec/specs/unified-player-core/spec.md`

---

### 為什麼技術債削減？(2026-03-16)

**問題**：專案存在多項技術債影響可維護性：
- 排序欄位使用字串而非 enum，缺乏編譯時檢查
- 搜尋邏輯重複且缺乏測試
- 前端 `SearchView.vue` 過於龐大（1896 行）
- 生產環境存在大量 `println!` 除錯輸出

**決策**：分階段重構，Phase 1 處理安全變更：
- 新增 `SortField`/`SortDirection` enum 提升類型安全
- 建立 `useSearch`/`useSearchFilters` composables（新功能可用，舊代碼逐步遷移）
- 建立 `build_search_query()` 函數（測試用，保持向後兼容）
- 移除所有 `println!` 除錯輸出

**效益**：
- 提升類型安全，減少運行時錯誤
- 新功能可使用測試過的 composables
- 改善生產環境效能
- Phase 2 準備完成（移除 legacy 狀態欄位）

**參見**：`openspec/changes/archive/2026-03-16-reduce-technical-debt/`

---

### 為什麼移除 Legacy 狀態欄位？(2026-03-16)

**問題**：`state.rs` 同時保留新舊兩套狀態模型：
- Legacy: `playlist_type`, `search_results`, `history_results`, `watch_later_results`, `playlist_index`
- New: `list_contexts`, `active_playback`

這增加了維護複雜度與同步風險。

**決策**：移除所有 legacy 欄位，統一使用 ListContext 模型：
- `playlist_index` → `active_playback.current_index`
- `*_results` → `list_contexts[id].items`
- `playlist_type` → `active_playback.list_id` (透過 `set_browsing_list()`)

**效益**：
- 降低狀態同步複雜度
- 單一真值來源
- 更容易維護與擴展

**參見**：`openspec/changes/remove-legacy-state-fields/`

### 為什麼原子更新瀏覽參數？(2026-03-13)

**問題**：`reserve_list_context_version()` 只更新版本號和清除 items，但不更新 sort、filters 等瀏覽參數。這建立了不一致狀態的時間窗口。

```
Before:
  reserve() → version=4, sort="asc"(old)
  load_more reads → sort="asc" ← WRONG!
  finalize() → sort="desc"
```

**決策**：`reserve_list_context_version()` 原子更新所有瀏覽參數（query, sort, filters, exclude_watched, formula_filter）。

```
After:
  reserve() → version=4, sort="desc"(new) ← ATOMIC
  load_more reads → sort="desc" ← CORRECT!
  finalize() → sort="desc"(idempotent)
```

**參見**：`openspec/changes/archive/2026-03-13-fix-load-more-race-condition/`

---

### 為什麼 Search 播放需要凍結 watched 邊界？(2026-03-18)

**問題**：當 `exclude watched` 啟用時，Search 分頁每次請求都會重新評估最新的 history 狀態。這導致播放 session 內的 membership 漂移，造成分頁跳號或影片突然消失。

**根因**：`exclude watched` 的語意是「排除目前為止已看過的影片」，但「目前為止」在播放 session 內會隨著新標記的 watched 影片而改變。

**決策**：引入 Search playback snapshot metadata：
- 建立 immutable first-watch sequence（`first_watched_seq`）追蹤首次觀看順序
- Search 播放開始時凍結當下的 `MAX(first_watched_seq)` 作為邊界
- 同一 Search session 內的分頁只排除凍結邊界內的 watched 影片
- 新標記的 watched 影片只更新 UI badge，不改變分頁 membership

**效益**：
- Search 播放 session 內的 membership 維持穩定
- 手動捲動與 PiP 連播行為一致
- History 與 Watch Later 不受影響（不使用 snapshot boundary）

**參見**：`openspec/changes/archive/2026-03-18-stabilize-search-playback-snapshot/`

## OpenSpec 規格驅動開發

本專案使用 OpenSpec 進行規格驅動開發，所有功能變更都應遵循 OpenSpec 工作流程。

### 目錄結構

```
openspec/
├── config.yaml              # OpenSpec 配置
├── specs/                   # 能力規格（當前版本）
│   └── <capability>/        # 能力名稱（kebab-case）
│       └── spec.md          # 規格文件
└── changes/                 # 變更目錄
    ├── <YYYY-MM-DD>-<name>/ # 活躍變更
    │   ├── proposal.md      # 變更提案
    │   ├── design.md        # 設計決策
    │   ├── tasks.md         # 任務清單
    │   └── specs/           # 受影響規格快照（歸檔時加入）
    └── archive/             # 已歸檔變更
        └── <YYYY-MM-DD>-<name>/
            ├── .openspec.yaml    # 變更元數據
            ├── proposal.md
            ├── design.md
            ├── tasks.md
             └── specs/            # 完成時的規格快照
```

**注意：** `openspec/` 目錄不在 Git 版本控制中（已加入 `.gitignore`），因為這些是開發過程的工作文件，不需要保留在版本歷史中。

### 變更工作流程

#### 1. 建立新變更

當需要進行功能新增、重構或重大修復時：

1. 在 `openspec/changes/` 下建立目錄：`<YYYY-MM-DD>-<change-name>`
2. 撰寫 `proposal.md`：說明 Why、What Changes、Capabilities、Impact
3. 撰寫 `design.md`：詳細設計決策、程式碼範例、測試計畫
4. 撰寫 `tasks.md`：實作任務清單（使用 `- [ ]` 和 `- [x]`）

#### 2. 更新變更

- 實作過程中更新 `tasks.md` 的任務狀態
- 如設計有變更，同步更新 `design.md`

#### 3. 歸檔變更

完成實作並驗證後：

1. 在變更目錄下建立 `specs/` 子目錄
2. 複製受影響的當前規格到 `specs/` 作為快照
3. 建立 `.openspec.yaml` 元數據檔案
4. 將整個變更目錄移至 `openspec/changes/archive/`

#### 4. Delta Sync

比較歸檔規格與當前規格：

- **歸檔規格**：`openspec/changes/archive/<change>/specs/` - 歷史快照
- **當前規格**：`openspec/specs/` - 正式版本

規格可能已演進，歸檔快照保留當時狀態供追溯。

### Proposal 格式

```markdown
## Why
變更動機與問題描述

## What Changes
- 具體變更內容
- 受影響模組

## Capabilities
### New Capabilities
- `<capability-name>`: 描述

### Modified Capabilities
- `<capability-name>`: 變更說明

## Impact
- 受影響檔案
- 風險評估
```

### 規格文件格式

規格使用 BDD 風格撰寫：

```markdown
# <capability-name>

## ADDED Requirements

### Requirement: <需求標題>
描述...

#### Scenario: <場景名稱>
- **WHEN** 觸發條件
- **THEN** 預期結果
- **AND** 額外條件
```

### 過往重要變更

| 日期 | 變更名稱 | 說明 |
|------|----------|------|
| 2026-03-18 | stabilize-search-playback-snapshot | Search 播放 session 凍結 watched 邊界，確保分頁 membership 穩定 |
| 2026-03-17 | fix-search-pagination-stability | 修復搜尋分頁排序穩定性，加入 ORDER BY tie-breaker |
| 2026-03-16 | fix-load-more-frontend-race-condition | 修復前端 loadMore 競態條件，增加 loading 檢查與後端同步 |
| 2026-03-16 | remove-legacy-state-fields | 移除舊版狀態欄位，統一使用 ListContext 模型 |
| 2026-03-16 | resolve-technical-debt | 技術債削減 Phase 1：類型安全、composables、測試 |
| 2026-03-14 | unify-player-architecture | 統一播放器架構，消除主視窗與 PiP 重複邏輯 |
| 2026-03-13 | fix-load-more-race-condition | 修復排序/篩選切換時的 load_more 競態條件 |
| 2026-03-12 | harden-playlist-state-management | 強化播放清單狀態管理，引入 ListContext 版本控制 |
| 2026-03-06 | unified-rust-state | Rust 統一狀態管理，前端改為純顯示層 |
### 關鍵能力規格

- `playlist-context-management` - 播放清單上下文管理與版本控制
- `unified-player-core` - 統一播放器核心邏輯
- `rust-state-manager` - Rust 端統一狀態管理
- `playlist-state-sync` - 播放清單狀態同步

## 相關文件

- [README.md](./README.md) - 完整專案說明
- [Release Notes](./README.md#release-notes) - 版本更新記錄
- [OpenSpec Archive](./openspec/changes/archive/README.md) - 歸檔變更說明
