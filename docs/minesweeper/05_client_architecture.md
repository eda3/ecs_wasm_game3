# マルチプレイヤーゲームフレームワーク クライアントアーキテクチャ設計 🎮

## 概要
本ドキュメントでは、マルチプレイヤーゲームフレームワークのクライアントサイド実装に関する詳細な設計を記述します。クライアントはRustで実装され、WebAssembly (Wasm)にコンパイルされてブラウザで実行されます。

## アーキテクチャ

### 全体構成

```
+-------------------------------------------------------------+
|                     ブラウザ環境                            |
+-------------------------------------------------------------+
|                                                             |
|  +------------------------+       +---------------------+   |
|  |                        |       |                     |   |
|  |     HTMLページ         |       |    JavaScript       |   |
|  |                        |       |                     |   |
|  +------------------------+       +---------------------+   |
|                |                            |               |
|                v                            v               |
|  +------------------------+       +---------------------+   |
|  |                        |       |                     |   |
|  |     WebAssembly        | <---> |    WebSocket        |   |
|  |     (Rust)             |       |    クライアント     |   |
|  |                        |       |                     |   |
|  +------------------------+       +---------------------+   |
|                |                                            |
|                v                                            |
|  +------------------------+                                 |
|  |                        |                                 |
|  |     Canvas             |                                 |
|  |     レンダリング       |                                 |
|  |                        |                                 |
|  +------------------------+                                 |
|                                                             |
+-------------------------------------------------------------+
```

## コンポーネント詳細

### WebAssemblyモジュール（Rust）
ゲームロジックの主要部分をRustで実装し、WebAssemblyにコンパイルします。

```rust
// Wasmモジュールのエントリーポイント
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // パニック時のフック設定
    console_error_panic_hook::set_once();
    
    // ロギング初期化
    console_log::init_with_level(log::Level::Debug).unwrap();
    
    // ゲームアプリケーションの初期化
    let app = GameApp::new()?;
    
    // グローバルスコープにアプリインスタンスを保存
    set_app_instance(app);
    
    Ok(())
}
```

### GameApp
アプリケーションのメインコンポーネントで、全体の状態とロジックを管理します。

```rust
pub struct GameApp {
    // アプリの状態
    state: AppState,
    // ゲームシステム
    game: GameSystem,
    // レンダリングシステム
    renderer: RenderSystem,
    // 入力システム
    input: InputSystem,
    // ネットワークシステム
    network: NetworkSystem,
    // UI管理
    ui: UiSystem,
    // エンティティ管理
    world: World,
}

impl GameApp {
    pub fn new() -> Result<Self, JsValue> { ... }
    
    pub fn update(&mut self, delta_time: f32) -> Result<(), JsValue> { ... }
    
    pub fn render(&mut self) -> Result<(), JsValue> { ... }
    
    pub fn handle_input(&mut self, event: InputEvent) -> Result<(), JsValue> { ... }
    
    pub fn connect_to_server(&mut self, url: &str) -> Result<(), JsValue> { ... }
    
    pub fn create_game_room(&mut self, player_name: &str, game_type: GameType, options: GameOptions) -> Result<(), JsValue> { ... }
    
    pub fn join_game_room(&mut self, player_name: &str, room_code: &str) -> Result<(), JsValue> { ... }
}
```

### AppState
アプリケーションの状態を管理するステートマシン。

```rust
#[derive(Debug)]
pub enum AppState {
    Loading,
    MainMenu,
    GameTypeSelect,
    OptionsSelect,
    RoomCreation,
    RoomJoin,
    Lobby(RoomInfo),
    Game(GameInfo),
    GameOver(GameResult),
    Error(String),
}

impl AppState {
    pub fn transition_to(&mut self, new_state: AppState) { ... }
}
```

### ECSベースのゲームシステム

Rustの既存のECSフレームワークを活用して、ゲームロジックを実装します。

