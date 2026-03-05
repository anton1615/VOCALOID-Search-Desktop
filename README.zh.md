# VOCALOID Search Desktop

一款本地優先的桌面應用程式，用於搜尋 Niconico VOCALOID 影片，擁有現代化的 Spotify 風格介面。

> **靈感來自 [ニコニコ超検索](https://gokulin.info/search/)** – 一個熱門的 Niconico 影片搜尋服務。

這是 [網頁版 VOCALOID Search](https://github.com/anton1615/VOCALOID-Search) 的桌面移植版，使用 Tauri 重新打造為本地優先的應用程式。

> **註**：本專案完全以 **vibe coding** 方式開發 – 透過 AI 輔助的迭代開發過程構建。

**[English](./README.md) | [日本語](./README.ja.md)**

---

## 截圖

![搜尋介面](./screenshot1.png)
![PiP 視窗](./screenshot2.png)

---

## 功能

- **現代化介面**：Spotify 風格 UI，播放清單式版面配置
- **亮色/暗色模式**：主題切換
- **多語言支援**：英文、日文、繁體中文
- **PiP 視窗**：彈出子母畫面播放器，保持在最上層
- **觀看紀錄**：追蹤已觀看的影片
- **本地資料庫**：將影片資料下載至本地，支援離線快速搜尋
- **自訂公式排序與篩選**：以自訂權重排序觀看數、喜歡數、收藏數、留言數
- **嵌入式播放器**：使用官方 Niconico 嵌入播放器連續播放
- **關鍵字 + 標籤搜尋**：支援標籤篩選的全文搜尋
- **無限滾動**：動態載入取代傳統分頁

---

## 網頁版與桌面版比較

| 功能 | 網頁版 | 桌面版 |
|------|--------|--------|
| **部署方式** | 自架伺服器 (NixOS/Linux) | 本地應用程式 (Windows) |
| **運行模式** | 24 小時伺服器運作 | 按需啟動 |
| **爬蟲** | 透過 systemd timer 自動執行 | 手動執行 |
| **多使用者** | 支援（可註冊帳號） | 單一使用者，本地資料 |
| **手機支援** | PWA/TWA 支援 Android | 不適用 |
| **PiP 模式** | 視窗縮小為精簡模式 | 原生置頂視窗 |
| **資料儲存** | 伺服器端資料庫 | 本地檔案系統 |
| **離線搜尋** | 需連接伺服器 | 完全本地 |
| **平台** | 僅限 NixOS/Linux | Windows（Linux 支援計劃中） |

### 該選擇哪個版本？

**適合網頁版的情況：**
- 需要 24 小時自動爬取資料
- 需要多使用者支援
- 希望透過 PWA 在手機使用
- 有 NixOS/Linux 伺服器

**適合桌面版的情況：**
- 偏好原生桌面體驗
- 需要與 OS 緊密結合的 PiP 視窗
- 不想管理伺服器
- 需要完全離線搜尋功能

---

## 技術規格

| 層級 | 技術 |
|------|------|
| **前端** | TypeScript, Vue 3, Vite, Tailwind CSS |
| **後端** | Rust (Tauri 2.x) |
| **資料庫** | SQLite (FTS5 全文搜尋) |
| **資料來源** | [Niconico Snapshot API v2](https://site.nicovideo.jp/search-api-docs/snapshot.html), [GetThumbInfo API](https://site.nicovideo.jp/search-api-docs/thumb-info.html) |

### 實作架構

```
┌─────────────────────────────────────────┐
│           Vue 3 Frontend                │
│  (Search UI, Player, Settings, PiP)     │
└─────────────────┬───────────────────────┘
                  │ Tauri IPC
┌─────────────────▼───────────────────────┐
│           Rust Backend                   │
│  (SQLite, HTTP Client, File System)     │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│    Local SQLite Database (FTS5)         │
└─────────────────────────────────────────┘
```

### 主要技術細節

- **自訂協定**：使用 Tauri 的自訂協定（`tauri://localhost`）而非 HTTP localhost，以避開 Niconico 嵌入播放器的網域限制
- **FTS5 全文搜尋**：SQLite FTS5 實現支援 AND/OR/NOT 運算子的快速關鍵字與標籤搜尋
- **公式化評分**：自訂權重的彈性排序系統

## 系統需求

- **作業系統**：Windows 10/11 (x64)
- **記憶體**：最低 4GB，建議 8GB
- **儲存空間**：應用程式約 500MB + 資料庫（大小不定）
- **網路**：影片播放與資料同步需要網路連線

## 資料儲存位置

資料庫與設定儲存於：
```
Windows: %APPDATA%\com.vocaloid-search.desktop
```

---

## 使用說明

### 使用流程

1. **同步資料庫**：開啟應用程式，進入「資料同步」頁面。點擊「開始同步」從 Niconico 下載影片資料。Niconico Snapshot API 每天約 JST 5-6 點更新，建議每天同步一次以取得最新影片資料。

2. **搜尋與瀏覽**：使用搜尋列以關鍵字搜尋影片。加入標籤篩選縮小範圍。調整排序公式以優先顯示觀看數、喜歡數、收藏數或留言數。

3. **觀看影片**：點擊影片在嵌入式播放器中播放。影片會從清單自動連續播放。

4. **PiP 模式**：點擊彈出按鈕開啟子母畫面視窗。PiP 視窗是簡化版的播放器，沒有導覽列，並保持在最上層。

---

## 從原始碼建置

### 前置需求

- Node.js 18+
- Rust 1.70+（含 `cargo`）
- Windows 10/11 SDK

### 建置步驟

```bash
# 複製儲存庫
git clone https://github.com/anton1615/VOCALOID-Search-Desktop.git
cd VOCALOID-Search-Desktop/vocaloid-search-desktop

# 安裝依賴
npm install
```

#### 開發版建置

```bash
npm run tauri dev
```

這會啟動前端開發伺服器和 Tauri 開發模式。

#### 正式版建置

```bash
# 同時建置前端和後端
npm run tauri build

# 或使用 debug 旗標建置（檔案較大但較易診斷問題）
npm run tauri build -- --debug
```

> **重要**：Tauri 使用自訂協定（`tauri://localhost`）而非 HTTP localhost 來處理嵌入播放器。這是因為 Niconico 的嵌入播放器會拒絕來自 `localhost` 網域的請求。正式版建置會正確處理這個問題；如果遇到問題，請嘗試使用 `--debug` 旗標。

建置完成的執行檔位置：
```
vocaloid-search-desktop/src-tauri/target/release/vocaloid-search-desktop.exe
```

---

## 已知問題與未來計劃

> ⚠️ **原型版本**：這是早期原型。功能可能不穩定，部分功能尚未完整測試。

### 已知問題

1. **歷史頁面**：顯示觀看紀錄但點擊影片無作用
2. **PiP ↔ 主視窗同步**：狀態同步不完整
   - PiP 中的播放不會記錄到觀看歷史
   - 開啟/關閉 PiP 時播放狀態會重置
3. **PiP 視窗**：偶爾無法關閉（原因不明）
4. **地區限制影片**：會中斷自動播放，因無法播放也無法標記為已觀看
5. **其他**：許多邊緣情況尚未測試

### 未來計劃

1. **內建「稍後觀看」**：實作類似 Niconico「あとで見る」的功能
2. **自訂播放清單**：使用者自訂影片收藏
3. **改善狀態同步**：主視窗與 PiP 間的同步優化
4. **Linux 支援**：利用 Tauri 的跨平台能力開發原生 Linux 版本

---

## 授權

[MIT License](./LICENSE)
