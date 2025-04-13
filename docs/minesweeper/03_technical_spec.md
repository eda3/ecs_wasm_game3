# ゲームフレームワーク 技術仕様書

## システム概要

このシステムはRustとWebAssemblyを使用した汎用的なマルチプレイヤーゲームフレームワークです。様々なタイプのゲームを容易に開発できるよう設計されており、協力モードや対戦モードなど複数のプレイスタイルをサポートしています。

## 技術スタック

### サーバーサイド
- **言語**: Rust
- **WebSocketサーバー**: tokioベースのWebSocketライブラリ
- **HTTPサーバー**: actix-web
- **データベース**: SQLite（開発）、PostgreSQL（本番）
- **認証**: JWT（JSON Web Tokens）

### クライアントサイド
- **HTML5/CSS3**
- **JavaScript**
- **WebAssembly**（Rustからコンパイル）

## アーキテクチャ

システム全体は以下のコンポーネントで構成されています：

```
                 ┌───────────────────────────────────────┐
                 │              クライアント              │
                 │                                       │
                 │   ┌─────────────┐    ┌─────────────┐  │
                 │   │  WASM Game  │    │    UI       │  │
                 │   │   Engine    │◄───►  Components │  │
                 │   └─────┬───────┘    └─────────────┘  │
                 │         │                             │
                 └─────────┼─────────────────────────────┘
                           │
                           ▼
     ┌───────────────────────────────────────────────────────┐
     │                       WebSocket                       │
     └───────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                           サーバー                              │
│                                                                 │
│  ┌─────────────┐   ┌────────────┐   ┌────────────────────────┐  │
│  │  HTTPサーバー │   │ WebSocket │   │                        │  │
│  │ (actix-web) │   │  サーバー   │   │                        │  │
│  └─────────────┘   └─────┬──────┘   │                        │  │
│                          │          │                        │  │
│                    ┌─────▼──────┐   │     ゲームロジック      │  │
│                    │ セッション  │   │                        │  │
│                    │ マネージャー │   │                        │  │
│                    └─────┬──────┘   │                        │  │
│                          │          │                        │  │
│                    ┌─────▼──────┐   │                        │  │
│                    │   ルーム    │   │                        │  │
│                    │ マネージャー │   │                        │  │
│                    └─────┬──────┘   └────────────────────────┘  │
│                          │                                       │
│                          ▼                                       │
│                    ┌────────────┐                                │
│                    │ データベース │                                │
│                    │（オプション） │                                │
│                    └────────────┘                                │
└─────────────────────────────────────────────────────────────────┘
```

## コンポーネント詳細

### サーバーコンポーネント

1. **HTTPサーバー**
   - 静的ファイルの提供
   - RESTエンドポイント（ユーザー管理、統計など）
   - ヘルスチェックエンドポイント

2. **WebSocketサーバー**
   - クライアント接続管理
   - メッセージのシリアライズ/デシリアライズ
   - メッセージルーティング

3. **セッションマネージャー**
   - プレイヤーセッション追跡
   - 一時的なユーザーID割り当て
   - タイムアウト処理

4. **ルームマネージャー**
   - ゲームルーム作成/削除
   - プレイヤー参加/退出管理
   - ルームメタデータ管理

5. **ゲームロジックエンジン**
   - 汎用ゲームステート管理
   - ゲームルール検証
   - プラグイン式ゲーム実装
   - イベント発行

### クライアントコンポーネント

1. **WASMゲームエンジン**
   - サーバーとの通信
   - ローカルゲーム状態管理
   - 入力処理
   - 予測/補正ロジック

2. **UIコンポーネント**
   - レンダリングエンジン
   - ユーザー入力ハンドリング
   - アニメーション
   - サウンド管理

## データモデル

### 基本構造体

```rust
// ゲームインターフェイス - すべてのゲーム実装はこのトレイトを実装する必要がある
pub trait Game {
    type State: GameState;
    type Action: GameAction;
    
    fn new(config: GameConfig) -> Self;
    fn apply_action(&mut self, action: Self::Action) -> Result<(), GameError>;
    fn get_state(&self) -> &Self::State;
    fn is_game_over(&self) -> bool;
    fn get_winners(&self) -> Vec<PlayerId>;
}

// 汎用ゲーム状態トレイト
pub trait GameState: Clone + Serialize + Deserialize {
    fn get_player_view(&self, player_id: PlayerId) -> Self;
}

// 汎用ゲームアクショントレイト
pub trait GameAction: Clone + Serialize + Deserialize {
    fn get_player_id(&self) -> PlayerId;
    fn is_valid(&self, state: &dyn GameState) -> bool;
}

// プレイヤー
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub connected: bool,
    pub last_activity: SystemTime,
}

// ゲームルーム
pub struct GameRoom {
    pub id: RoomId,
    pub name: String,
    pub game_type: GameType,
    pub players: Vec<Player>,
    pub state: GameState,
    pub created_at: SystemTime,
    pub settings: GameSettings,
    pub status: RoomStatus,
}

// ルームステータス
pub enum RoomStatus {
    Lobby,
    Playing,
    GameOver,
}

// ゲーム設定
pub struct GameSettings {
    pub max_players: u8,
    pub game_mode: GameMode,
    pub custom_options: HashMap<String, Value>,
}

// ゲームモード
pub enum GameMode {
    Cooperative,
    Competitive,
    Team,
}
```

