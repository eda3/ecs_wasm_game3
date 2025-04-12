use wasm_bindgen::prelude::*;
use web_sys::console;

// モジュール宣言
pub mod ecs;
pub mod game;
pub mod rendering;
pub mod physics;
pub mod input;
pub mod network;
pub mod utils;

// 初期化用のエントリーポイント
#[wasm_bindgen(start)]
pub fn start() {
    // ロガーの初期化
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("WebAssembly module initialized!");
}

// ロガー初期化用のエクスポート関数
#[wasm_bindgen]
pub fn wasm_logger_init() {
    wasm_logger::init(wasm_logger::Config::default());
}

// ゲームインスタンスを作成するエクスポート関数
#[wasm_bindgen]
pub fn initialize_game(canvas_id: &str) -> Result<GameInstance, JsValue> {
    // ゲームインスタンスを初期化して返す
    let game = GameInstance::new(canvas_id)?;
    Ok(game)
}

// JavaScriptからアクセス可能なゲームインスタンス
#[wasm_bindgen]
pub struct GameInstance {
    // ゲームワールドやリソースへの参照を保持する
    world: ecs::World,
    network_client: Option<network::client::NetworkClient>,
    last_update_time: f64,
}

#[wasm_bindgen]
impl GameInstance {
    // 新しいゲームインスタンスを作成
    pub fn new(canvas_id: &str) -> Result<GameInstance, JsValue> {
        console::log_1(&"Creating new game instance".into());
        
        // ワールドを初期化
        let mut world = ecs::World::new();
        
        // レンダリングシステムの初期化
        rendering::init_rendering_system(&mut world, canvas_id)?;
        
        // 物理システムの初期化
        physics::init_physics_system(&mut world);
        
        // 入力システムの初期化
        input::init_input_system(&mut world);
        
        // ゲームシステムの初期化
        game::init_game_systems(&mut world);
        
        Ok(GameInstance {
            world,
            network_client: None,
            last_update_time: js_sys::Date::now(),
        })
    }
    
    // サーバーに接続
    #[wasm_bindgen]
    pub fn connect_to_server(&mut self, server_url: &str) -> Result<(), JsValue> {
        log::info!("Connecting to server: {}", server_url);
        
        // ネットワーク設定の作成
        let network_config = network::NetworkConfig {
            server_url: server_url.to_string(),
            ..Default::default()
        };
        
        // ネットワーククライアントの作成
        let mut client = network::client::NetworkClient::new(network_config);
        
        // サーバーへの接続を試行
        match client.connect() {
            Ok(_) => {
                log::info!("Connection initiated successfully");
                self.network_client = Some(client);
                
                // ネットワークコンポーネントをワールドに追加
                let network_resource = network::NetworkResource::new(server_url.to_string());
                self.world.add_resource(network_resource);
                
                Ok(())
            },
            Err(err) => {
                let error_msg = format!("Failed to connect: {:?}", err);
                log::error!("{}", error_msg);
                Err(JsValue::from_str(&error_msg))
            }
        }
    }
    
    // サーバーから切断
    #[wasm_bindgen]
    pub fn disconnect_from_server(&mut self) -> Result<(), JsValue> {
        if let Some(client) = &mut self.network_client {
            match client.disconnect() {
                Ok(_) => {
                    log::info!("Disconnected from server");
                    self.network_client = None;
                    Ok(())
                },
                Err(err) => {
                    let error_msg = format!("Failed to disconnect: {:?}", err);
                    log::error!("{}", error_msg);
                    Err(JsValue::from_str(&error_msg))
                }
            }
        } else {
            Ok(()) // 既に切断済み
        }
    }
    
    // 接続状態を取得
    #[wasm_bindgen]
    pub fn get_connection_state(&self) -> String {
        if let Some(client) = &self.network_client {
            match client.get_connection_state() {
                network::ConnectionState::Connected => "connected",
                network::ConnectionState::Connecting => "connecting",
                network::ConnectionState::Disconnected => "disconnected",
                network::ConnectionState::Disconnecting => "disconnecting",
                network::ConnectionState::Error(msg) => {
                    log::error!("Connection error: {}", msg);
                    "error"
                }
            }.to_string()
        } else {
            "disconnected".to_string()
        }
    }
    
    // ゲームのメインループを1フレーム進める
    #[wasm_bindgen]
    pub fn update(&mut self) -> f32 {
        // フレーム間の時間を計算
        let current_time = js_sys::Date::now();
        let delta_time = (current_time - self.last_update_time) as f32 / 1000.0;
        self.last_update_time = current_time;
        
        // ネットワーククライアントの更新
        if let Some(client) = &mut self.network_client {
            if let Err(err) = client.update(&mut self.world) {
                log::warn!("Network update error: {:?}", err);
            }
        }
        
        // ワールドの更新
        self.world.update(delta_time);
        
        // デルタタイムを返す（パフォーマンスメトリクス用）
        delta_time
    }
    
    // ゲームを描画
    #[wasm_bindgen]
    pub fn render(&mut self) {
        // レンダリングシステムによる描画
        self.world.render();
    }
    
    // キー入力を処理
    #[wasm_bindgen]
    pub fn handle_key_event(&mut self, event_type: &str, key_code: &str) {
        // 入力システムにイベントを送信
        let event = input::KeyboardEvent {
            event_type: event_type.to_string(),
            key: key_code.to_string(),
        };
        
        self.world.handle_keyboard_event(event);
    }
    
    // マウス入力を処理
    #[wasm_bindgen]
    pub fn handle_mouse_event(&mut self, event_type: &str, x: f32, y: f32, button: Option<i32>) {
        // 入力システムにイベントを送信
        let event = input::MouseEvent {
            event_type: event_type.to_string(),
            position: (x, y),
            button,
        };
        
        self.world.handle_mouse_event(event);
    }
}
