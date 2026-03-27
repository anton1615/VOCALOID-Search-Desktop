<p align="center">
  <img src="./icon.png" alt="VOCALOID Search Desktop icon" width="128" height="128">
</p>

<h1 align="center">VOCALOID Search Desktop</h1>

<p align="center">
  🎵 一款專注於搜尋與播放 Niconico VOCALOID 影片、具備 Spotify 風格介面的本地優先桌面應用程式。
</p>

<p align="center">
  <strong>Windows 原生 • Tauri + Rust 後端 • Vue 3 前端 • SQLite FTS5 搜尋</strong>
</p>

<p align="center">
  <a href="https://www.microsoft.com/windows"><img alt="Platform" src="https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4"></a>
  <a href="https://tauri.app/"><img alt="Tauri" src="https://img.shields.io/badge/Tauri-2.x-24C8DB"></a>
  <a href="https://vuejs.org/"><img alt="Vue" src="https://img.shields.io/badge/Vue-3-42B883"></a>
  <a href="https://www.rust-lang.org/"><img alt="Rust" src="https://img.shields.io/badge/Rust-backend-000000"></a>
  <a href="https://www.sqlite.org/fts5.html"><img alt="Database" src="https://img.shields.io/badge/SQLite-FTS5-003B57"></a>
</p>

<p align="center">
  <a href="./README.md">English</a> · <a href="./README.ja.md">日本語</a>
</p>

