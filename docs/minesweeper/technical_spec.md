# マルチプレイヤーマインスイーパー 技術仕様書 🛠️

## システム概要

マルチプレイヤーマインスイーパーは、Rust と WebAssembly (Wasm) を用いたブラウザベースのゲームです。プレイヤーは協力モードまたは競争モードでマインスイーパーをプレイできます。

## 技術スタック

### サーバーサイド
- **言語**: Rust
- **WebSocketサーバー**: tokio + tokio-tungstenite (ポート: 8101)
- **HTTPサーバー**: actix-web (ポート: 8001)
- **データベース**: SQLite（ローカル開発用）/ PostgreSQL（本番環境用）
- **認証**: JWT (JSON Web Tokens)

### クライアントサイド
- **フロントエンド基盤**: HTML5 + CSS3 + JavaScript
- **Rustコード**: WebAssembly (Wasm) にコンパイル
- **WASM処理**: wasm-bindgen + web-sys
- **レンダリング**: Canvas API
- **UI**: カスタムコンポーネント + シンプルな CSS フレームワーク
- **WebSocket通信**: ブラウザの WebSocket API

## アーキテクチャ

### 全体構成

```
+---------------------+         +---------------------+
|                     |         |                     |
|    クライアント     | <-----> |      サーバー       |
|    (ブラウザ)       |         |                     |
|                     |         |                     |
+---------------------+         +---------------------+
       |                                 |
       v                                 v
+---------------------+         +---------------------+
|  Rust → WebAssembly |         |      Rustコード     |
|                     |         |                     |
+---------------------+         +---------------------+
```

### サーバー構成

```
+---------------------------------------------+
|                サーバー                     |
|                                             |
| +-------------------+ +-------------------+ |
| |                   | |                   | |
| |   HTTPサーバー    | |  WebSocketサーバー| |
| |  (静的アセット)   | |    (ゲーム通信)   | |
| |                   | |                   | |
| +-------------------+ +-------------------+ |
|              |                 |            |
| +-------------------+ +-------------------+ |
| |                   | |                   | |
| |    ゲームロジック | |   プレイヤー管理  | |
| |                   | |                   | |
| +-------------------+ +-------------------+ |
|              |                 |            |
| +-------------------+ +-------------------+ |
| |                   | |                   | |
| |    デスリンク     | |    データベース   | |
| |                   | |                   | |
| +-------------------+ +-------------------+ |
+---------------------------------------------+
```

## コンポーネント詳細

### サーバーコンポーネント

#### HTTPサーバー (actix-web)
- 静的アセット配信（HTML, CSS, JavaScript, WASM）
- REST API エンドポイント提供
  - ユーザー認証
  - ゲームルーム管理
  - 統計情報

#### WebSocketサーバー (tokio-tungstenite)
- リアルタイム通信
- ルーム管理
- メッセージブロードキャスト
- 接続状態監視

#### ゲームロジックモジュール
- ゲームボード生成
- セル開封処理
- 勝利/敗北条件判定
- スコア計算

#### プレイヤー管理モジュール
- プレイヤーセッション管理
- 認証・認可
- プレイヤー情報の永続化

#### データストレージ
- プレイヤープロフィール保存
- ゲーム履歴保存
- ハイスコア管理

### クライアントコンポーネント

#### WebAssemblyモジュール
- Rustをコンパイルして生成
- ゲームロジックの一部をブラウザで実行
- 入力処理
- ローカル状態管理

#### レンダリングエンジン
- Canvas APIによるゲームボード描画
- アニメーション処理
- スプライト管理

#### 通信モジュール
- WebSocket接続管理
- メッセージのシリアライズ/デシリアライズ
- 再接続処理
- 通信状態監視

#### UIコンポーネント
- ゲームコントロール（開始、停止、リセット）
- チャットインターフェース
- 設定パネル
- スコアボード

## データモデル

### ゲームボード
```rust
struct GameBoard {
    width: u8,
    height: u8,
    cells: Vec<Cell>,
    mine_count: u32,
}

struct Cell {
    position: (u8, u8),
    is_mine: bool,
    is_revealed: bool,
    is_flagged: bool,
    adjacent_mines: u8,
    revealed_by: Option<PlayerId>,
}
```

### プレイヤー
```rust
struct Player {
    id: PlayerId,
    username: String,
    color: RgbaColor,
    score: u32,
    is_host: bool,
    is_ready: bool,
    connection_state: ConnectionState,
}

enum ConnectionState {
    Connected,
    Disconnected(Instant), // 切断時刻
    Reconnecting,
}
```

