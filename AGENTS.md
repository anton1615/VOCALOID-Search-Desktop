# AGENTS.md

本文件提供 AI 代理的快速專案導覽：讓你進行開發修改時，快速理解工作目錄、專案架構、關鍵契約與驗證方式，而不被歷史細節與冗長背景汙染上下文。

若需要更完整的背景，優先看目前程式碼、測試與現行 spec；不要把這份文件當成歷史決策檔。

## 工作目錄與範圍

- Repo root：`D:\Downloads\vocaloid-search-alt`，主要放文件與專案級設定；更新 root 文件
  （`README.md`、`README.ja.md`、`README.zh.md`、root `AGENTS.md`）時在 repo root 操作；
  若工作區存在 `openspec/`，OpenSpec 指令也在 repo root 執行
- 主專案：`vocaloid-search-desktop/`；所有 `npm`、`npx`、`cargo`、`tauri` 指令都在這裡執行
- 在 `vocaloid-search-desktop/` 執行 `git status --short` 時，repo root 檔案可能顯示成 `../<file>`，這是正常現象

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

- 前端：Vue 3 + Vite + TypeScript，負責呈現與使用者互動；後端：Rust + Tauri，負責狀態、資料存取、抓取與跨視窗同步
- Rust 是播放與清單狀態的單一真值來源；前端不要額外維護會與 Rust 衝突的狀態副本

### 目前最重要的狀態模型

- `list_contexts`：Search / History / Watch Later 各自的瀏覽上下文；`active_playback`：目前播放中的清單與索引
- browsing 與 playback 已解耦：切換可見清單不應隱式改變目前播放綁定

### 核心檔案

- `src-tauri/src/state.rs` → 清單上下文、版本控制、active playback
- `src/composables/usePlayerCore.ts` → 主視窗與 PiP 共用播放器核心
- `src/composables/usePlayerEvents.ts` → 播放器事件與同步處理
- `src/views/SearchView.vue` → 搜尋頁主要 UI
- `src/views/HistoryView.vue` / `WatchLaterView.vue` → History / Watch Later 頁
- `src/views/ScraperView.vue` → Scraper sync 與 watch-data import UI

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
- 播放器 / staged metadata 改動要加跑 `usePlayerCore.test.ts`、`usePlayerEvents.test.ts`、`playerColumnLayout.test.ts`，外加 `vue-tsc --noEmit` 與 `npm run build`。

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
- active Search playback session 的 watched exclusion boundary 必須保持 frozen；不可因新 watched 狀態讓既有分頁 membership 在同一播放 session 中漂移
- Search `load more`、連續播放、PiP / 主視窗同步都要以同一個 active Search playback snapshot 為準，而不是各自讀取當下最新的 live browsing state

### 5. staged metadata / player update

- Search / History / Watch Later 的播放區要先顯示嵌入播放器，再等待 Rust enrichment 後更新 metadata
- 不要在前端先用 `uploader_id` 之類的暫時值當過渡 UI
- `playback-video-updated` 是獨立於 `video-selected` 的 metadata refresh；只有 playlist type、version、index、video id 全匹配時才應套用

### 6. Search restore 與 route reset

- Search restore 不可只因 `results.length === 0` 就視為需要重做 initial search
- persisted empty-result、query、sort、filters、pagination 都可能是有效的 browsing state
- 進入 `/scraper` 時需要做 playback reset，但不能順手清掉 Search / History / Watch Later 的 browsing state

### 7. Metadata panel 與 PiP 佈局

- `VideoMetaPanel` 的 description toggle 以實際 rendered overflow 決定，不能靠固定字數門檻
- 量測要在 mount / video 變更後重新執行；寬度變化用 `ResizeObserver` 維持主視窗與 PiP 一致
- PiP compact header 問題先分辨責任層：`VideoMetaPanel`、`UnifiedPlayer` shell、或 PiP section stack，不要盲改樣式

### 8. Watch Later remove confirm

- 只有 `WatchLaterView` 列表卡右側 `✕` 需要確認框
- `WatchLaterButton` 的 heart toggle 在主視窗與 PiP 仍維持即時 add/remove
- 確認框按鈕樣式要在 `WatchLaterView` 內有明確 class，不要依賴不存在的通用按鈕 class

### 9. Single-video metadata source

- Search 播放時，shared fields 以 `videos.db` 為主，只額外補 `description` 與 `uploader_name`
- History / Watch Later 播放時，共用 metadata 由 watch JSON 提供；成功 enrichment 必須從 `video_id` 衍生 canonical `watch_url`，供共享 `VideoMetaPanel` 顯示網址複製列
- enrichment 失敗時保留 placeholder；不可為缺少的 metadata 偽造 `watch_url`
- upload date 不可 fallback 成 watched / added timestamp

