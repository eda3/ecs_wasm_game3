use crate::ecs::{System, World, ResourceManager, SystemPhase, SystemPriority, Entity};
use crate::input::InputResource;
use crate::network::client::NetworkClient;
use crate::network::protocol::MouseCursorUpdateData;
use wasm_bindgen::prelude::*;
use super::component::MouseCursorComponent;
use std::collections::HashMap;
use web_sys::console;

/// マウスカーソルシステム
pub struct MouseCursorSystem {
    /// ローカルプレイヤーID
    local_player_id: Option<u32>,
    /// 最後のマウス位置
    last_position: (f32, f32),
    /// マウスカーソルのエンティティID
    local_cursor_entity: Option<Entity>,
    /// 他プレイヤーのカーソルマップ (player_id -> entity)
    player_cursors: HashMap<u32, Entity>,
    /// 同期間隔（ミリ秒）
    sync_interval: f64,
    /// 最後に同期した時間
    last_sync_time: f64,
}

impl MouseCursorSystem {
    /// 新しいマウスカーソルシステムを作成
    pub fn new() -> Self {
        Self {
            local_player_id: None,
            last_position: (0.0, 0.0),
            local_cursor_entity: None,
            player_cursors: HashMap::new(),
            sync_interval: 100.0, // デフォルト100ms
            last_sync_time: js_sys::Date::now(),
        }
    }
    
    /// マウスカーソル更新を処理
    pub fn handle_cursor_update(&mut self, world: &mut World, data: MouseCursorUpdateData) {
        // 自分自身のカーソル更新は無視（すでにローカルで反映済み）
        if let Some(player_id) = self.local_player_id {
            if data.player_id == player_id {
                return;
            }
        }
        
        // 既存のプレイヤーカーソルを探す
        if let Some(entity) = self.player_cursors.get(&data.player_id) {
            // 既存のカーソルコンポーネントを更新
            if let Some(cursor) = world.get_component_mut::<MouseCursorComponent>(*entity) {
                cursor.update_position(data.x, data.y);
                cursor.set_visible(data.visible);
            }
        } else {
            // 新しいカーソルエンティティを作成
            let new_entity = world.create_entity();
            let cursor = MouseCursorComponent::new(data.player_id, data.x, data.y);
            
            world.add_component(new_entity, cursor);
            self.player_cursors.insert(data.player_id, new_entity);
            
            // ログ出力
            console::log_1(&format!("📍 Created cursor entity for player: {}", data.player_id).into());
        }
    }
}

impl System for MouseCursorSystem {
    fn name(&self) -> &'static str {
        "MouseCursorSystem"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Input // 入力フェーズで実行
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(100) // 高い優先度
    }
    
    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        let now = js_sys::Date::now();
        
        // NetworkClientからプレイヤーIDを取得（まだ設定されていない場合）
        if self.local_player_id.is_none() {
            if let Some(network_client) = resources.get::<NetworkClient>() {
                self.local_player_id = network_client.get_player_id();
            }
        }
        
        // 接続済みで、プレイヤーIDがある場合のみ処理
        if let Some(player_id) = self.local_player_id {
            // InputResourceからマウス位置を取得
            if let Some(input_resource) = resources.get::<InputResource>() {
                let mouse_pos = input_resource.get_mouse_position();
                let is_in_canvas = input_resource.is_mouse_in_canvas();
                
                // ローカルカーソルエンティティが未作成なら作成
                if self.local_cursor_entity.is_none() {
                    let cursor = MouseCursorComponent::new(player_id, mouse_pos.0, mouse_pos.1);
                    
                    let entity = world.create_entity();
                    world.add_component(entity, cursor);
                    self.local_cursor_entity = Some(entity);
                    
                    console::log_1(&format!("🖱️ Created local cursor for player: {}", player_id).into());
                }
                
                // ローカルカーソルを更新
                if let Some(entity) = self.local_cursor_entity {
                    if let Some(cursor) = world.get_component_mut::<MouseCursorComponent>(entity) {
                        // 位置が変わったか、表示状態が変わった場合に更新
                        let position_changed = 
                            (self.last_position.0 - mouse_pos.0).abs() > 1.0 || 
                            (self.last_position.1 - mouse_pos.1).abs() > 1.0;
                        
                        if position_changed || cursor.visible != is_in_canvas {
                            cursor.update_position(mouse_pos.0, mouse_pos.1);
                            cursor.set_visible(is_in_canvas);
                            
                            // 前回の同期から十分な時間が経過していれば同期
                            if now - self.last_sync_time >= self.sync_interval {
                                if let Some(network_client) = resources.get_mut::<NetworkClient>() {
                                    network_client.send_mouse_cursor_update(
                                        mouse_pos.0, 
                                        mouse_pos.1, 
                                        is_in_canvas
                                    ).ok();
                                    
                                    self.last_sync_time = now;
                                }
                            }
                            
                            self.last_position = mouse_pos;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
} 