```rust
// コンポーネント定義
#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct GameObject {
    pub object_type: ObjectType,
    pub state: GameObjectState,
    pub properties: HashMap<String, Value>,
}

#[derive(Component)]
pub struct GameBoard {
    pub width: u32,
    pub height: u32,
    pub config: GameBoardConfig,
}

#[derive(Component)]
pub struct PlayerOwned {
    pub player_id: PlayerId,
}

// システム定義
pub fn interact_with_object_system(
    world: &mut World,
    board_query: Query<&GameBoard>,
    mut object_query: Query<(&Position, &mut GameObject)>,
    pos: Position,
    player_id: PlayerId,
    action_type: ActionType,
) -> Result<Vec<GameEvent>, GameError> { ... }

pub fn toggle_object_state_system(
    world: &mut World,
    board_query: Query<&GameBoard>,
    mut object_query: Query<(&Position, &mut GameObject)>,
    pos: Position,
    player_id: PlayerId,
) -> Result<bool, GameError> { ... }
```

### レンダリングシステム

ゲームボードとUIをCanvasにレンダリングします。

```rust
pub struct RenderSystem {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    sprites: HashMap<SpriteType, HtmlImageElement>,
    object_size: f32,
    board_offset_x: f32,
    board_offset_y: f32,
}

impl RenderSystem {
    pub fn new() -> Result<Self, JsValue> { ... }
    
    pub fn render(&self, world: &World, app_state: &AppState) -> Result<(), JsValue> { ... }
    
    pub fn render_board(&self, world: &World) -> Result<(), JsValue> { ... }
    
    pub fn render_game_object(&self, object: &GameObject, position: &Position, player_color: Option<[u8; 4]>) -> Result<(), JsValue> { ... }
    
    pub fn render_ui(&self, app_state: &AppState) -> Result<(), JsValue> { ... }
    
    pub fn screen_to_board_position(&self, screen_x: f32, screen_y: f32) -> Option<Position> { ... }
}
```

### 入力システム

ユーザー入力を処理します。

```rust
pub struct InputSystem {
    mouse_position: (f32, f32),
    mouse_buttons: [bool; 3],
    keyboard_state: HashMap<String, bool>,
    touch_state: Vec<TouchPoint>,
    event_listeners: Vec<EventListener>,
}

impl InputSystem {
    pub fn new() -> Result<Self, JsValue> { ... }
    
    pub fn setup_listeners(&mut self, app: Rc<RefCell<GameApp>>) -> Result<(), JsValue> { ... }
    
    pub fn handle_mouse_down(&mut self, event: &MouseEvent) -> Result<(), JsValue> { ... }
    
    pub fn handle_mouse_up(&mut self, event: &MouseEvent) -> Result<(), JsValue> { ... }
    
    pub fn handle_mouse_move(&mut self, event: &MouseEvent) -> Result<(), JsValue> { ... }
    
    pub fn handle_touch_start(&mut self, event: &TouchEvent) -> Result<(), JsValue> { ... }
    
    pub fn handle_touch_end(&mut self, event: &TouchEvent) -> Result<(), JsValue> { ... }
    
    pub fn handle_touch_move(&mut self, event: &TouchEvent) -> Result<(), JsValue> { ... }
    
    pub fn handle_key_down(&mut self, event: &KeyboardEvent) -> Result<(), JsValue> { ... }
    
    pub fn handle_key_up(&mut self, event: &KeyboardEvent) -> Result<(), JsValue> { ... }
}
```

### ネットワークシステム

WebSocketを使用してサーバーと通信します。

```rust
pub struct NetworkSystem {
    socket: Option<WebSocket>,
    client_id: Option<Uuid>,
    connection_status: ConnectionStatus,
    message_queue: VecDeque<ServerMessage>,
    callback: Option<Closure<dyn FnMut(MessageEvent)>>,
}

impl NetworkSystem {
    pub fn new() -> Self { ... }
    
    pub fn connect(&mut self, url: &str) -> Result<(), JsValue> { ... }
    
    pub fn disconnect(&mut self) -> Result<(), JsValue> { ... }
    
    pub fn send_message(&self, message: ClientMessage) -> Result<(), JsValue> { ... }
    
    pub fn process_messages(&mut self, app: &mut GameApp) -> Result<(), JsValue> { ... }
    
    pub fn create_room(&self, player_name: &str, game_type: GameType, options: GameOptions) -> Result<(), JsValue> { ... }
    
    pub fn join_room(&self, player_name: &str, room_code: &str) -> Result<(), JsValue> { ... }
    
    pub fn leave_room(&self) -> Result<(), JsValue> { ... }
    
    pub fn toggle_ready(&self) -> Result<(), JsValue> { ... }
    
    pub fn start_game(&self) -> Result<(), JsValue> { ... }
    
    pub fn perform_action(&self, action_type: ActionType, x: f32, y: f32, params: Option<ActionParams>) -> Result<(), JsValue> { ... }
    
    pub fn send_chat_message(&self, content: &str) -> Result<(), JsValue> { ... }
}
```