> **靈感來自 [ニコニコ超検索](https://gokulin.info/search/)** – 一個熱門的 Niconico 影片搜尋服務。
>
> 這個專案是將 [網頁版 VOCALOID Search](https://github.com/anton1615/VOCALOID-Search) 以 Tauri 改寫成更偏向本地優先的桌面應用程式。

> [!TIP]
> **最適合這類使用者：** 想要使用 Windows 原生 app，而不是自架 web stack；希望同時擁有本地搜尋、整合式 PiP 播放、Watch Later，以及由 Rust 主導的 authoritative 播放狀態管理。

## 📑 目錄

- [⚡ 快速開始](#-快速開始)
- [✨ 為什麼要做這個桌面版](#-為什麼要做這個桌面版)
- [📸 截圖](#-截圖)
- [🚀 核心功能](#-核心功能)
- [🧭 架構亮點](#-架構亮點)
- [🧩 網頁版與桌面版比較](#-網頁版與桌面版比較)
- [🛠️ 技術規格](#️-技術規格)
- [💻 系統需求](#-系統需求)
- [🗂️ 資料儲存位置](#️-資料儲存位置)
- [📚 使用說明](#-使用說明)
- [❓ FAQ / 疑難排解](#-faq--疑難排解)
- [🏗️ 從原始碼建置](#️-從原始碼建置)
- [⚠️ 目前限制](#️-目前限制)
- [🗺️ 未來計劃](#️-未來計劃)
- [📄 授權](#-授權)

---

## ⚡ 快速開始

如果你只是想先在本機跑起來，最短路徑如下：

1. clone 這個 repository
2. 進入 `vocaloid-search-desktop/`
3. 執行 `npm install`
4. 一般 UI 驗證可用 `npm run tauri dev`
5. 要驗證嵌入播放器實際行為時，使用 `npm run tauri build -- --debug`

> [!IMPORTANT]
> Niconico 的嵌入播放會拒絕 localhost origin，因此 `tauri dev` **不足以** 做最終播放驗證。只要你要確認真正的嵌入播放器行為，請改用 debug 或 release 的 Tauri build。

> [!NOTE]
> 這個桌面版已內建 Watch Later、由 Rust 管理的 authoritative playback state、主視窗 / PiP 共用的播放器行為，以及同步前的儲存空間 preflight 檢查。

---

## ✨ 為什麼要做這個桌面版

VOCALOID Search Desktop 是為了那些想要使用 **原生本地 app**、而不是自架網頁服務的人而設計的。

- 🎧 **Spotify 式播放流程**，搭配嵌入式 Niconico 播放器
- 🖥️ **原生 PiP 視窗**，會保持在最上層並與主視窗共享播放狀態
- 🔎 **基於 SQLite FTS5 的高速本地搜尋**
- 📚 **觀看紀錄 + Watch Later** 直接整合在桌面版裡
- 🧠 **自訂公式排序 / 篩選**，可以按自己的偏好做排名
- 💾 **本地優先的儲存模式**，同步後的搜尋與瀏覽速度更穩定

---

## 📸 截圖

![搜尋介面](./screenshot1.png)
![PiP 視窗](./screenshot2.png)

---

## 🚀 核心功能

- 🎨 **現代化介面**：Spotify 風格 UI 與播放清單式版面
- 🌗 **亮 / 暗色模式**：可依觀看環境切換主題
- 🌍 **多語言支援**：英文、日文、繁體中文
- 🪟 **PiP 視窗**：原生置頂子母畫面播放視窗
- 📜 **觀看紀錄**：追蹤已看過的影片並可直接回播
- ⏰ **Watch Later**：把影片存到稍後觀看清單
- 🙈 **排除已觀看**：將看過的影片從搜尋結果中排除
- 💾 **視窗狀態保存**：跨 session 保留視窗大小、位置、最大化狀態
- 🗃️ **本地資料庫**：將影片 metadata 存在本地以提升搜尋速度
- 🧮 **自訂公式排序 / 篩選**：自由加權觀看數、喜歡數、收藏數與留言數
- ⏭️ **自動跳過**：可選擇自動略過影片尾段
- ▶️ **嵌入式播放器**：使用官方 Niconico 嵌入播放器做連續播放
- 🏷️ **關鍵字 + 標籤搜尋**：支援標籤過濾的全文搜尋
- ♾️ **無限滾動**：動態載入取代固定分頁
- 🧪 **共享播放器邏輯**：主視窗與 PiP 使用同一套 playback event flow

---

## 🧭 架構亮點

這個專案現在已經不只是早期原型，而是建立在幾個明確架構決策之上的桌面應用。

### 1. Rust 是單一真值來源
所有 authoritative 的播放、瀏覽與 list state 都集中在 Rust 後端。Vue 前端不再維護一份彼此競爭的狀態副本，而是專注於顯示與互動。

### 2. ListContext versioning 用來抵抗競態條件
Search / History / Watch Later 各自擁有獨立的 context、identity 與 version。這能避免搜尋條件切換與 load-more 重疊時，舊結果混進新結果。

### 3. 主視窗與 PiP 共用同一個播放器模型
播放器核心邏輯已統一，兩個視窗會消費相同的 backend playback event 與 metadata update。播放相關修正通常會同時套用到兩邊。

### 4. Search 播放會凍結 watched 邊界
當 Search 在啟用「排除已觀看」的狀態下開始播放時，系統會固定該 session 的 watched 邊界，避免播放過程中影片突然從結果中消失。

### 5. 播放 metadata enrichment 已集中到 Rust
播放器會先出現，之後再由後端 authoritative update 套上更完整的 metadata。這降低了 Search / History / Watch Later 與主視窗 / PiP 之間的行為分歧。

---

## 🧩 網頁版與桌面版比較

| 功能 | 網頁版 | 桌面版 |
|------|--------|--------|
| **部署方式** | 自架伺服器 (NixOS/Linux) | 本地應用程式 (Windows) |
| **運行模式** | 24 小時伺服器運作 | 按需啟動 |
| **爬蟲** | 透過 systemd timer 自動執行 | 以 preflight 確認輔助的手動執行 |
| **多使用者** | 支援（可註冊帳號） | 單一使用者，本地資料 |
| **手機支援** | PWA/TWA 支援 Android | 不適用 |
| **PiP 模式** | 視窗縮小為精簡模式 | 原生置頂視窗 |
| **資料儲存** | 伺服器端資料庫 | 本地檔案系統 |
| **離線搜尋** | 需連接伺服器 | 同步後可完全本地搜尋 |
| **平台** | 僅限 NixOS/Linux | Windows（Linux 支援規劃中） |

### 該選擇哪個版本？

**適合網頁版的情況：**
- 需要 24 小時自動爬取資料
- 需要多使用者支援
- 希望透過 PWA 在手機使用
- 有 NixOS/Linux 伺服器可管理

**適合桌面版的情況：**
- 偏好原生桌面體驗
- 想要與 OS 緊密整合的 PiP 視窗
- 不想維護伺服器
- 想讓搜尋與播放清單瀏覽盡可能在本地高速完成

---

## 🛠️ 技術規格

| 層級 | 技術 |
|------|------|
| **前端** | TypeScript, Vue 3, Vite, Tailwind CSS |
| **後端** | Rust (Tauri 2.x) |
| **資料庫** | SQLite (FTS5 全文搜尋) |
| **資料來源** | 用於同步 / 搜尋快取的 [Niconico Snapshot API v2](https://site.nicovideo.jp/search-api-docs/snapshot.html)，以及用於單支影片播放 metadata enrichment 的 `watch/{id}?responseType=json` |

### 實作架構

```text
┌─────────────────────────────────────────┐
│           Vue 3 Frontend                │
│  (Search UI, Player, Settings, PiP)     │
└─────────────────┬───────────────────────┘
                  │ Tauri IPC
┌─────────────────▼───────────────────────┐
│            Rust Backend                  │
│ (State, SQLite, HTTP Client, File I/O)  │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│     Local SQLite Database (FTS5)        │
└─────────────────────────────────────────┘
```

### 主要技術細節

- 🔐 **自訂協定**：使用 `tauri://localhost`，避開 localhost 對嵌入播放器的限制
- 🔎 **FTS5 全文搜尋**：利用 SQLite FTS5 進行快速關鍵字 / 標籤搜尋
- 🧮 **公式化評分**：可自訂權重以彈性排序
- 🔄 **Versioned List Contexts**：Search / History / Watch Later 各自維持穩定的 context identity
- 🪄 **Unified Player Core**：主視窗與 PiP 共用播放狀態與事件處理邏輯
- 🧷 **Staged Metadata Rendering**：先顯示播放器，再由後端 enrichment 後套用 authoritative metadata

### 穩定性說明

- ✅ 前端邏輯有聚焦式 Vitest 測試覆蓋
- ✅ Rust 側也有 state / playback flow 的後端測試
- ✅ 近幾版更新的重點放在競態條件、狀態一致性與雙視窗同步，而不只是外觀調整

---

## 💻 系統需求

- **作業系統**：Windows 10/11 (x64)
- **記憶體**：最低 4GB，建議 8GB
- **儲存空間**：執行檔約 10MB。資料庫大小會隨同步範圍而變動（例如 `VOCALOID` 關鍵字、20 天、音樂分類 ≈ 40MB）
- **網路**：影片播放與資料同步需要網路連線
- **執行環境**：WebView2 Runtime 通常已預裝於 Windows 10/11

---

## 🗂️ 資料儲存位置

資料庫與設定會儲存在：

```text
Windows: %APPDATA%\com.vocaloid-search.desktop
```

### 便攜模式

如果你想使用便攜模式（把資料存進應用程式資料夾），請在執行檔同目錄建立 `data/` 資料夾：

```text
<vocaloid-search-desktop.exe 所在目錄>/data/
```

啟用便攜模式後，資料庫、設定、縮圖等資料都會寫入這個資料夾。把整個資料夾複製到另一台電腦時，也能一併保留資料。

**注意**：便攜模式與標準模式之間切換時，不會自動搬移資料。

---

## 📚 使用說明

### 使用流程

1. **啟動檢查**
   - 啟動時會先確認資料庫是否為空或過期。
   - Niconico Snapshot API 的資料通常會在 JST 5–6 點更新。

2. **同步資料庫**
   - 到 **資料同步** 頁面設定爬蟲條件。
   - 正式執行前，系統會顯示 **preflight 確認**，包含推估影片數、推估資料庫大小與可用磁碟空間。
   - 若預估資料庫大小超過可用空間，系統會阻止同步。

3. **搜尋與瀏覽**
   - 可以組合關鍵字搜尋、標籤篩選與公式化排序。
   - 啟用 **排除已觀看** 後，搜尋結果會排除已看過的影片。

4. **觀看影片**
   - 在 Search / History / Watch Later 點擊影片即可開始播放。
   - 嵌入播放器會先顯示，之後再由後端 authoritative update 補齊更完整的 metadata。

5. **使用 PiP 模式**
   - 可以把播放彈出成原生 PiP 視窗。
   - PiP 與主視窗共用同一條 authoritative playback flow。

### 爬蟲設定選項

| 選項 | 預設值 | 說明 |
|------|--------|------|
| `query` | `VOCALOID` | Niconico Snapshot API 搜尋關鍵字 |
| `max_age_days` | `365` | 僅抓取最近 N 天內的影片。留空則不限制 |
| `targets` | `tags` | 搜尋目標：`tags`、`tagsExact`、`title`、`description` 等 |
| `category_filter` | `MUSIC` | Niconico 分類篩選（MUSIC, GAME, ANIME, ENTERTAINMENT, DANCE, OTHER） |

---

## ❓ FAQ / 疑難排解

### `npm run tauri dev` 可以完整驗證嵌入播放嗎？

不行。`tauri dev` 很適合一般 UI 開發，但 Niconico 的嵌入播放會拒絕 localhost origin。若要驗證真正的嵌入播放器行為，請改用 `npm run tauri build -- --debug` 或 release build。

### 為什麼播放器會先出現，metadata 之後才補上？

這是刻意設計的。此專案採用 staged metadata rendering：先顯示嵌入播放器，再等 Rust 端 enrichment 完成後套用更完整的 metadata。

### 為什麼 Search 播放時，就算標記更多影片為已觀看，結果還是很穩定？

因為 Search 播放會對當前 playback session 凍結 watched 邊界，避免播放過程中 result membership 被動態改寫。

### 為什麼同步前還要做 preflight 檢查？

因為系統會先估算命中影片數、資料庫大小與可用磁碟空間，避免在儲存空間不足時做出具誤導性或破壞性的同步操作。

---

## 🏗️ 從原始碼建置

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

這會啟動前端開發伺服器與 Tauri 開發模式。

#### 正式版建置

```bash
# 一起建置前端與後端
npm run tauri build

# 或加上 debug 旗標（編譯較快，二進位較大）
npm run tauri build -- --debug
```

> **重要**：Tauri 會使用自訂協定（`tauri://localhost`）而不是 HTTP localhost，因為 Niconico 嵌入播放器會拒絕來自 `localhost` 的請求。
>
> **開發注意**：如果要驗證播放相關行為，光靠 localhost 開發模式不夠，仍需重新建置整個 app。`--debug` 可以跳過 Rust 最佳化，讓迭代速度更快。

建置完成後的執行檔位置：

```text
vocaloid-search-desktop/src-tauri/target/release/vocaloid-search-desktop.exe
```

---

## ⚠️ 目前限制

目前仍需要注意的限制如下：

1. **PiP 視窗**：偶爾可能無法乾淨關閉（原因尚未完全釐清）
2. **地區限制影片**：可能中斷自動播放，若播放失敗也無法標記為已觀看
3. **事件進行中的操作**：在同步或播放狀態切換等較長事件進行時切換分頁，仍可能踩到邊界情況
4. **罕見的 PiP 同步失敗**：主視窗 / PiP 的同步已大幅改善，但少數邊界案例仍可能失敗

---

## 🗺️ 未來計劃

目前仍與產品方向相符的未來項目包括：

- 🎛️ **快捷鍵控制** 播放流程
- 🌐 在播放器與 PiP 上加入 **以瀏覽器開啟** 的動線
- 🔊 提供獨立於嵌入播放器的 **全域音量控制**
- 🏷️ **可點擊標籤** 可直接回填搜尋條件
- 🔗 在嵌入播放器上方加入 **標題 / 作者連結**
- 🐧 利用 Tauri 跨平台能力推進 **Linux 支援**
- 📦 提供 **離線播放** 所需的本地下載能力
- 🗂️ 發展超越 Watch Later 的 **自訂播放清單**

---

## 📄 授權

[MIT License](./LICENSE)
