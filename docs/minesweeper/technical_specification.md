# マルチプレイヤーマインスイーパー 技術仕様書 🔧

## アーキテクチャ概要
マルチプレイヤーマインスイーパーは、Rust + WebAssemblyで実装されたクライアント-サーバーアーキテクチャを採用します。サーバーはゲームの状態を管理し、WebSocketを通じてクライアントと通信します。

```
+----------------+       WebSocket       +----------------+
|                |<-------------------->|                |
|    クライアント   |      (port 8101)     |     サーバー     |
|  (WASM + JS)   |                      |     (Rust)     |
|                |                      |                |
+----------------+                      +----------------+
        ^                                      ^
        |                                      |
        | HTTP (port 8001)                     |
        |                                      |
+----------------+                      +----------------+
|   Webブラウザ   |                      |   ゲーム状態DB   |
+----------------+                      +----------------+
```

## 通信プロトコル
### WebSocket (ポート8101)
- **接続確立**: クライアントがサーバーに接続
- **メッセージフォーマット**: JSON形式で以下の構造
  ```rust
  struct GameMessage {
      msg_type: MessageType,
      payload: serde_json::Value,
      timestamp: u64,
  }
  
  enum MessageType {
      Join,              // ゲーム参加
      Leave,             // ゲーム退出
      RevealCell,        // セル公開
      FlagCell,          // 旗設置
      GameState,         // ゲーム状態更新
      Chat,              // チャットメッセージ
      Error,             // エラー通知
  }
  ```

### HTTP サーバー (ポート8001)
- 静的ファイル配信（HTML, CSS, JS, WASM）
- RESTful API エンドポイント:
  - `GET /api/games` - アクティブなゲーム一覧取得
  - `POST /api/games` - 新規ゲーム作成
  - `GET /api/games/{id}` - 特定ゲームの情報取得
  - `GET /api/users/{id}` - ユーザー情報取得

## データモデル
### ゲームボード
```rust
struct GameBoard {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    mine_count: usize,
}

struct Cell {
    x: usize,
    y: usize,
    is_mine: bool,
    is_revealed: bool,
    is_flagged: bool,
    adjacent_mines: u8,
}
```

### ゲームセッション
```rust
struct GameSession {
    id: Uuid,
    board: GameBoard,
    players: Vec<Player>,
    game_mode: GameMode,
    state: GameState,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

enum GameMode {
    Cooperative,    // 協力モード
    Competitive,    // 競争モード
}

enum GameState {
    Waiting,        // プレイヤー待機中
    InProgress,     // ゲーム進行中
    Won,            // ゲーム勝利
    Lost,           // ゲーム敗北
}
```

### プレイヤー
```rust
struct Player {
    id: Uuid,
    name: String,
    score: usize,
    color: String,  // プレイヤー識別色
}
```

## ECSの活用
既存のECSフレームワークを活用して以下のコンポーネントとシステムを実装します：

### コンポーネント
- `Position` - セルの位置情報
- `Mine` - 地雷の有無
- `Revealed` - セルの公開状態
- `Flagged` - 旗の設置状態
- `AdjacentCount` - 隣接する地雷の数
- `PlayerOwned` - どのプレイヤーが操作したか

### システム
- `BoardGenerationSystem` - ゲームボード生成
- `RevealSystem` - セル公開ロジック
- `GameStateSystem` - ゲーム状態管理
- `NetworkSyncSystem` - ネットワーク同期
- `ScoreSystem` - スコア計算

## パフォーマンス要件
- WebSocketメッセージの最大サイズ: 16KB
- 最大同時接続数: 100
- クライアント-サーバー間の最大レイテンシ: 300ms
- 最小フレームレート: 30FPS

## セキュリティ考慮事項
- WebSocketコネクション認証
- 不正なゲーム操作の検出と防止
- レート制限の実装
- サニタイズされた入力処理

## テスト戦略
- 単体テスト: 各コンポーネントとシステムの機能テスト
- 統合テスト: クライアント-サーバー通信テスト
- 負荷テスト: 多数のクライアント接続時のパフォーマンステスト
- ユーザビリティテスト: 実際のプレイヤーによるフィードバック 