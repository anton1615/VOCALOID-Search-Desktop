# VOCALOID Search Desktop

**[English](#english) | [日本語](#日本語) | [中文](#中文)**

---

<a name="english"></a>
## English

A local-first desktop application for searching Niconico VOCALOID videos with a modern Spotify-like interface.

> **Inspired by [ニコニコ超検索](https://gokulin.info/search/)** – a popular Niconico video search service.

This is a desktop port of the [web-based VOCALOID Search](https://github.com/anton1615/VOCALOID-Search), rebuilt as a local-first application using Tauri.

> **Note**: This project is entirely **vibe coded** – built through iterative AI-assisted development.

### Screenshots

![Search Interface](./screenshot1.png)
![PiP Window](./screenshot2.png)

### Features

- **Modern Interface**: Spotify-inspired UI with playlist-like layout
- **Light/Dark Mode**: Toggle between themes
- **Multi-language Support**: English, Japanese, Chinese (Traditional)
- **PiP Window**: Pop out a picture-in-picture player that stays on top
- **Watch History**: Track videos you've watched
- **Local Database**: Scrape and store video data locally for fast offline search
- **Custom Formula Sorting & Filtering**: Weight videos by views, likes, mylists, and comments using your own formula
- **Embedded Player**: Continuous playback with the official Niconico embed player
- **Keyword + Tag Search**: Full-text search with tag filtering
- **Infinite Scroll**: Dynamic loading instead of fixed pagination

### Technical Specifications

| Layer | Technology |
|-------|------------|
| **Frontend** | TypeScript, Vue 3, Vite, Tailwind CSS |
| **Backend** | Rust (Tauri 2.x) |
| **Database** | SQLite with FTS5 full-text search |
| **Data Source** | [Niconico Snapshot API v2](https://site.nicovideo.jp/search-api-docs/snapshot.html) |

### Implementation Architecture

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

### System Requirements

- **OS**: Windows 10/11 (x64)
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: ~500MB for application + database size varies
- **Network**: Internet connection required for video playback and data sync

### Data Storage

Database and settings are stored at:
```
Windows: %APPDATA%\com.vocaloid-search.desktop
```

---

### User Guide

#### Usage Flow

1. **Sync Database**: Open the app and go to **Data Sync** page. Click "Start Sync" to download video data from Niconico. The Niconico Snapshot API updates daily around 5-6 AM JST, so syncing once per day is recommended.

2. **Search & Browse**: Use the search bar to find videos by keywords. Add tag filters to narrow results. Adjust sorting formula to prioritize views, likes, mylists, or comments.

3. **Watch Videos**: Click a video to play it in the embedded player. Videos auto-play continuously from the list.

4. **PiP Mode**: Click the pop-out button to open a Picture-in-Picture window. The PiP window is a simplified player view without navigation, and stays always on top.

---

### Build from Source

#### Prerequisites

- Node.js 18+
- Rust 1.70+ (with `cargo`)
- Windows 10/11 SDK (for Windows builds)

#### Build Steps

```bash
# Clone the repository
git clone https://github.com/anton1615/VOCALOID-Search-Desktop.git
cd VOCALOID-Search-Desktop/vocaloid-search-desktop

# Install dependencies
npm install

# Development mode
npm run tauri dev

# Build release
npm run tauri build
```

The built executable will be at:
```
vocaloid-search-desktop/src-tauri/target/release/vocaloid-search-desktop.exe
```

---

### Known Issues & Future Plans

> ⚠️ **Prototype Status**: This is an early prototype. Features may be unstable and untested.

#### Known Issues

1. **History Page**: Displays watch records but clicking videos does not work
2. **PiP ↔ Main Window Sync**: State synchronization is incomplete
   - Playback in PiP is not recorded to watch history
   - Playback state is reset when opening/closing PiP
3. **PiP Window**: Occasionally cannot be closed (unknown cause)
4. **Region-Locked Videos**: Interrupt auto-play; cannot be marked as watched since they fail to play
5. **Other Issues**: Many edge cases remain untested

#### Future Plans

1. **Built-in Watch Later**: Implement "Watch Later" and custom playlists similar to Niconico's あとで見る feature
2. **Custom Playlists**: User-defined video collections
3. **Improved State Sync**: Better synchronization between main window and PiP

---

### License

[MIT License](./LICENSE)

---

<a name="日本語"></a>
## 日本語

ニコニコ動画のVOCALOID動画を検索できる、モダンなSpotify風UIのローカルファースト・デスクトップアプリケーション。

> **[ニコニコ超検索](https://gokulin.info/search/) に触発されました** – 人気のニコニコ動画検索サービス。

これは [ウェブ版VOCALOID Search](https://github.com/anton1615/VOCALOID-Search) をTauriでローカルファーストのデスクトップアプリとして移植したものです。

> **注**: このプロジェクトは完全に **vibe coding** で構築されました – AIアシストによる反復開発で作成されています。

### スクリーンショット

![検索インターフェース](./screenshot1.png)
![PiPウィンドウ](./screenshot2.png)

### 機能

- **モダンなインターフェース**: Spotify風のUI、プレイリスト形式のレイアウト
- **ライト/ダークモード**: テーマ切替対応
- **多言語対応**: 英語、日本語、中国語（繁体字）
- **PiPウィンドウ**: 常に最前面に表示されるピクチャー・イン・ピクチャープレイヤー
- **視聴履歴**: 視聴した動画の記録
- **ローカルデータベース**: 動画データをローカルに保存し、オフラインで高速検索
- **カスタム数式での並べ替え・フィルタリング**: 再生数、マイリスト数、コメント数、いいね数を独自の重み付けでソート
- **埋め込みプレイヤー**: 公式ニコニコ埋め込みプレイヤーによる連続再生
- **キーワード+タグ検索**: タグフィルタ付き全文検索
- **無限スクロール**: 固定ページネーションではなく動的読み込み

### 技術仕様

| レイヤー | 技術 |
|---------|------|
| **フロントエンド** | TypeScript, Vue 3, Vite, Tailwind CSS |
| **バックエンド** | Rust (Tauri 2.x) |
| **データベース** | SQLite (FTS5 全文検索) |
| **データソース** | [ニコニコ Snapshot API v2](https://site.nicovideo.jp/search-api-docs/snapshot.html) |

### データ保存先

データベースと設定は以下の場所に保存されます：
```
Windows: %APPDATA%\com.vocaloid-search.desktop
```

---

### 使い方

1. **データ同期**: アプリを開き、「データ同期」ページへ移動。「同期開始」をクリックしてニコニコから動画データをダウンロード。ニコニコSnapshot APIは毎日JST 5-6時頃に更新されるため、1日1回の同期をおすすめします。

2. **検索・閲覧**: 検索バーでキーワード検索。タグフィルタで結果を絞り込み。ソート数式で再生数やマイリスト数などの重み付けを調整。

3. **動画視聴**: 動画をクリックすると埋め込みプレイヤーで再生。リストから連続自動再生。

4. **PiPモード**: ポップアウトボタンでPiPウィンドウを開く。ナビゲーションのない簡易プレイヤーで、常に最前面に表示。

---

### ソースからのビルド

```bash
# リポジトリをクローン
git clone https://github.com/anton1615/VOCALOID-Search-Desktop.git
cd VOCALOID-Search-Desktop/vocaloid-search-desktop

# 依存関係をインストール
npm install

# 開発モード
npm run tauri dev

# リリースビルド
npm run tauri build
```

---

### 既知の問題と今後の計画

> ⚠️ **プロトタイプ版**: 早期プロトタイプです。機能が不安定で、テストされていない部分があります。

#### 既知の問題

1. **履歴ページ**: 視聴記録は表示されますが、クリックしても動作しません
2. **PiP ↔ メインウィンドウ同期**: 状態同期が不完全
   - PiPでの再生が履歴に記録されません
   - PiPを開く/閉じる際、再生状態がリセットされます
3. **PiPウィンドウ**: まれに閉じられないことがあります（原因不明）
4. **地域制限動画**: 自動再生が中断されます。再生できないため視聴済みにも記録されません
5. **その他**: 多くのエッジケースが未テストです

#### 今後の計画

1. **「あとで見る」機能**: ニコニコの「あとで見る」のような機能を実装予定
2. **カスタムプレイリスト**: ユーザー定義の動画コレクション
3. **状態同期の改善**: メインウィンドウとPiP間の同期改善

---

### ライセンス

[MIT License](./LICENSE)

---

<a name="中文"></a>
## 中文

一款本地優先的桌面應用程式，用於搜尋 Niconico VOCALOID 影片，擁有現代化的 Spotify 風格介面。

> **靈感來自 [ニコニコ超検索](https://gokulin.info/search/)** – 一個熱門的 Niconico 影片搜尋服務。

這是 [網頁版 VOCALOID Search](https://github.com/anton1615/VOCALOID-Search) 的桌面移植版，使用 Tauri 重新打造為本地優先的應用程式。

> **註**：本專案完全以 **vibe coding** 方式開發 – 透過 AI 輔助的迭代開發過程構建。

### 截圖

![搜尋介面](./screenshot1.png)
![PiP 視窗](./screenshot2.png)

### 功能

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

### 技術規格

| 層級 | 技術 |
|------|------|
| **前端** | TypeScript, Vue 3, Vite, Tailwind CSS |
| **後端** | Rust (Tauri 2.x) |
| **資料庫** | SQLite (FTS5 全文搜尋) |
| **資料來源** | [Niconico Snapshot API v2](https://site.nicovideo.jp/search-api-docs/snapshot.html) |

### 資料儲存位置

資料庫與設定儲存於：
```
Windows: %APPDATA%\com.vocaloid-search.desktop
```

---

### 使用說明

1. **同步資料庫**：開啟應用程式，進入「資料同步」頁面。點擊「開始同步」從 Niconico 下載影片資料。Niconico Snapshot API 每天約 JST 5-6 點更新，建議每天同步一次以取得最新影片資料。

2. **搜尋與瀏覽**：使用搜尋列以關鍵字搜尋影片。加入標籤篩選縮小範圍。調整排序公式以優先顯示觀看數、喜歡數、收藏數或留言數。

3. **觀看影片**：點擊影片在嵌入式播放器中播放。影片會從清單自動連續播放。

4. **PiP 模式**：點擊彈出按鈕開啟子母畫面視窗。PiP 視窗是簡化版的播放器，沒有導覽列，並保持在最上層。

---

### 從原始碼建置

```bash
# 複製儲存庫
git clone https://github.com/anton1615/VOCALOID-Search-Desktop.git
cd VOCALOID-Search-Desktop/vocaloid-search-desktop

# 安裝依賴
npm install

# 開發模式
npm run tauri dev

# 建置正式版
npm run tauri build
```

---

### 已知問題與未來計劃

> ⚠️ **原型版本**：這是早期原型。功能可能不穩定，部分功能尚未完整測試。

#### 已知問題

1. **歷史頁面**：顯示觀看紀錄但點擊影片無作用
2. **PiP ↔ 主視窗同步**：狀態同步不完整
   - PiP 中的播放不會記錄到觀看歷史
   - 開啟/關閉 PiP 時播放狀態會重置
3. **PiP 視窗**：偶爾無法關閉（原因不明）
4. **地區限制影片**：會中斷自動播放，因無法播放也無法標記為已觀看
5. **其他**：許多邊緣情況尚未測試

#### 未來計劃

1. **內建「稍後觀看」**：實作類似 Niconico「あとで見る」的功能
2. **自訂播放清單**：使用者自訂影片收藏
3. **改善狀態同步**：主視窗與 PiP 間的同步優化

---

### 授權

[MIT License](./LICENSE)
