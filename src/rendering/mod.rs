//! レンダリングシステムモジュール
//! 
//! このモジュールは、ゲームの2Dレンダリングを担当します。
//! スプライト、アニメーション、UI要素などの描画を管理します。

mod sprite;
mod animation;
mod camera;
mod layer;

pub use sprite::*;
pub use animation::*;
pub use camera::*;
pub use layer::*;

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use std::collections::HashMap;
use std::time::Duration;
use crate::ecs::Resource;

/// レンダリングシステムを初期化
pub fn init_rendering_system(world: &mut crate::ecs::World, canvas_id: &str) -> Result<(), JsValue> {
    log::warn!("🖌️ レンダリングシステム初期化開始: canvas_id = {}", canvas_id);
    
    // 直接キャンバスをテスト
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window not available"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("document not available"))?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str("canvas not found"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
        
    log::warn!("✅ レンダリング初期化: キャンバス取得成功 {}x{}", canvas.width(), canvas.height());
    
    let ctx = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        
    // テスト描画
    ctx.set_fill_style_str("#00FFCC");
    ctx.fill_rect(200.0, 200.0, 100.0, 100.0);
    log::warn!("💚 レンダリング初期化内: テスト四角を描画");
    
    // レンダラーの初期化
    match Renderer::new(canvas_id) {
        Ok(renderer) => {
            log::warn!("✅ Renderer初期化成功！");
            
            // レンダリングリソースをワールドに追加
            world.insert_resource(renderer);
            log::warn!("✅ Rendererをワールドに登録完了");
            
            Ok(())
        },
        Err(e) => {
            log::error!("❌ Renderer初期化失敗: {:?}", e);
            Err(e)
        }
    }
}

/// レンダラー構造体
/// 
/// キャンバスとコンテキストを管理し、描画コマンドを実行します。
pub struct Renderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    sprites: HashMap<String, Sprite>,
    animation_manager: AnimationManager,
    camera: Camera,
    layers: Vec<RenderLayer>,
}

// Renderer を Resource として実装
impl Resource for Renderer {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// スプライト構造体
/// 
/// 描画する画像の情報を保持します。
pub struct Sprite {
    image_id: String,
    width: f64,
    height: f64,
    source_x: f64,
    source_y: f64,
    source_width: f64,
    source_height: f64,
    scale_x: f64,
    scale_y: f64,
    rotation: f64,
    pivot_x: f64,
    pivot_y: f64,
    flip_x: bool,
    flip_y: bool,
    visible: bool,
    opacity: f64,
}

impl Renderer {
    /// 新しいレンダラーを作成
    pub fn new(canvas_id: &str) -> Result<Renderer, JsValue> {
        let document = web_sys::window()
            .ok_or_else(|| JsValue::from_str("Failed to get window"))?
            .document()
            .ok_or_else(|| JsValue::from_str("Failed to get document"))?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("Failed to get canvas element"))?
            .dyn_into::<HtmlCanvasElement>()?;

        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        Ok(Renderer {
            canvas,
            context,
            sprites: HashMap::new(),
            animation_manager: AnimationManager::new(),
            camera: Camera::new(),
            layers: Vec::new(),
        })
    }

    /// レンダリングレイヤーを追加
    pub fn add_layer(&mut self, layer: RenderLayer) {
        self.layers.push(layer);
    }

    /// レンダリングレイヤーを取得
    pub fn get_layer(&mut self, name: &str) -> Option<&mut RenderLayer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    /// カメラを更新
    pub fn update_camera(&mut self, delta_time: Duration) {
        self.camera.update(delta_time);
    }

    /// スプライトを登録
    pub fn register_sprite(&mut self, sprite: Sprite) {
        self.sprites.insert(sprite.image_id.clone(), sprite);
    }

