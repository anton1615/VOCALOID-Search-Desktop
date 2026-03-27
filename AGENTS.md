# AGENTS.md

本文件提供 AI 代理的快速專案導覽。目標是讓你在進行開發修改時，
能快速理解工作目錄、專案架構、關鍵契約與驗證方式，而不被歷史細節
與冗長背景汙染上下文。

若需要更完整的背景，優先看目前程式碼、測試與現行 spec；不要把這份
文件當成歷史決策檔。

## 先看這裡

- Repo root：`D:\Downloads\vocaloid-search-alt`
- 主專案：`vocaloid-search-desktop/`
- 所有 `npm`、`npx`、`cargo`、`tauri` 指令都在
  `vocaloid-search-desktop/` 執行
- root 主要放文件與專案級設定；產品程式碼在
  `vocaloid-search-desktop/`

## 工作目錄規則

### Repo root

- 更新 root 文件時在 repo root 操作，例如：`README.md`、`README.ja.md`、
  `README.zh.md`、root `AGENTS.md`
- 若工作區存在 `openspec/`，OpenSpec 指令也在 repo root 執行

### App project

- 日常開發、建構、測試都在 `vocaloid-search-desktop/` 執行
- 從 `vocaloid-search-desktop/` 執行 `git status --short` 時，repo root 檔案
  可能顯示成 `../<file>`，這是正常現象

## 常用指令

以下指令預設都在 `vocaloid-search-desktop/` 執行。

### 前端

```bash
npm run dev
npx vue-tsc --noEmit
npm run build
npm run test
npx vitest run src/features/playlistViews/searchViewInteractions.test.ts
npx vitest run -t "restores saved playlist item"
```

### Rust / Tauri

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
cd src-tauri && cargo test accepts_search_load_more
cd src-tauri && cargo clippy
npm run tauri build
npm run tauri build -- --debug
```

### 播放器 / staged metadata 相關修改優先驗證

```bash
npx vitest run src/composables/usePlayerCore.test.ts src/composables/usePlayerEvents.test.ts src/features/playlistViews/playerColumnLayout.test.ts
npx vue-tsc --noEmit
npm run build
```

## 專案結構

```text
vocaloid-search-desktop/
├── src/
│   ├── api/                    # Tauri 命令封裝
│   ├── composables/            # 共用狀態與播放器邏輯
│   ├── features/playlistViews/ # 播放清單、播放器與相關測試契約
│   ├── stores/                 # Pinia stores
│   ├── views/                  # Search / History / Watch Later / Scraper
│   └── main.ts
├── src-tauri/
│   └── src/
│       ├── commands.rs         # Tauri command entry points
│       ├── state.rs            # Rust 端單一真值來源
│       ├── models.rs           # 型別與資料模型
│       ├── database.rs         # SQLite 與查詢邏輯
│       ├── scraper.rs          # Niconico 抓取
│       └── scraper_preflight.rs
└── vite.config.ts
```

## 架構速記

### 前後端責任

- 前端：Vue 3 + Vite + TypeScript，負責呈現與使用者互動
- 後端：Rust + Tauri，負責狀態、資料存取、抓取與跨視窗同步
- Rust 是播放與清單狀態的單一真值來源；前端不要額外維護會與 Rust 衝突
  的狀態副本

### 目前最重要的狀態模型

- `list_contexts`：Search / History / Watch Later 各自的瀏覽上下文
- `active_playback`：目前播放中的清單與索引
- browsing 與 playback 已解耦：切換可見清單不應隱式改變目前播放綁定

### 核心檔案

- `vocaloid-search-desktop/src-tauri/src/state.rs`
  - 清單上下文、版本控制、active playback
- `vocaloid-search-desktop/src/composables/usePlayerCore.ts`
  - 主視窗與 PiP 共用播放器核心
- `vocaloid-search-desktop/src/composables/usePlayerEvents.ts`
  - 播放器事件與同步處理
- `vocaloid-search-desktop/src/views/SearchView.vue`
  - 搜尋頁主要 UI
- `vocaloid-search-desktop/src/views/HistoryView.vue`
- `vocaloid-search-desktop/src/views/WatchLaterView.vue`
- `vocaloid-search-desktop/src/views/ScraperView.vue`

## 程式碼風格

- TypeScript / Vue：沿用現有檔案慣例；匯入順序為 Node → 第三方 → `@/`
- TypeScript：值用 `camelCase`、型別用 `PascalCase`、可空值明確寫 `| null`
- Rust：模組 / 函式 / 變數用 `snake_case`、型別用 `PascalCase`、常數用
  `SCREAMING_SNAKE_CASE`
- Tauri command 維持 `Result<T, String>`；Rust 錯誤訊息使用英文

## 測試規則

- 前端測試與原始碼同目錄，檔名 `*.test.ts`
- 使用 Vitest：`describe`、`test`、`expect`
- Rust 測試放在 `#[cfg(test)] mod tests { ... }`
- 修改後至少跑與變更直接相關的測試；若影響播放器、同步或 state contract，
  要擴大驗證範圍

## 修改時必記契約

