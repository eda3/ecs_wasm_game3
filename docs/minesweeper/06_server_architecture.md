# マルチプレイヤーゲームフレームワーク サーバーアーキテクチャ設計 🖥️

## 概要
本ドキュメントでは、汎用マルチプレイヤーゲームフレームワークのサーバーサイド実装に関する詳細な設計を記述します。サーバーはRustで実装され、WebSocketとHTTPの両方のサービスを提供します。

## アーキテクチャ

### 全体構成

```
                               +------------------------+
                               |                        |
                               |    メインサーバー      |
                               |                        |
                               +-----------+------------+
                                           |
                      +-------------------+-------------------+
                      |                   |                   |
          +-----------v---------+ +-------v---------+ +------v----------+
          |                     | |                 | |                 |
          |   HTTPサーバー      | | WebSocketサーバー| |  ゲームマネージャー |
          |   (actix-web)      | | (tokio-tungstenite)| |                 |
          |                     | |                 | |                 |
          +---------------------+ +-----------------+ +-----------------+
                                                           |
                                             +-------------+-------------+
                                             |                           |
                                    +--------v--------+         +--------v--------+
                                    |                 |         |                 |
                                    |   ルームマネージャー |         |   ECSワールド    |
                                    |                 |         |                 |
                                    +-----------------+         +-----------------+
```

## コンポーネント詳細

### メインサーバー
- サーバーの起動とシャットダウンを管理
- 設定の読み込みと適用
- 各サブシステムの初期化と連携

```rust
pub struct GameServer {
    config: ServerConfig,
    http_server: HttpServer,
    websocket_server: WebSocketServer,
    game_manager: GameManager,
}

impl GameServer {
    pub fn new(config: ServerConfig) -> Self { ... }
    pub async fn start(&mut self) -> Result<(), ServerError> { ... }
    pub async fn shutdown(&mut self) -> Result<(), ServerError> { ... }
}
```

### HTTPサーバー (actix-web)
- 静的ファイル配信
- APIエンドポイント提供
- ヘルスチェック

```rust
#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub host_name: String,
    pub game_type: GameType,
    pub settings: GameSettings,
}

#[derive(Serialize)]
pub struct RoomResponse {
    pub room_id: String,
    pub room_code: String,
    pub player_count: usize,
    pub max_players: usize,
    pub game_type: GameType,
    pub settings: GameSettings,
    pub state: GameState,
}

// エンドポイント例
#[post("/api/rooms")]
async fn create_room(
    req: web::Json<CreateRoomRequest>,
    game_manager: web::Data<GameManager>,
) -> impl Responder { ... }

#[get("/api/rooms/{room_code}")]
async fn get_room(
    path: web::Path<String>,
    game_manager: web::Data<GameManager>,
) -> impl Responder { ... }
```

### WebSocketサーバー (tokio-tungstenite)
- WebSocket接続の確立と管理
- メッセージ処理と転送
- クライアント状態の監視

```rust
pub struct WebSocketServer {
    addr: SocketAddr,
    game_manager: Arc<Mutex<GameManager>>,
    connections: HashMap<Uuid, WebSocketConnection>,
}

impl WebSocketServer {
    pub fn new(addr: SocketAddr, game_manager: Arc<Mutex<GameManager>>) -> Self { ... }
    pub async fn start(&mut self) -> Result<(), WebSocketError> { ... }
    async fn handle_connection(&mut self, socket: WebSocket) { ... }
    async fn process_message(&mut self, client_id: Uuid, msg: Message) { ... }
    async fn send_to_client(&mut self, client_id: Uuid, msg: ServerMessage) { ... }
    async fn broadcast_to_room(&mut self, room_id: RoomId, msg: ServerMessage) { ... }
}
```

### ゲームマネージャー
- ゲームルームの作成と管理
- プレイヤー管理
- ゲームロジックの実行