### 10. Cross-list same-id playback session boundary

- `Search`、`History`、`WatchLater` 之間若明確點選到相同 `video.id`，仍要視為新的 playback session
- active playback identity 必須保留 list context（playlist type、playlist version、index），不能只用 `video.id` 判斷是否同一 session
- 主視窗與 PiP 的 next/previous 必須走 Rust authoritative `play_next` / `play_previous`，不能再用目前 browsing list 的 `set_playlist_index(currentIndex +/- 1)` 代替
- 前端播放器 session boundary 要以 authoritative playback identity 觸發；同 id 跨 list 切換時，iframe / player shell 應回到新的 pre-ready session，而不是沿用舊 media session

### 11. Watch-data import / user_data merge

- `ScraperView` 的 watch-data import 與 scraper sync 是兩條獨立流程；不要共用確認框或文案
- 匯入來源接受 desktop-compatible transfer DB（`history` / `watch_later` 契約），不要分 worker / desktop 品牌來源
- in-app import 只能 merge `history` / `watch_later`；不可直接 replace 整個 `user_data.db`，且 `config` 必須保留
- merge key 是 `video_id`；History merge 後要依 `first_watched_at`、`watched_at`、`video_id` 重算 `first_watched_seq`
- preview 與 execute 必須綁定同一份檔案內容；檔案內容或 confirmed summary 不一致時要拒絕執行
- 匯入完成後要先做 Rust authoritative state refresh，再發 `watch-data-import-complete`；主視窗、PiP、已掛載 list views 都要從 Rust 真值重新同步
- scraper sync 完成後要重新讀 Rust stats 並刷新 `check_database_freshness` 注入狀態；`/scraper` 是獨立 route，內容區需保留自己的垂直捲動，避免進度區被 main layout 截掉

### 12. Title entity normalization

- Snapshot 與 Watch API 的 title 在 Rust ingress 邊界只解碼 HTML entities 一次
- 啟動時以有持久 migration marker 的 transaction 正規化既有 `videos.title`、`history.title`、`watch_later.title`；不可每次啟動重複解碼 `&amp;amp;`
- 標題一律以 Vue text interpolation 顯示，不用 `v-html`

### 13. Embedded player readiness

- iframe message listener 必須在初始 iframe render 前安裝；unmount 時移除
- `playerReady` / `isPlaying` 屬於當前 iframe session；相同 authoritative identity 的重複 props/event 同步不可重設狀態
- 只有 playback identity/session 改變或 playback 清除時才重設 readiness；跨 list 同 id 仍是新 session，主視窗與 PiP 共用此契約

### 14. Search text syntax（三個 view 與同步一致）

- Search / History / Watch Later 共用同一套語法，實作在 `src-tauri/src/search_query.rs`：
  空白 = AND、`OR` 前後需空白、`"..."` = 片語、`-詞` = 排除（`-` 與詞之間不可有空白，
  `- 詞` 為字面）、`*` = 全部（只限本地三個清單；只能是獨立 token，`"*"` 仍為字面）