### 1. Niconico 嵌入播放器

- 嵌入播放器只會在 `tauri://` 協議下正常工作
- `npm run dev` 可做一般前端開發，但不能拿來驗證嵌入播放器行為
- 要驗證實際播放，使用 `npm run tauri build -- --debug` 或其他 Tauri 執行流程

### 2. 播放與瀏覽分離

- `set_browsing_list()` 只改目前可見清單
- 只有使用者明確選影片時才應重綁 `active_playback`
- 非 active list 的 refresh / clear 不應誤清空播放器

### 3. ListContext 版本控制

- `ListContext.version` 用來防止並發請求混入不同查詢結果
- Search 與 load more 相關改動要特別注意 `state.rs` 中的 version 契約
- 更新 search 參數時要維持原子性，避免 load more 讀到舊參數

### 4. Search playback snapshot / watched boundary

- Search 播放啟動後，Rust 端會把當前 Search session 綁定到 playback snapshot
- active Search playback session 的 watched exclusion boundary 必須保持 frozen；
  不可因新 watched 狀態讓既有分頁 membership 在同一播放 session 中漂移
- Search `load more`、連續播放、PiP / 主視窗同步都要以同一個 active Search
  playback snapshot 為準，而不是各自讀取當下最新的 live browsing state

### 5. staged metadata / player update

- Search / History / Watch Later 的播放區要先顯示嵌入播放器，再等待 Rust
  enrichment 後更新 metadata
- 不要在前端先用 `uploader_id` 之類的暫時值當過渡 UI
- `playback-video-updated` 是獨立於 `video-selected` 的 metadata refresh；
  只有 playlist type、version、index、video id 全匹配時才應套用

### 6. Search restore 與 route reset

- Search restore 不可只因 `results.length === 0` 就視為需要重做 initial search
- persisted empty-result、query、sort、filters、pagination 都可能是有效的
  browsing state
- 進入 `/scraper` 時需要做 playback reset，但不能順手清掉
  Search / History / Watch Later 的 browsing state

### 7. Metadata panel 與 PiP 佈局

- `VideoMetaPanel` 的 description toggle 以實際 rendered overflow 決定，
  不能靠固定字數門檻
- 量測要在 mount / video 變更後重新執行；寬度變化用 `ResizeObserver`
  維持主視窗與 PiP 一致
- PiP compact header 問題先分辨責任層：`VideoMetaPanel`、`UnifiedPlayer`
  shell、或 PiP section stack，不要盲改樣式

### 8. Watch Later remove confirm

- 只有 `WatchLaterView` 列表卡右側 `✕` 需要確認框
- `WatchLaterButton` 的 heart toggle 在主視窗與 PiP 仍維持即時 add/remove
- 確認框按鈕樣式要在 `WatchLaterView` 內有明確 class，不要依賴不存在的
  通用按鈕 class

### 9. Single-video metadata source

- Search 播放時，shared fields 以 `videos.db` 為主，只額外補 `description`
  與 `uploader_name`
- History / Watch Later 播放時，共用 metadata 由 watch JSON 提供
- upload date 不可 fallback 成 watched / added timestamp

### 10. Cross-list same-id playback session boundary

- `Search`、`History`、`WatchLater` 之間若明確點選到相同 `video.id`，仍要視為新的 playback session
- active playback identity 必須保留 list context（playlist type、playlist version、index），不能只用 `video.id` 判斷是否同一 session
- 主視窗與 PiP 的 next/previous 必須走 Rust authoritative `play_next` / `play_previous`，不能再用目前 browsing list 的 `set_playlist_index(currentIndex +/- 1)` 代替
- 前端播放器 session boundary 要以 authoritative playback identity 觸發；同 id 跨 list 切換時，iframe / player shell 應回到新的 pre-ready session，而不是沿用舊 media session

## OpenSpec 使用原則

- 本 workspace 目前包含 `openspec/`；功能新增、重大修復、重構應遵循
  OpenSpec 流程
- 先看 `openspec/specs/` 的現行規格，再看 `openspec/changes/` 與
  `openspec/changes/archive/` 理解脈絡
- archive 是歷史快照，不要把 archive 內容直接當成現行規格
- spec / 程式碼 / archive 不一致時，以現行 spec 與實際程式碼為主，必要時再
  同步文件
- 若未來某個 checkout 沒有 `openspec/`，再退回以現行程式碼、測試與 repo
  文件為準

### 目前最常先看的能力規格

- `playlist-context-management`
- `unified-player-core`
- `rust-state-manager`
- `playlist-state-sync`

## 文件維護原則

- 這份 `AGENTS.md` 只保留高價值、會直接影響日常修改判斷的資訊
- 詳細歷史背景、長篇決策說明、一次性除錯筆記，應放在 spec、設計文件、
  測試或其他專用文件，不要持續堆回這裡
- 若專案架構或契約明顯改變，修改功能後應順手更新這份文件，保持它是
  「快速導覽」而不是「歷史百科」

## 相關文件

- `README.md`
- `README.ja.md`
- `README.zh.md`
- `vocaloid-search-desktop/AGENTS.md`
