pub mod component;
pub mod system;
pub mod rendering;

pub use component::MouseCursorComponent;
pub use system::MouseCursorSystem;
pub use rendering::MouseCursorRenderingSystem;

use crate::ecs::World;
use crate::network::client::NetworkClient;
use wasm_bindgen::prelude::*;

/// マウスカーソルの初期化を行う関数
pub fn init_mouse_cursor_system(world: &mut World) -> Result<(), JsValue> {
    // マウスカーソルシステムの作成と登録
    let cursor_system = MouseCursorSystem::new();
    world.register_system(cursor_system);
    
    // マウスカーソル描画システムの作成と登録
    let cursor_rendering_system = MouseCursorRenderingSystem::new();
    world.register_system(cursor_rendering_system);
    
    Ok(())
}

/// リソースからネットワーククライアントを取得してマウスカーソルハンドラを登録する
pub fn register_mouse_cursor_handler(world: &mut World) -> Result<(), JsValue> {
    // ネットワーククライアントを取得
    if let Some(mut network_client) = world.get_resource_mut::<NetworkClient>() {
        // マウスカーソルシステムは直接取得できないので、
        // カーソル更新データをNetworkClientで保持し、次のフレームで処理する
        network_client.register_mouse_cursor_handler(move |data| {
            // カーソル更新データの受信をログ出力
            let _ = web_sys::console::log_1(&format!(
                "📍 マウスカーソル更新を受信: player_id={}, pos=({:.1},{:.1}), visible={}", 
                data.player_id, data.x, data.y, data.visible
            ).into());
            
            // NetworkClientはカーソルデータをキャッシュし、
            // 次のMouseCursorSystemの更新時に処理される
        });
        
        Ok(())
    } else {
        Err(JsValue::from_str("NetworkClientリソースが見つかりません"))
    }
} 