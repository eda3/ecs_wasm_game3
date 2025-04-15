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
    // ネットワーククライアントとカーソルシステムの取得
    if let Some(mut network_client) = world.get_resource_mut::<NetworkClient>() {
        // マウスカーソルシステムを取得
        if let Some(cursor_system) = world.get_system_mut::<MouseCursorSystem>() {
            // 参照のライフタイム問題を回避するため、データをクローン
            let cursor_system_ptr = cursor_system as *mut MouseCursorSystem;
            
            // カーソル更新ハンドラを登録
            network_client.register_mouse_cursor_handler(move |data| {
                // 安全でない操作：参照外しによるシステムへのアクセス
                unsafe {
                    if let Some(system) = cursor_system_ptr.as_mut() {
                        let world_ptr = world as *mut World;
                        if let Some(world) = world_ptr.as_mut() {
                            system.handle_cursor_update(world, data);
                        }
                    }
                }
            });
        }
    }
    
    Ok(())
} 