    /// スプライトを描画
    pub fn draw_sprite(&self, sprite_id: &str, x: f64, y: f64) -> Result<(), JsValue> {
        if let Some(sprite) = self.sprites.get(sprite_id) {
            if !sprite.visible {
                return Ok(());
            }

            let document = web_sys::window()
                .ok_or_else(|| JsValue::from_str("Failed to get window"))?
                .document()
                .ok_or_else(|| JsValue::from_str("Failed to get document"))?;

            let image = document
                .create_element("img")?
                .dyn_into::<HtmlImageElement>()?;
            image.set_src(&sprite.image_id);

            // カメラ変換を適用
            let (screen_x, screen_y) = self.camera.world_to_screen(x, y);

            // 描画変換を適用
            self.context.save();
            
            // 位置設定
            self.context.translate(screen_x, screen_y);
            
            // 回転設定
            if sprite.rotation != 0.0 {
                self.context.rotate(sprite.rotation);
            }
            
            // スケール設定
            if sprite.scale_x != 1.0 || sprite.scale_y != 1.0 {
                self.context.scale(sprite.scale_x, sprite.scale_y);
            }
            
            // 反転設定
            if sprite.flip_x || sprite.flip_y {
                self.context.scale(
                    if sprite.flip_x { -1.0 } else { 1.0 },
                    if sprite.flip_y { -1.0 } else { 1.0 }
                );
            }
            
            // 透明度設定
            if sprite.opacity != 1.0 {
                self.context.set_global_alpha(sprite.opacity);
            }

            self.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                sprite.source_x,
                sprite.source_y,
                sprite.source_width,
                sprite.source_height,
                -sprite.width * sprite.pivot_x,
                -sprite.height * sprite.pivot_y,
                sprite.width,
                sprite.height,
            )?;

            self.context.restore();
        }
        Ok(())
    }

    /// アニメーションを追加
    pub fn add_animation(&mut self, id: String, animation: Animation) {
        self.animation_manager.add_animation(id, animation);
    }

    /// アニメーションを取得
    pub fn get_animation(&mut self, id: &str) -> Option<&mut Animation> {
        self.animation_manager.get_animation(id)
    }

    /// アニメーションを更新
    pub fn update_animations(&mut self, delta_time: Duration) {
        self.animation_manager.update(delta_time);
    }

    /// キャンバスをクリア
    pub fn clear(&self) {
        self.context.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
    }

    /// キャンバスのサイズを取得
    pub fn get_size(&self) -> (f64, f64) {
        (
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        )
    }

    /// キャンバスのサイズを設定
    pub fn set_size(&mut self, width: f64, height: f64) {
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
    }
}

impl Sprite {
    /// 新しいスプライトを作成
    pub fn new(
        image_id: String,
        width: f64,
        height: f64,
        source_x: f64,
        source_y: f64,
        source_width: f64,
        source_height: f64,
    ) -> Self {
        Self {
            image_id,
            width,
            height,
            source_x,
            source_y,
            source_width,
            source_height,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            pivot_x: 0.5,
            pivot_y: 0.5,
            flip_x: false,
            flip_y: false,
            visible: true,
            opacity: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    use std::time::Duration;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_renderer_creation() {
        // テスト用のキャンバスを作成
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.create_element("canvas").unwrap();
        canvas.set_id("test_canvas");
        document.body().unwrap().append_child(&canvas).unwrap();

        // レンダラーの作成をテスト
        let renderer = Renderer::new("test_canvas");
        assert!(renderer.is_ok());

        // クリーンアップ
        document.body().unwrap().remove_child(&canvas).unwrap();
    }

    #[test]
    fn test_animation_integration() {
        let mut renderer = Renderer::new("test_canvas").unwrap();
        
        // アニメーションの追加をテスト
        let frames = vec![
            AnimationFrame::new("frame1".to_string(), Duration::from_millis(100)),
            AnimationFrame::new("frame2".to_string(), Duration::from_millis(100)),
        ];
        let animation = Animation::new(frames, true);
        renderer.add_animation("test_anim".to_string(), animation);
        
        // アニメーションの取得をテスト
        assert!(renderer.get_animation("test_anim").is_some());
        
        // アニメーションの更新をテスト
        renderer.update_animations(Duration::from_millis(50));
    }
} 