### UIシステム

ユーザーインターフェースを管理します。

```rust
pub struct UiSystem {
    elements: HashMap<String, HtmlElement>,
    event_handlers: Vec<Closure<dyn FnMut(Event)>>,
}

impl UiSystem {
    pub fn new() -> Result<Self, JsValue> { ... }
    
    pub fn update_for_state(&mut self, app_state: &AppState) -> Result<(), JsValue> { ... }
    
    pub fn show_main_menu(&mut self) -> Result<(), JsValue> { ... }
    
    pub fn show_game_type_select(&mut self) -> Result<(), JsValue> { ... }
    
    pub fn show_options_select(&mut self) -> Result<(), JsValue> { ... }
    
    pub fn show_room_creation(&mut self) -> Result<(), JsValue> { ... }
    
    pub fn show_room_join(&mut self) -> Result<(), JsValue> { ... }
    
    pub fn show_lobby(&mut self, room_info: &RoomInfo) -> Result<(), JsValue> { ... }
    
    pub fn show_game(&mut self, game_info: &GameInfo) -> Result<(), JsValue> { ... }
    
    pub fn show_game_over(&mut self, result: &GameResult) -> Result<(), JsValue> { ... }
    
    pub fn update_player_list(&mut self, players: &[Player]) -> Result<(), JsValue> { ... }
    
    pub fn update_chat(&mut self, messages: &[ChatMessage]) -> Result<(), JsValue> { ... }
    
    pub fn update_game_info(&mut self, game_info: &GameInfo) -> Result<(), JsValue> { ... }
}
```

## メッセージングとプロトコル

クライアントとサーバー間の通信に使用するメッセージ形式。

```rust
// クライアントからサーバーへのメッセージ
#[derive(Serialize)]
pub enum ClientMessage {
    JoinRoom {
        room_id: String,
        player_name: String,
    },
    CreateRoom {
        player_name: String,
        game_type: GameType,
        options: GameOptions,
    },
    LeaveRoom,
    StartGame,
    Ping {
        timestamp: u64,
    },
    PerformAction {
        action_type: ActionType,
        x: f32,
        y: f32,
        params: Option<ActionParams>,
    },
    ChatMessage {
        message: String,
    },
    PlayerReady {
        ready: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GameOptions {
    pub parameters: HashMap<String, Value>,
}

// サーバーからクライアントへのメッセージ
#[derive(Deserialize)]
pub enum ServerMessage {
    Welcome {
        player_id: String,
        server_time: u64,
    },
    RoomJoined {
        room_id: String,
        game_type: GameType,
        options: GameOptions,
        players: Vec<PlayerInfo>,
    },
    RoomCreated {
        room_id: String,
        game_type: GameType,
        options: GameOptions,
    },
    PlayerJoined {
        player: PlayerInfo,
    },
    PlayerLeft {
        player_id: String,
    },
    GameStarted {
        game_id: String,
        start_time: u64,
        board: GameBoardInfo,
    },
    Pong {
        timestamp: u64,
        server_time: u64,
    },
    ActionPerformed {
        player_id: String,
        action_type: ActionType,
        x: f32,
        y: f32,
        result: ActionResult,
        affected_objects: Vec<GameObjectUpdate>,
    },
    GameOver {
        result: String,
        cause_player_id: Option<String>,
        scores: Vec<PlayerScore>,
        game_time: u32,
    },
    GameWon {
        scores: Vec<PlayerScore>,
        game_time: u32,
        winner: Option<String>,
    },
    ChatReceived {
        player_id: String,
        player_name: String,
        message: String,
        timestamp: u64,
    },
    PlayerReadyChanged {
        player_id: String,
        ready: bool,
    },
    Error {
        code: String,
        message: String,
    },
    SystemMessage {
        message_type: String, // "INFO" または "WARNING"
        message: String,
        timestamp: u64,
    },
}

#[derive(Deserialize)]
pub struct PlayerInfo {
    pub id: String,
    pub name: String,
    pub is_host: bool,
    pub ready: bool,
}

#[derive(Deserialize)]
pub struct GameBoardInfo {
    pub width: u32,
    pub height: u32,
    pub config: HashMap<String, Value>,
    pub initial_objects: Option<Vec<GameObjectInfo>>,
}

#[derive(Deserialize)]
pub struct GameObjectInfo {
    pub id: String,
    pub object_type: String,
    pub x: f32,
    pub y: f32,
    pub state: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Deserialize)]
pub struct GameObjectUpdate {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub state: String,
    pub properties_update: HashMap<String, Value>,
}

#[derive(Deserialize)]
pub struct PlayerScore {
    pub player_id: String,
    pub name: String,
    pub score: u32,
}
```