```rust
pub struct GameManager {
    rooms: HashMap<RoomId, GameRoom>,
    room_codes: HashMap<String, RoomId>,
    players: HashMap<PlayerId, PlayerSession>,
}

impl GameManager {
    pub fn new() -> Self { ... }
    pub fn create_room(&mut self, host_name: String, game_type: GameType, settings: GameSettings) -> Result<RoomId, GameError> { ... }
    pub fn join_room(&mut self, room_code: &str, player_name: String) -> Result<(RoomId, PlayerId), GameError> { ... }
    pub fn leave_room(&mut self, room_id: RoomId, player_id: PlayerId) -> Result<(), GameError> { ... }
    pub fn start_game(&mut self, room_id: RoomId, player_id: PlayerId) -> Result<(), GameError> { ... }
    pub fn perform_action(&mut self, room_id: RoomId, player_id: PlayerId, action: GameAction) -> Result<ActionResult, GameError> { ... }
    // その他のゲーム操作メソッド...
}
```

### ルームマネージャー
- 個別のゲームルームの状態管理
- プレイヤーの入退室処理
- ゲーム進行の管理

```rust
pub struct RoomManager {
    room: GameRoom,
    event_queue: VecDeque<RoomEvent>,
}

impl RoomManager {
    pub fn new(room: GameRoom) -> Self { ... }
    pub fn process_events(&mut self) -> Vec<RoomEvent> { ... }
    pub fn add_player(&mut self, player: Player) -> Result<(), GameError> { ... }
    pub fn remove_player(&mut self, player_id: PlayerId) -> Result<(), GameError> { ... }
    pub fn toggle_ready(&mut self, player_id: PlayerId) -> Result<bool, GameError> { ... }
    pub fn start_game(&mut self, player_id: PlayerId) -> Result<(), GameError> { ... }
    // その他のルーム管理メソッド...
}
```

### ECSワールド
既存のECSフレームワークを利用する場合の統合ポイント。ゲームロジックをコンポーネントとシステムで実装。

```rust
pub struct GameWorld {
    world: World,
    schedule: Schedule,
}

impl GameWorld {
    pub fn new() -> Self { ... }
    pub fn register_components(&mut self) { ... }
    pub fn register_systems(&mut self) { ... }
    pub fn create_game_state(&mut self, settings: &GameSettings) -> EntityId { ... }
    pub fn perform_action(&mut self, game_id: EntityId, action: GameAction) -> Result<ActionResult, GameError> { ... }
    pub fn update(&mut self) { ... }
}
```

## メッセージ処理フロー

### クライアントからのメッセージ処理
```
Client -> WebSocketサーバー -> メッセージ解析 -> ゲームマネージャー -> ルームマネージャー -> ゲームロジック実行 -> 結果生成 -> WebSocketサーバー -> Broadcast -> Clients
```

例として、ゲーム内アクションの処理フロー：
1. クライアントがActionメッセージを送信
2. WebSocketサーバーがメッセージを受信して解析
3. ゲームマネージャーのperform_actionメソッドを呼び出し
4. 対応するルームマネージャーが処理を実行
5. ゲームロジック（ECSまたは直接実装）が実行され結果を返す
6. 結果がRoomEventとして生成される
7. WebSocketサーバーがルーム内の全クライアントに結果をブロードキャスト

## エラーハンドリング

```rust
#[derive(Debug, Error)]
pub enum GameError {
    #[error("Room not found: {0}")]
    RoomNotFound(RoomId),
    
    #[error("Player not found: {0}")]
    PlayerNotFound(PlayerId),
    
    #[error("Room is full")]
    RoomFull,
    
    #[error("Invalid action: {0}")]
    InvalidAction(String),
    
    #[error("Not authorized: {0}")]
    NotAuthorized(String),
    
    #[error("Game not in progress")]
    GameNotInProgress,
    
    #[error("Internal error: {0}")]
    Internal(String),
}
```

エラーハンドリングのアプローチ：
1. 明確なエラー型の定義
2. Result型を用いた一貫したエラーハンドリング
3. クライアントへの適切なエラーメッセージの送信
4. 重大なエラーのログ記録
5. 自動リカバリーメカニズムの実装（可能な場合）

## 接続管理

