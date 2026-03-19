<p align="center">
  <img src="./icon.png" alt="VOCALOID Search Desktop icon" width="128" height="128">
</p>

<h1 align="center">VOCALOID Search Desktop</h1>

<p align="center">
  🎵 ニコニコ VOCALOID 動画の検索と再生に特化した、Spotify ライクなローカルファーストのデスクトップアプリ。
</p>

<p align="center">
  <strong>Windows ネイティブ • Tauri + Rust バックエンド • Vue 3 フロントエンド • SQLite FTS5 検索</strong>
</p>

<p align="center">
  <a href="https://www.microsoft.com/windows"><img alt="Platform" src="https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4"></a>
  <a href="https://tauri.app/"><img alt="Tauri" src="https://img.shields.io/badge/Tauri-2.x-24C8DB"></a>
  <a href="https://vuejs.org/"><img alt="Vue" src="https://img.shields.io/badge/Vue-3-42B883"></a>
  <a href="https://www.rust-lang.org/"><img alt="Rust" src="https://img.shields.io/badge/Rust-backend-000000"></a>
  <a href="https://www.sqlite.org/fts5.html"><img alt="Database" src="https://img.shields.io/badge/SQLite-FTS5-003B57"></a>
</p>

<p align="center">
  <a href="./README.md">English</a> · <a href="./README.zh.md">中文</a>
</p>