## アプリケーションのライフサイクル

```
初期化 -> メインメニュー -> ゲームタイプ選択 -> オプション選択 -> ルーム作成/参加 -> ロビー -> ゲームプレイ -> ゲーム終了 -> メインメニュー
```

### 初期化処理
```rust
fn initialize() -> Result<(), JsValue> {
    // WAASMモジュールの初期化
    utils::set_panic_hook();
    
    // Canvasの取得と設定
    let canvas = document.get_element_by_id("game-canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()?;
    
    // ゲームアプリケーションの作成
    let app = GameApp::new(canvas)?;
    
    // グローバル状態として保存
    APP_STATE.with(|state| {
        *state.borrow_mut() = Some(app);
    });
    
    // メインループの設定
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        if let Some(app) = APP_STATE.with(|state| state.borrow().clone()) {
            // 前回のタイムスタンプを取得
            let last_timestamp = LAST_TIMESTAMP.with(|ts| {
                let current = *ts.borrow();
                *ts.borrow_mut() = timestamp;
                current
            });
            
            // デルタタイム計算（初回は0）
            let delta_time = if last_timestamp == 0.0 {
                0.0
            } else {
                (timestamp - last_timestamp) / 1000.0 // 秒単位に変換
            };
            
            // アプリケーションの更新とレンダリング
            app.borrow_mut().update(delta_time as f32).unwrap();
            app.borrow().render().unwrap();
        }
        
        // 次のアニメーションフレームをリクエスト
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));
    
    // アニメーションループ開始
    request_animation_frame(g.borrow().as_ref().unwrap());
    
    Ok(())
}
```

### ゲームプレイフロー
```rust
fn game_play_update(app: &mut GameApp, delta_time: f32) -> Result<(), JsValue> {
    // ネットワークメッセージの処理
    app.network.process_messages(app)?;
    
    // ワールドの更新
    app.world.update(delta_time);
    
    // UIの更新
    app.ui.update_for_state(&app.state)?;
    
    Ok(())
}
```

### ユーザー入力処理
```rust
fn handle_click(app: &mut GameApp, x: f32, y: f32, button: MouseButton) -> Result<(), JsValue> {
    // 現在の状態に基づいて処理を分岐
    match &app.state {
        AppState::Game(ref game_info) => {
            // クリック位置をゲームボード上の位置に変換
            if let Some(position) = app.renderer.screen_to_board_position(x, y) {
                // 操作タイプを決定
                let action = match button {
                    MouseButton::Left => ActionType::Primary,
                    MouseButton::Right => ActionType::Secondary,
                    MouseButton::Middle => ActionType::Tertiary,
                };
                
                // アクションをサーバーに送信
                app.network.perform_action(action, position.x, position.y, None)?;
            }
        },
        // その他の状態でのクリック処理...
        _ => {
            // 状態の可変参照が必要なため、matchの外で処理
            app.ui.handle_click(x, y, button, &mut app.state)?;
        }
    }
    
    Ok(())
}

// アクションタイプを表す列挙型
pub enum ActionType {
    Primary,
    Secondary,
    Tertiary,
    Custom(String),
}
```

## パフォーマンス最適化

### レンダリング最適化
```rust
fn optimized_rendering(renderer: &mut RenderSystem, world: &World) -> Result<(), JsValue> {
    // ダーティフラグによる再描画最適化
    let dirty_regions = world.get_dirty_regions();
    
    if dirty_regions.is_empty() && !renderer.full_redraw_needed {
        // 変更がない場合はスキップ
        return Ok(());
    }
    
    // 全体の再描画が必要な場合
    if renderer.full_redraw_needed {
        renderer.clear();
        renderer.render_background();
        renderer.render_board(world)?;
        renderer.full_redraw_needed = false;
        return Ok(());
    }
    
    // 変更された領域のみを再描画
    for region in dirty_regions {
        renderer.render_region(world, region)?;
    }
    
    Ok(())
}
```