### ゲームルーム
```rust
struct GameRoom {
    id: RoomId,
    code: String, // 参加コード（ABCDE形式）
    players: Vec<Player>,
    board: GameBoard,
    game_mode: GameMode,
    difficulty: Difficulty,
    state: GameState,
    created_at: Instant,
    last_activity: Instant,
}

enum GameMode {
    Cooperative,
    Competitive,
}

enum Difficulty {
    Beginner,    // 9x9, 10 mines
    Intermediate, // 16x16, 40 mines
    Advanced,    // 30x16, 99 mines
    Custom(u8, u8, u32), // width, height, mines
}

enum GameState {
    Lobby,
    InProgress(Instant), // 開始時刻
    Completed(GameResult),
    Abandoned,
}

struct GameResult {
    winner: Option<PlayerId>, // 競争モードの場合のみ
    scores: HashMap<PlayerId, u32>,
    duration: Duration,
    is_victory: bool,
}
```

## 通信プロトコル

WebSocketを使用して、クライアントとサーバー間でJSONメッセージを交換します。

### メッセージ形式
```json
{
  "type": "message_type",
  "data": { ... },
  "timestamp": 1234567890
}
```

### クライアント → サーバー メッセージ
- `join_room`: ルームに参加
- `create_room`: 新しいルームを作成
- `ready`: 準備完了状態をトグル
- `start_game`: ゲーム開始（ホストのみ）
- `reveal_cell`: セルを開く
- `flag_cell`: セルに旗を立てる
- `chat_message`: チャットメッセージを送信
- `leave_room`: ルームを退出

### サーバー → クライアント メッセージ
- `room_update`: ルーム情報の更新
- `game_state`: ゲーム状態の更新
- `cell_revealed`: セルが開かれた
- `cell_flagged`: セルに旗が立てられた
- `game_over`: ゲーム終了（勝利または敗北）
- `player_joined`: 新しいプレイヤーが参加
- `player_left`: プレイヤーが退出
- `chat_broadcast`: チャットメッセージをブロードキャスト
- `error`: エラーメッセージ

## 状態管理と同期

### サーバー権威モデル
- サーバーがゲーム状態の信頼できる唯一の情報源
- クライアントは予測的に動作可能だが、サーバーからの確認が必要
- 不正行為防止のための検証をサーバーで実施

### 状態同期
- イベントベースの同期
- デルタ更新（変更された部分のみ送信）
- クライアント側での補間/予測

### 遅延対策
- クライアント側予測
- サーバー更新時の滑らかな補正
- 接続品質モニタリング

## セキュリティ考慮事項

### 入力検証
- すべてのクライアント入力はサーバーで検証
- 不正な操作の拒否と記録

### 認証と認可
- JWTによるセッション認証
- 適切な権限チェック（ホスト専用操作など）

### レート制限
- 短時間での過剰なリクエスト防止
- DoS攻撃対策

## パフォーマンス最適化

### ネットワーク最適化
- メッセージの圧縮
- バッチ処理（複数の小さな更新を1つのメッセージにまとめる）
- 優先度に基づくメッセージ送信

### WebAssembly最適化
- バイナリサイズの最小化
- メモリ使用量の最適化
- ホットパスの最適化

### レンダリング最適化
- ダーティ領域のみの再描画
- オフスクリーンレンダリング
- キャンバスレイヤーの適切な使用

## エラー処理と回復

### 接続エラー
- 自動再接続メカニズム
- 接続状態の通知
- オフライン操作のキューイング

### ゲームエラー
- 無効な操作の優雅な処理
- エラーログ記録
- ユーザーへの明確なフィードバック

### クラッシュ回復
- ゲーム状態の定期的な保存
- 再接続時の状態回復
- クラッシュレポート機能

## 開発・デプロイフロー

### 開発環境
- Rustツールチェーン (stable)
- wasm-pack ビルドツール
- ローカルデバッグサーバー

### ビルドプロセス
```bash
# Wasmビルド
wasm-pack build --target web --out-dir ./static/wasm

# サーバービルド
cargo build --release
```

### デプロイメント
- サーバー: Docker コンテナ
- クライアント: 静的アセットとしてCDN配信
- データベース: マネージドサービスまたはDockerコンテナ

## 将来の拡張性

### 追加ゲームモード
- タイムアタックモード
- カスタムルール対応

### ソーシャル機能
- フレンドリスト
- リーダーボード

### カスタマイゼーション
- テーマ変更
- カスタムアバター

## テスト戦略

### ユニットテスト
- ゲームロジックの検証
- エッジケースの処理確認

### 統合テスト
- クライアント-サーバー通信テスト
- 複数プレイヤーシナリオテスト

### 負荷テスト
- 同時接続テスト
- メッセージ処理スループットテスト

### ユーザーテスト
- UIユーザビリティテスト
- ゲームプレイフィードバック収集 