> **[ニコニコ超検索](https://gokulin.info/search/) に触発されました** – 人気のニコニコ動画検索サービス。
>
> このプロジェクトは、[ウェブ版 VOCALOID Search](https://github.com/anton1615/VOCALOID-Search) を Tauri でローカルファーストなデスクトップアプリへ移植したものです。

> [!TIP]
> **こんな人に向いています:** セルフホスト構成ではなく、ローカル検索・統合 PiP 再生・Watch Later・Rust 主導の再生状態管理を備えた Windows ネイティブアプリを使いたい人。

## 📑 目次

- [⚡ クイックスタート](#-クイックスタート)
- [✨ このデスクトップ版の狙い](#-このデスクトップ版の狙い)
- [📸 スクリーンショット](#-スクリーンショット)
- [🚀 主な機能](#-主な機能)
- [🧭 アーキテクチャの見どころ](#-アーキテクチャの見どころ)
- [🧩 ウェブ版とデスクトップ版の比較](#-ウェブ版とデスクトップ版の比較)
- [🛠️ 技術仕様](#️-技術仕様)
- [💻 動作環境](#-動作環境)
- [🗂️ データ保存先](#️-データ保存先)
- [📚 使い方](#-使い方)
- [❓ FAQ / トラブルシューティング](#-faq--トラブルシューティング)
- [🏗️ ソースからのビルド](#️-ソースからのビルド)
- [⚠️ 現在の制約](#️-現在の制約)
- [🗺️ 今後の計画](#️-今後の計画)
- [📄 ライセンス](#-ライセンス)

---

## ⚡ クイックスタート

とりあえずローカルで試したい場合は、次の流れが最短です。

1. リポジトリを clone する
2. `vocaloid-search-desktop/` に入る
3. `npm install` で依存関係を入れる
4. UI の確認は `npm run tauri dev` を使う
5. 埋め込みプレイヤー挙動の確認は `npm run tauri build -- --debug` を使う

> [!IMPORTANT]
> ニコニコの埋め込み再生は localhost origin を拒否するため、`tauri dev` だけでは最終的な再生検証は不十分です。実際の埋め込み再生を確認する場合は、debug あるいは release の Tauri ビルドを使ってください。

> [!NOTE]
> このデスクトップ版には、Watch Later、Rust による authoritative な再生状態管理、メインウィンドウ / PiP 共通のプレイヤー挙動、同期前のストレージ preflight チェックがすでに組み込まれています。

---

## ✨ このデスクトップ版の狙い

VOCALOID Search Desktop は、セルフホスト型ウェブサービスではなく **ネイティブなローカルアプリ** を使いたい人向けに作られています。

- 🎧 **Spotify ライクな再生フロー** と埋め込みニコニコプレイヤー
- 🖥️ **ネイティブ PiP ウィンドウ** が常時最前面で再生状態を共有
- 🔎 **SQLite FTS5 ベースの高速ローカル検索**
- 📚 **視聴履歴 + Watch Later** をアプリ内に統合
- 🧠 **数式ベースの並べ替え / フィルタ** で好みの順位付けが可能
- 💾 **ローカルファーストな保存モデル** により、同期後の閲覧と検索が高速

---

## 📸 スクリーンショット

![検索インターフェース](./screenshot1.png)
![PiPウィンドウ](./screenshot2.png)

---

## 🚀 主な機能

- 🎨 **モダンなインターフェース**: Spotify 風の UI とプレイリスト式レイアウト
- 🌗 **ライト / ダークモード**: 視認性に合わせたテーマ切替
- 🌍 **多言語対応**: 英語・日本語・繁体字中国語
- 🪟 **PiP ウィンドウ**: ネイティブ常時最前面のピクチャー・イン・ピクチャー再生
- 📜 **視聴履歴**: 見た動画を追跡し、そこから再生を再開可能
- ⏰ **Watch Later**: あとで見たい動画を専用リストへ保存
- 🙈 **視聴済み除外**: 見た動画を検索結果から除外
- 💾 **ウィンドウ状態保存**: サイズ・位置・最大化状態をセッション間で保持
- 🗃️ **ローカルデータベース**: 動画 metadata をローカル保存して高速検索
- 🧮 **カスタム数式での並べ替え / フィルタ**: 再生数・いいね・マイリスト・コメント数を自由に重み付け
- ⏭️ **オートスキップ**: 動画終盤を自動でスキップ可能
- ▶️ **埋め込みプレイヤー**: 公式ニコニコ埋め込みプレイヤーによる連続再生
- 🏷️ **キーワード + タグ検索**: タグフィルタ付き全文検索
- ♾️ **無限スクロール**: 固定ページではなく動的読み込み
- 🧪 **共有プレイヤーロジック**: メインウィンドウと PiP が同じ再生イベントフローを利用

---

## 🧭 アーキテクチャの見どころ

このプロジェクトは、単なる初期プロトタイプ段階を超えて、いくつかの重要な設計方針を土台に動いています。

### 1. Rust が単一の真実の情報源
再生状態、閲覧状態、各リストの authoritative な状態は Rust バックエンドに集約されています。Vue 側は別の状態コピーを持たず、表示と操作に集中します。

### 2. ListContext versioning で競合状態を防止
Search / History / Watch Later はそれぞれ独立した context・identity・version を持ちます。これにより、検索条件変更と load-more が重なったときの stale な結果混入を防ぎます。

### 3. メインウィンドウと PiP は同じプレイヤーモデルを共有
プレイヤーコアは共通化されており、両ウィンドウが同じ backend playback event と metadata update に従います。再生バグ修正も両方へ反映されやすい構造です。

### 4. Search 再生は watched 境界を固定する
Search で「視聴済み除外」が有効なまま再生を始めると、そのセッション中は watched 境界を凍結します。これにより、再生中に動画が突然リストから消えることを防ぎます。

### 5. 再生 metadata enrichment は Rust に集約
プレイヤー本体を先に表示し、その後により豊富な metadata を backend 主導で反映します。Search / History / Watch Later / メインウィンドウ / PiP の挙動差を最小化できます。

---

## 🧩 ウェブ版とデスクトップ版の比較

| 機能 | ウェブ版 | デスクトップ版 |
|------|---------|---------------|
| **デプロイ** | セルフホストサーバー (NixOS/Linux) | ローカルアプリ (Windows) |
| **稼働** | 24時間サーバー運用 | オンデマンド起動 |
| **スクレイパー** | systemd タイマーで自動実行 | preflight 確認付きの手動実行 |
| **マルチユーザー** | 対応（ユーザー登録可能） | 単一ユーザー、ローカルデータ |
| **モバイル対応** | PWA/TWA で Android 対応 | 非対応 |
| **PiP モード** | コンパクトウィンドウにリサイズ | ネイティブ常時最前面ウィンドウ |
| **データ保存** | サーバー側データベース | ローカルファイルシステム |
| **オフライン検索** | サーバー接続が必要 | 同期後は完全ローカル |
| **プラットフォーム** | NixOS/Linux 限定 | Windows（Linux 対応予定） |

### どちらを選ぶべき？

**ウェブ版が向いている場合:**
- 24時間自動スクレイピングしたい
- マルチユーザー対応が必要
- PWA でモバイルから使いたい
- NixOS/Linux サーバーを運用している

**デスクトップ版が向いている場合:**
- ネイティブデスクトップ体験を重視したい
- OS と密結合した PiP を使いたい
- サーバー運用をしたくない
- 検索やプレイリスト閲覧をローカルで高速に完結させたい

---

## 🛠️ 技術仕様

| レイヤー | 技術 |
|---------|------|
| **フロントエンド** | TypeScript, Vue 3, Vite, Tailwind CSS |
| **バックエンド** | Rust (Tauri 2.x) |
| **データベース** | SQLite (FTS5 全文検索) |
| **データソース** | [ニコニコ Snapshot API v2](https://site.nicovideo.jp/search-api-docs/snapshot.html), [GetThumbInfo API](https://site.nicovideo.jp/search-api-docs/thumb-info.html) |

### 実装アーキテクチャ

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

### 主な技術的詳細

- 🔐 **カスタムプロトコル**: `tauri://localhost` を利用し、localhost 制約を回避
- 🔎 **FTS5 全文検索**: SQLite FTS5 による高速なキーワード / タグ検索
- 🧮 **数式ベースのスコアリング**: 柔軟な重み付けで並べ替え可能
- 🔄 **Versioned List Contexts**: Search / History / Watch Later が安定した context identity を保持
- 🪄 **Unified Player Core**: メインウィンドウと PiP 間で再生状態とイベント処理を共有
- 🧷 **Staged Metadata Rendering**: プレイヤー先行表示 + backend enrichment 後の authoritative 更新

### 信頼性について

- ✅ フロントエンドロジックは focused な Vitest テストで検証
- ✅ Rust 側でも state / playback flow のテストを実施
- ✅ 近年の更新は見た目よりも、競合状態・状態整合性・二画面一貫性の改善に重点

---

## 💻 動作環境

- **OS**: Windows 10/11 (x64)
- **メモリ**: 最低 4GB、推奨 8GB
- **ストレージ**: 実行ファイル約 10MB。データベースサイズは同期範囲次第（例：`VOCALOID` キーワード、20日、音楽カテゴリ ≈ 40MB）
- **ネットワーク**: 動画再生とデータ同期にインターネット接続が必要
- **ランタイム**: WebView2 Runtime は通常 Windows 10/11 にプリインストール済み

---

## 🗂️ データ保存先

データベースと設定は以下に保存されます。

```text
Windows: %APPDATA%\com.vocaloid-search.desktop
```

### ポータブルモード

実行ファイルと同じディレクトリに `data/` フォルダを作成すると、ポータブルモードで運用できます。

```text
<vocaloid-search-desktop.exe の場所>/data/
```

ポータブルモードが有効な場合、データベース・設定・サムネイルなどはすべてこのフォルダに保存されます。フォルダごと別 PC にコピーすれば、そのまま同じデータを利用できます。

**注意**: ポータブルモードと通常モードの切り替え時に、データは自動移行されません。

---

## 📚 使い方

### 利用の流れ

1. **起動時チェック**
   - 起動時にデータベースが空か古いかを確認します。
   - ニコニコ Snapshot API のデータは JST 5〜6 時台に更新されることが多いです。

2. **データ同期**
   - **データ同期** ページでスクレイピング設定を行います。
   - 実行前に、推定ヒット動画数・推定データベースサイズ・空きディスク容量を確認する **preflight 確認** が表示されます。
   - 推定データベースサイズが空き容量を超える場合、同期は実行されません。

3. **検索と閲覧**
   - キーワード検索、タグ絞り込み、数式ベースの並べ替えを組み合わせて使えます。
   - **視聴済み除外** を有効にすると、既に見た動画を検索結果から外せます。

4. **動画再生**
   - Search / History / Watch Later から動画をクリックすると再生が始まります。
   - 埋め込みプレイヤーが先に表示され、その後 backend 主導の metadata 更新が適用されます。

5. **PiP モード**
   - 再生をネイティブ PiP ウィンドウへポップアウトできます。
   - PiP もメインウィンドウと同じ authoritative playback flow を利用します。

### スクレイパー設定オプション

| オプション | デフォルト | 説明 |
|-----------|-----------|------|
| `query` | `VOCALOID` | ニコニコ Snapshot API の検索キーワード |
| `max_age_days` | `365` | 過去 N 日以内の動画のみ取得。空欄で無制限 |
| `targets` | `tags` | 検索対象: `tags`, `tagsExact`, `title`, `description` など |
| `category_filter` | `MUSIC` | ニコニコカテゴリフィルタ（MUSIC, GAME, ANIME, ENTERTAINMENT, DANCE, OTHER） |

---

## ❓ FAQ / トラブルシューティング

### `npm run tauri dev` だけで埋め込み再生を検証できますか？

完全にはできません。`tauri dev` は一般的な UI 作業には便利ですが、ニコニコの埋め込み再生は localhost origin を拒否します。実際の埋め込みプレイヤー挙動を確認するには、`npm run tauri build -- --debug` か release build を使ってください。

### プレイヤーが先に出て、metadata が後から更新されるのはなぜですか？

意図された挙動です。このアプリは staged metadata rendering を採用しており、埋め込みプレイヤーを先に表示し、Rust 側の enrichment 完了後により豊富な metadata を反映します。

### Search 再生中に視聴済みを増やしても、なぜ結果が安定しているのですか？

Search 再生では、アクティブな playback session に対して watched 境界を固定しています。そのため、セッション中に結果 membership が崩れにくくなっています。

### なぜ同期前に preflight 確認が必要なのですか？

同期前に、推定ヒット動画数・推定データベースサイズ・空きディスク容量を確認することで、ストレージ不足時の無駄な同期や誤解を避けるためです。

---

## 🏗️ ソースからのビルド

### 前提条件

- Node.js 18+
- Rust 1.70+（`cargo` 含む）
- Windows 10/11 SDK

### ビルド手順

```bash
# リポジトリをクローン
git clone https://github.com/anton1615/VOCALOID-Search-Desktop.git
cd VOCALOID-Search-Desktop/vocaloid-search-desktop

# 依存関係をインストール
npm install
```

#### 開発ビルド

```bash
npm run tauri dev
```

フロントエンド開発サーバーと Tauri が開発モードで起動します。

#### 本番ビルド

```bash
# フロントエンドとバックエンドを一緒にビルド
npm run tauri build

# または debug フラグ付きでビルド（ビルド高速、バイナリは大きめ）
npm run tauri build -- --debug
```

> **重要**: Tauri は埋め込みプレイヤー用に HTTP localhost の代わりにカスタムプロトコル（`tauri://localhost`）を使用します。これはニコニコ埋め込みプレイヤーが `localhost` 由来のリクエストを拒否するためです。
>
> **開発時の注意**: localhost 接続では埋め込みプレイヤーを十分に検証できないため、再生まわりの確認ではアプリ全体のビルドが必要です。`--debug` を使うと Rust 最適化を省き、反復開発が速くなります。

ビルドされた実行ファイルの場所:

```text
vocaloid-search-desktop/src-tauri/target/release/vocaloid-search-desktop.exe
```

---

## ⚠️ 現在の制約

現時点でも意識しておくべき制約は次のとおりです。

1. **PiP ウィンドウ**: まれに正常に閉じられないことがあります（原因は未特定）
2. **地域制限動画**: 自動再生を中断することがあり、再生自体に失敗すると視聴済み記録も付けられません
3. **イベント進行中の操作**: 同期や再生遷移など長めのイベント実行中にタブ切り替えを行うと、エッジケースに当たる可能性があります
4. **稀な PiP 同期失敗**: メインウィンドウ / PiP の同期は大幅に安定しましたが、稀な失敗はまだエッジケースとして残っています

---

## 🗺️ 今後の計画

現在の方向性と相性が良い将来案は次のとおりです。

- 🎛️ **キーボードショートカット** による再生操作
- 🌐 **ブラウザで開く** 動線をプレイヤーや PiP に追加
- 🔊 **グローバル音量コントロール** を埋め込みプレイヤーと独立して提供
- 🏷️ **クリック可能なタグ** から検索条件へ直接反映
- 🔗 **タイトル / 投稿者リンク** を埋め込みプレイヤー上部へ追加
- 🐧 **Linux 対応** を Tauri のクロスプラットフォーム性で進める
- 📦 **オフライン再生** のためのローカルダウンロード
- 🗂️ **Watch Later を超えるカスタムプレイリスト**

---

## 📄 ライセンス

[MIT License](./LICENSE)
