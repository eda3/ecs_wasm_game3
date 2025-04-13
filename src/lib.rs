use wasm_bindgen::prelude::*;
use web_sys::console;
use std::sync::Mutex;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::cell::RefCell;

// モジュール宣言
pub mod ecs;
pub mod game;
pub mod rendering;
pub mod physics;
pub mod input;
pub mod network;
pub mod utils;

// グローバルクライアント管理用
thread_local! {
    static NETWORK_CLIENTS: RefCell<HashMap<String, Rc<RefCell<network::client::NetworkClient>>>> = 
        RefCell::new(HashMap::new());
    static GAME_INSTANCES: RefCell<HashMap<String, Weak<RefCell<GameInstance>>>> = 
        RefCell::new(HashMap::new());
}

// 初期化用のエントリーポイント
#[wasm_bindgen(start)]
pub fn start() {
    // エラーをコンソールにパニックフックとして表示
    console_error_panic_hook::set_once();
    
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
    // 直接参照ではなく、IDで参照する
    network_client_id: Option<String>,
    last_update_time: f64,
    instance_id: String,
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
        
        // インスタンスIDを生成
        let instance_id = format!("game_{}", js_sys::Date::now());
        
        // インスタンスを作成
        let instance = GameInstance {
            world,
            network_client_id: None,
            last_update_time: js_sys::Date::now(),
            instance_id: instance_id.clone(),
        };
        
        // グローバルストアに弱参照として保存
        let rc_instance = Rc::new(RefCell::new(instance));
        let weak_ref = Rc::downgrade(&rc_instance);
        GAME_INSTANCES.with(|instances| {
            instances.borrow_mut().insert(instance_id.clone(), weak_ref);
        });
        
        // インスタンスを返す
        Ok(GameInstance {
            world: ecs::World::new(),
            network_client_id: None,
            last_update_time: js_sys::Date::now(),
            instance_id,
        })
    }
    
    // サーバーに接続
    #[wasm_bindgen]
    pub fn connect_to_server(&mut self, server_url: &str) -> Result<(), JsValue> {
        log::info!("Connecting to server: {}", server_url);
        
        // 既存の接続があれば削除
        if let Some(client_id) = &self.network_client_id {
            NETWORK_CLIENTS.with(|clients| {
                clients.borrow_mut().remove(client_id);
            });
        }
        
        // 新しいクライアントIDを生成
        let client_id = format!("client_{}", js_sys::Date::now());
        
        // ネットワーク設定の作成
        let network_config = network::NetworkConfig {
            server_url: server_url.to_string(),
            ..Default::default()
        };
        
        // ネットワーククライアントの作成（循環参照なし）
        let client = network::client::NetworkClient::new(network_config);
        
        // グローバルマップに保存
        NETWORK_CLIENTS.with(|clients| {
            clients.borrow_mut().insert(client_id.clone(), Rc::new(RefCell::new(client)));
        });
        self.network_client_id = Some(client_id.clone());
        
        // クライアントの接続を試行
        let result = NETWORK_CLIENTS.with(|clients| {
            let clients = clients.borrow();
            if let Some(client_rc) = clients.get(&client_id) {
                let mut client = client_rc.borrow_mut();
                
                // 接続を試行
                match client.connect() {
                    Ok(_) => {
                        log::info!("Connection initiated successfully");
                        
                        // ネットワークコンポーネントをワールドに追加
                        let network_resource = network::NetworkResource::new(server_url.to_string());
                        self.world.insert_resource(network_resource);
                        
                        Ok(())
                    },
                    Err(err) => {
                        let error_msg = format!("Failed to connect: {:?}", err);
                        log::error!("{}", error_msg);
                        Err(JsValue::from_str(&error_msg))
                    }
                }
            } else {
                Err(JsValue::from_str("Failed to store network client"))
            }
        });
        
        result
    }
    
    // サーバーから切断
    #[wasm_bindgen]
    pub fn disconnect_from_server(&mut self) -> Result<(), JsValue> {
        if let Some(client_id) = &self.network_client_id {
            let result = NETWORK_CLIENTS.with(|clients| {
                let clients = clients.borrow();
                if let Some(client_rc) = clients.get(client_id) {
                    let mut client = client_rc.borrow_mut();
                    match client.disconnect() {
                        Ok(_) => {
                            log::info!("Disconnected from server");
                            Ok(())
                        },
                        Err(err) => {
                            let error_msg = format!("Failed to disconnect: {:?}", err);
                            log::error!("{}", error_msg);
                            Err(JsValue::from_str(&error_msg))
                        }
                    }
                } else {
                    Ok(()) // クライアントが既に存在しない
                }
            });
            
            if result.is_ok() {
                self.network_client_id = None;
            }
            
            result
        } else {
            Ok(()) // 既に切断済み
        }
    }
    
    // 接続状態を取得
    #[wasm_bindgen]
    pub fn get_connection_state(&self) -> String {
        if let Some(client_id) = &self.network_client_id {
            NETWORK_CLIENTS.with(|clients| {
                let clients = clients.borrow();
                if let Some(client_rc) = clients.get(client_id) {
                    let client = client_rc.borrow();
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
            })
        } else {
            "disconnected".to_string()
        }
    }
    
    // ゲームのメインループを1フレーム進める
    #[wasm_bindgen]
    pub fn update(&mut self) -> f32 {
        // フレーム間の時間を計算（安全対策付き）
        let current_time = js_sys::Date::now();
        let mut delta_time = (current_time - self.last_update_time) as f32 / 1000.0;
        
        // デルタタイムを安全な範囲に制限
        if delta_time.is_nan() || delta_time <= 0.0 || delta_time > 0.5 {
            delta_time = 0.016; // ~60FPS相当のデフォルト値
        }
        
        self.last_update_time = current_time;
        
        // ネットワーククライアントの更新（安全な方法で）
        if let Some(client_id) = &self.network_client_id {
            NETWORK_CLIENTS.with(|clients| {
                let clients = clients.borrow();
                if let Some(client_rc) = clients.get(client_id) {
                    let mut client = client_rc.borrow_mut();
                    
                    // エラー処理を強化
                    if let Err(err) = client.update(&mut self.world) {
                        log::warn!("Network update error: {:?}", err);
                        // エラーが発生しても続行
                    }
                }
            });
        }
        
        // ワールドの更新（安全に）
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
    
    /// キーイベントを処理
    pub fn handle_key_event(&mut self, key_code: u32) -> Result<(), JsValue> {
        // InputSystem取得方法の修正
        if let Some(input_resource) = self.world.get_resource_mut::<input::InputResource>() {
            // 適切なInputResource経由でキーイベントを処理
            let event = input::KeyboardEvent {
                key: key_code.to_string(),
                event_type: "keydown".to_string(),
            };
            input_resource.handle_keyboard_event(&event);
            return Ok(())
        }
        
        // InputSystemが見つからない場合のエラー処理
        log::warn!("InputSystem not found, key event ignored");
        Ok(())
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
        
        // 入力システムを取得して処理を委譲
        if let Some(input_system) = self.get_input_system() {
            input_system.handle_mouse_event(&event);
        } else {
            log::warn!("入力システムが見つかりません");
        }
    }
    
    // 入力システムを取得
    fn get_input_system(&mut self) -> Option<&mut input::InputSystem> {
        // InputResourceからInputSystemを取得する
        self.world.get_resource_mut::<input::InputResource>()
            .map(|input_resource| &mut input_resource.system)
    }
    
    // 解放時の処理
    #[wasm_bindgen]
    pub fn dispose(&mut self) {
        // インスタンスをグローバルマップから削除
        GAME_INSTANCES.with(|instances| {
            instances.borrow_mut().remove(&self.instance_id);
        });
        
        // ネットワーククライアントを切断して削除
        if let Some(client_id) = self.network_client_id.take() {
            NETWORK_CLIENTS.with(|clients| {
                let client_opt = {
                    let clients_ref = clients.borrow();
                    clients_ref.get(&client_id).map(|c| c.clone())
                };
                
                if let Some(client_rc) = client_opt {
                    let mut client = client_rc.borrow_mut();
                    let _ = client.disconnect(); // エラーは無視
                }
                
                clients.borrow_mut().remove(&client_id);
            });
        }
    }
}
