use crate::ecs::{System, World, ResourceManager, SystemPhase, SystemPriority};
use crate::rendering::Renderer;
use wasm_bindgen::prelude::*;
use super::component::MouseCursorComponent;

/// マウスカーソル描画システム
pub struct MouseCursorRenderingSystem;

impl MouseCursorRenderingSystem {
    /// 新しいマウスカーソル描画システムを作成
    pub fn new() -> Self {
        Self {}
    }
}

impl System for MouseCursorRenderingSystem {
    fn name(&self) -> &'static str {
        "MouseCursorRenderingSystem"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Render // 描画フェーズで実行
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(900) // 通常の描画より後（最前面に表示）
    }
    
    fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        // 直接Canvasで描画する
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(canvas) = document.get_element_by_id("game-canvas") {
                    if let Ok(canvas) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
                        if let Ok(Some(context)) = canvas.get_context("2d") {
                            if let Ok(context) = context.dyn_into::<web_sys::CanvasRenderingContext2d>() {
                                // マウスカーソルコンポーネントを持つすべてのエンティティをレンダリング
                                let mut query = world.query::<MouseCursorComponent>();
                                for (_entity, cursor) in query.iter(world) {
                                    if cursor.visible {
                                        // カーソルの円を描画
                                        let (r, g, b) = cursor.color;
                                        let color_str = format!("rgba({}, {}, {}, {})", r, g, b, cursor.opacity);
                                        
                                        context.save();
                                        context.begin_path();
                                        context.set_fill_style(&JsValue::from_str(&color_str));
                                        
                                        // 円を描画
                                        context.arc(
                                            cursor.x as f64,
                                            cursor.y as f64,
                                            cursor.size as f64 / 2.0, // 半径なので直径の半分
                                            0.0,
                                            std::f64::consts::PI * 2.0,
                                        )?;
                                        
                                        context.fill();
                                        
                                        // プレイヤーIDを描画
                                        context.set_font("10px Arial");
                                        context.set_fill_style(&JsValue::from_str("white"));
                                        context.set_text_align("center");
                                        context.set_text_baseline("top");
                                        
                                        context.fill_text(
                                            &format!("Player {}", cursor.player_id),
                                            cursor.x as f64,
                                            (cursor.y + cursor.size / 2.0 + 5.0) as f64,
                                        )?;
                                        
                                        context.restore();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
} 