### メモリ使用量の最適化
```rust
// イメージ・スプライトのキャッシュ
pub struct SpriteCache {
    sprites: HashMap<SpriteType, HtmlImageElement>,
    loaded_count: usize,
    total_count: usize,
    on_loaded: Option<Closure<dyn FnMut()>>,
}

impl SpriteCache {
    pub fn new() -> Self { ... }
    
    pub fn load_sprites(&mut self, on_complete: impl FnOnce() + 'static) -> Result<(), JsValue> { ... }
    
    pub fn get_sprite(&self, sprite_type: SpriteType) -> Option<&HtmlImageElement> { ... }
}
```

## エラーハンドリング

```rust
// エラー型定義
#[derive(Debug)]
pub enum GameError {
    NetworkError(String),
    RenderingError(String),
    LogicError(String),
    InputError(String),
    ResourceError(String),
    JsError(JsValue),
}

// エラー処理関数
fn handle_error(app: &mut GameApp, error: GameError) {
    log::error!("Game error: {:?}", error);
    
    // エラーメッセージをユーザーに表示
    app.state.transition_to(AppState::Error(error.to_string()));
    
    // UIでエラーを表示
    if let Err(e) = app.ui.show_error(&error) {
        log::error!("Failed to show error UI: {:?}", e);
        // フォールバック: JavaScriptのアラート
        web_sys::window()
            .unwrap()
            .alert_with_message(&format!("Error: {}", error))
            .unwrap();
    }
}
```

## ビルドと配信

```
# Rustコードをwasmにコンパイル
wasm-pack build --target web --out-dir www/pkg

# 開発サーバー起動
cd www
python -m http.server
```

### HTML・JS統合例

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>マルチプレイヤーゲームフレームワーク</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <!-- ゲームコンテナ -->
    <div id="game-container">
        <!-- ゲームキャンバス -->
        <canvas id="game-canvas" width="800" height="600"></canvas>
        
        <!-- UI要素 -->
        <div id="ui-container">
            <!-- 各画面のUIを動的に表示 -->
            <div id="main-menu" class="ui-screen">
                <h1>ゲームフレームワーク</h1>
                <button id="new-game-btn">新しいゲームを開始</button>
                <button id="join-game-btn">ゲームに参加する</button>
            </div>
            
            <!-- その他の画面... -->
        </div>
    </div>
    
    <!-- Wasmロード用スクリプト -->
    <script type="module">
        import init from './pkg/game_framework.js';
        
        async function run() {
            // Wasmモジュールを初期化
            await init();
        }
        
        run();
    </script>
</body>
</html>
```

### JavaScript統合コード

```javascript
// game-bindings.js
export function setupGameBindings(wasm_module) {
    window.gameFramework = {
        // UIイベントハンドラー
        onNewGameClick: () => {
            wasm_module.handle_new_game_click();
        },
        
        onJoinGameClick: () => {
            wasm_module.handle_join_game_click();
        },
        
        onGameTypeSelect: (type) => {
            wasm_module.handle_game_type_select(type);
        },
        
        onOptionsSelect: (options) => {
            wasm_module.handle_options_select(options);
        },
        
        onCreateRoomSubmit: (playerName) => {
            wasm_module.handle_create_room(playerName);
        },
        
        onJoinRoomSubmit: (playerName, roomCode) => {
            wasm_module.handle_join_room(playerName, roomCode);
        },
        
        onReadyToggle: () => {
            wasm_module.handle_ready_toggle();
        },
        
        onStartGameClick: () => {
            wasm_module.handle_start_game();
        },
        
        onLeaveRoomClick: () => {
            wasm_module.handle_leave_room();
        },
        
        onSendChatMessage: (message) => {
            wasm_module.handle_chat_message(message);
        },
        
        // その他のバインディング...
    };
    
    // 初期UIバインディングの設定
    setupUIBindings();
}

function setupUIBindings() {
    // 各UIボタンにイベントハンドラを設定
    document.getElementById('new-game-btn').addEventListener('click', () => {
        window.gameFramework.onNewGameClick();
    });
    
    // その他のUI要素へのバインディング...
}
``` 