- 只有負向詞的查詢匹配 0 筆（與 snapshot API 一致，顯示為 `0`），不可當成「排除該詞後全選」
- `snapshot_query_issue` 守住同步路徑：使用 `*` 或只有排除詞的 sync 必須在清空 `videos.db` 前被拒絕（回 `wildcard` / `only_excluded`）
- 詞比對用 `LIKE ... ESCAPE '\'`，`%`/`_`/`\` 由 `like_pattern` 轉義；videos 比對 `title` 與 `tags`，History / Watch Later 只比對 `title`
- History / Watch Later 的 `get_history` / `get_watch_later` 必須同時過濾 count（`get_history_count` / `get_watch_later_count`），否則 `total` / `has_next` 不一致
- 不要加回 FTS：`video_fts` 表與 trigger 已從 schema 移除，既有資料庫由 `init_db` 的 `drop_legacy_video_fts` 清掉（README 也不應宣稱 FTS5）

### 15. videos.db storage / auto_vacuum

- `videos.db` 使用 `auto_vacuum=FULL`（`init_db` → `ensure_cache_auto_vacuum`）是有意的：同步縮小範圍後檔案跟著截斷，不停在歷史高水位
- 既有 DB 需要一次性 `VACUUM` 轉換；該次會先檢查可用空間，不足就跳過（下次啟動再試），失敗是安全的；FULL 的縮檔是就地搬頁，不需要暫存檔，同步期間不會出現 2 倍峰值
- `user_data.db` 維持預設（沒有大量刪除，開啟只是多餘負擔）
- 同步大小預估（`scraper_preflight::estimate_database_size_kb`）以 `(page_count − freelist_count) × page_size ÷ rows` 為基準，再取 `max(需要量, 目前檔案大小)`；FTS 移除與 VACUUM 的結果都會自動反映

### 16. 同步失敗回報與工作列進度

- snapshot API 請求失敗（網路錯誤、非 2xx、JSON 解析失敗）一律回 `Err`、**不重試**：
  `scraper.rs` 的 `SnapshotRequestError` → `run_scraper` 把 `progress.status` 設為
  `error: <訊息>`，`ScraperView` 顯示 `syncFailedTitle` 警告。
- 例外：offset 已達上限時的 `400` 視為正常換窗結束，不算失敗。
- `run_scraper` 同步更新工作列進度：`Indeterminate` → 取得 `totalCount` 後 `Normal` + 百分比 → 任何結束（完成／取消／失敗）都清除。
- 失敗時資料庫會停在「已清空」狀態（`clear_videos` 在抓取前執行），UI 必須讓使用者看得出來，不可靜默當成完成。

### 17. Uploader blacklist

- 只在 SearchView 的本地查詢層過濾（`commands.rs` 的 `blocked_uploader_clause`，三處 SQL 組裝點：
  `search`、`execute_search`、test-only `build_search_query`）；不做在 sync/scraper 層
  （snapshot API 沒有 `userId` filter/target），也不減少 `videos.db` 容量與下載量、不刪除既有資料
- 過濾子句必須是 `(v.uploader_id IS NULL OR v.uploader_id NOT IN (...))`（`uploader_id` 可為 NULL）；list 與 count 共用同一組 `where_clauses`
- 黑名單存 `user_data.db` 的 `uploader_blacklist`（`uploader_id` TEXT 主鍵、`display_name` 選填、`added_at`），冪等 upsert；比對一律用字串（snapshot 的 `userId` 是 JSON number，經 `deserialize_user_id` 正規化）
- 名稱→id 只能走 nvapi（`/v1/search/user`、`/v1/users/{id}`，需 `X-Frontend-Id: 6`、`X-Frontend-Version: 0`、`Referer`），因為 `videos.db` 刻意不存 `uploader_name`
- 候選頭像的 `img.nicoprofile.nimg.jp` 必須列在 `src-tauri/tauri.conf.json` 的 `img-src`（`tauriCspConfig.test.ts` 守住），否則 webview 擋圖，`UploaderAvatar` 會退成 `defaults/blank.jpg` 預設頭像
- 唯一入口是 `App.vue` nav-footer 按鈕開的全域 `UploaderBlacklistDialog.vue`（任何 route 都能開；
  候選點選即加入、移除才需確認框）；SearchView 只保留空狀態入口（黑名單非空時顯示封鎖數量），
  卡片 🚫 已移除，SearchView 內的確認框流程（`pendingBlockVideo`、`confirmBlockUploader`、
  modal markup）刻意保留但沒有 UI 入口可觸發。變更由 Rust emit `uploader-blacklist-updated` 同步；
  不影響正在播放的 playlist（只影響後續查詢）

## OpenSpec 使用原則

- 本 workspace 目前包含 `openspec/`；功能新增、重大修復、重構應遵循 OpenSpec 流程
- 先看 `openspec/specs/` 的現行規格，再看 `openspec/changes/` 與 `openspec/changes/archive/` 理解脈絡
- archive 是歷史快照，不要把 archive 內容直接當成現行規格
- spec / 程式碼 / archive 不一致時，以現行 spec 與實際程式碼為主，必要時再同步文件
- 若未來某個 checkout 沒有 `openspec/`，再退回以現行程式碼、測試與 repo 文件為準

### 目前最常先看的能力規格

- `playlist-context-management`
- `unified-player-core`
- `rust-state-manager`
- `playlist-state-sync`

## 文件維護原則

- 這份 `AGENTS.md` 只保留高價值、會直接影響日常修改判斷的資訊
- 詳細歷史背景、長篇決策說明、一次性除錯筆記，應放在 spec、設計文件、測試或其他專用文件，不要持續堆回這裡
- 若專案架構或契約明顯改變，修改功能後應順手更新這份文件，保持它是「快速導覽」而不是「歷史百科」
- 每個 `### <n>.` 契約硬上限 15 行；超出時先壓成 invariant，長篇背景放 `openspec/specs/<capability>/spec.md`（注意 `openspec/` 目前被 `.gitignore` 忽略、不進版控）

## 相關文件

- `README.md`
- `README.ja.md`
- `README.zh.md`
- `vocaloid-search-desktop/AGENTS.md`