## 通信プロトコル

WebSocketを介したJSON形式のメッセージングシステムを使用します。

### メッセージ構造

```rust
pub struct Message {
    pub msg_type: MessageType,
    pub payload: Value,
    pub client_id: Option<ClientId>,
    pub timestamp: u64,
}

pub enum MessageType {
    // システムメッセージ
    Connect,
    Disconnect,
    Ping,
    Pong,
    Error,
    
    // ルーム関連
    CreateRoom,
    JoinRoom,
    LeaveRoom,
    RoomInfo,
    RoomList,
    
    // ゲーム関連
    GameAction,
    GameState,
    GameStart,
    GameEnd,
    
    // チャット
    ChatMessage,
    
    // カスタムメッセージ
    Custom(String),
}
```

### メッセージ例

```json
{
  "msg_type": "GameAction",
  "payload": {
    "action_type": "PlaceToken",
    "position": {"x": 3, "y": 2},
    "player_id": "player123"
  },
  "client_id": "client456",
  "timestamp": 1623456789
}
```

## ステート管理と同期

### サーバー権威モデル

サーバーが唯一の信頼できるゲーム状態を保持します。クライアントからのアクションはすべてサーバーで検証され、承認されたアクションのみがゲーム状態に適用されます。

### クライアント予測

レイテンシーを最小化するために、クライアントはローカルで予測を実行します：

1. クライアントはアクションをローカルに適用
2. 同時にアクションをサーバーに送信
3. サーバーがアクションを検証し、処理
4. サーバーが更新された状態をブロードキャスト
5. クライアントが必要に応じて自身の状態を修正

### 部分状態更新

帯域幅を節約するため、完全な状態ではなく差分更新を送信します：

```rust
pub struct StateUpdate {
    pub full_state: bool,
    pub delta: Option<StateDelta>,
    pub state: Option<GameState>,
    pub sequence_number: u64,
}

pub struct StateDelta {
    pub changed_entities: Vec<Entity>,
    pub removed_entity_ids: Vec<EntityId>,
    pub added_entities: Vec<Entity>,
}
```

## セキュリティ考慮事項

1. **入力検証**
   - すべてのクライアント入力をサーバーで検証
   - 不正なアクションの拒否

2. **レート制限**
   - クライアントごとのメッセージ送信レート制限
   - 過剰なリクエストの防止

3. **認証**（拡張機能）
   - JWT（JSON Web Tokens）による認証
   - セッションタイムアウト

4. **WebSocketセキュリティ**
   - WSS（WebSocket Secure）接続の使用
   - 接続の適切な閉鎖処理

## パフォーマンス最適化

### ネットワーク最適化
- バイナリプロトコル（オプション）
- メッセージバッチ処理
- 差分更新

### WebAssembly最適化
- メモリ使用量の最小化
- 計算集約型タスクの最適化
- バイナリサイズの最適化

### レンダリング最適化
- キャンバスレンダリング効率化
- アセット事前読み込み
- アニメーションフレーム管理

## エラー処理と復旧

### 接続エラー
- 自動再接続メカニズム
- 指数バックオフリトライ
- 接続状態表示

### ゲームエラー
- アクションロールバック
- 状態修正機能
- エラーログ記録

### クラッシュリカバリー
- 定期的な状態スナップショット
- セッション復元メカニズム

## 開発・デプロイメントフロー

### 開発環境
- Rustツールチェーン
- wasm-packビルドツール
- npm開発サーバー

### ビルドプロセス
```
1. Rustバックエンドのビルド: cargo build --release
2. WASMコンポーネントのコンパイル: wasm-pack build --target web
3. フロントエンドアセットの構築: npm run build
```

### デプロイメント
- Dockerイメージ作成
- 環境変数による設定
- ヘルスチェックとモニタリング

## スケーラビリティ

### 将来の拡張性
- 新しいゲームタイプの追加
- カスタムゲームロジックプラグイン
- 拡張UIコンポーネント

### 水平スケーリング
- WebSocketサーバーの複数インスタンス
- ルーム分散システム
- ロードバランシング

## テスト戦略

### 単体テスト
- ゲームロジックのユニットテスト
- メッセージハンドラーのテスト

### 統合テスト
- クライアント-サーバー通信テスト
- 完全なゲームフローテスト

### 負荷テスト
- 同時接続テスト
- メッセージスループットテスト
- レイテンシーテスト

### ユーザーテスト
- プレイテスト
- UIユーザビリティテスト 