### 新規接続の確立
```rust
async fn accept_connection(socket: WebSocket, addr: SocketAddr, game_manager: Arc<Mutex<GameManager>>) {
    // クライアントIDの生成
    let client_id = Uuid::new_v4();
    
    // 接続の初期化
    let (tx, rx) = socket.split();
    
    // クライアントセッションの作成
    let client_session = ClientSession::new(client_id, tx);
    
    // 接続リストへの追加
    connections.insert(client_id, client_session);
    
    // Welcomeメッセージの送信
    let welcome_msg = ServerMessage::Welcome { client_id };
    send_to_client(client_id, welcome_msg).await;
    
    // メッセージ受信ループの開始
    while let Some(msg) = rx.next().await {
        // メッセージ処理
        if let Err(e) = process_message(client_id, msg, game_manager.clone()).await {
            log::error!("Error processing message: {}", e);
            break;
        }
    }
    
    // 切断処理
    handle_disconnect(client_id, game_manager).await;
}
```

### 切断処理
```rust
async fn handle_disconnect(client_id: Uuid, game_manager: Arc<Mutex<GameManager>>) {
    // 接続の削除
    let client_session = connections.remove(&client_id);
    
    // 関連するプレイヤーを取得
    if let Some(player_id) = client_session.player_id {
        // プレイヤーが参加中のルームからプレイヤーを削除
        let mut manager = game_manager.lock().await;
        if let Some(room_id) = manager.get_player_room(player_id) {
            // ルームからプレイヤーを削除（切断状態に更新）
            manager.player_disconnected(room_id, player_id);
            
            // ルーム内の他のプレイヤーに通知
            let disconnect_msg = ServerMessage::PlayerDisconnected { player_id };
            broadcast_to_room(room_id, disconnect_msg).await;
        }
    }
}
```

## パフォーマンス最適化

### メッセージ処理の最適化
- バッチ処理: 短時間に多数の更新がある場合、1つのメッセージにまとめる
- メッセージの圧縮: 大きなメッセージはバイナリ形式で送信
- WebSocketフレーム最適化: 最適なフレームサイズの使用

### 同時接続の処理
- 非同期I/O: tokioを使用した効率的な非同期処理
- コネクションプール: アクティブなコネクションの効率的な管理
- レート制限: クライアントごとの最大メッセージレートの設定

## メンテナンスと監視
- 定期的なルーム掃除: 非アクティブなルームの定期的な削除
- タイムアウト処理: 長時間アイドル状態のクライアントの切断
- ヘルスチェック: サーバー状態の定期的なチェック
- メトリクス収集: 接続数、メッセージレート、エラー率などの監視

## 設定例

```rust
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub http_bind_addr: String,
    pub ws_bind_addr: String,
    pub static_dir: String,
    pub max_rooms: usize,
    pub room_timeout_secs: u64,
    pub player_timeout_secs: u64,
    pub max_players_per_room: u8,
    pub ping_interval_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_bind_addr: "127.0.0.1:8001".to_string(),
            ws_bind_addr: "127.0.0.1:8101".to_string(),
            static_dir: "./static".to_string(),
            max_rooms: 1000,
            room_timeout_secs: 3600, // 1時間
            player_timeout_secs: 300, // 5分
            max_players_per_room: 8,
            ping_interval_secs: 30,
        }
    }
}
```

## 起動スクリプト例

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ロギングの設定
    env_logger::init();
    
    // 設定の読み込み
    let config = load_config()?;
    
    // サーバーの作成
    let mut server = GameServer::new(config);
    
    // シグナルハンドラの設定
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    
    // Ctrl+Cハンドラ
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        shutdown_tx.send(()).expect("Failed to send shutdown signal");
    });
    
    // サーバー起動
    log::info!("Starting Game Server...");
    let server_task = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            log::error!("Server error: {}", e);
        }
    });
    
    // シャットダウンシグナルの待機
    shutdown_rx.await?;
    
    // サーバーのシャットダウン
    log::info!("Shutting down server...");
    
    Ok(())
}
``` 