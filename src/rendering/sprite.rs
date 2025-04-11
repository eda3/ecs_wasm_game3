//! スプライトモジュール
//! 
//! ゲームのスプライトシステムを管理します。
//! スプライトの描画やアニメーションを制御します。

use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;
use std::collections::HashMap;

/// スプライト構造体
/// 
/// ゲームのスプライトを表します。
pub struct Sprite {
    /// 画像ID
    pub image_id: String,
    /// 幅
    pub width: f64,
    /// 高さ
    pub height: f64,
    /// ソースX座標
    pub source_x: f64,
    /// ソースY座標
    pub source_y: f64,
    /// ソース幅
    pub source_width: f64,
    /// ソース高さ
    pub source_height: f64,
    /// X方向のスケール
    pub scale_x: f64,
    /// Y方向のスケール
    pub scale_y: f64,
    /// 回転角度（ラジアン）
    pub rotation: f64,
    /// ピボットX座標（0.0〜1.0）
    pub pivot_x: f64,
    /// ピボットY座標（0.0〜1.0）
    pub pivot_y: f64,
    /// 水平反転
    pub flip_x: bool,
    /// 垂直反転
    pub flip_y: bool,
    /// 可視性
    pub visible: bool,
    /// 透明度（0.0〜1.0）
    pub opacity: f64,
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

    /// スケールを設定
    pub fn set_scale(&mut self, scale_x: f64, scale_y: f64) {
        self.scale_x = scale_x;
        self.scale_y = scale_y;
    }

    /// 回転を設定
    pub fn set_rotation(&mut self, rotation: f64) {
        self.rotation = rotation;
    }

    /// ピボットを設定
    pub fn set_pivot(&mut self, pivot_x: f64, pivot_y: f64) {
        self.pivot_x = pivot_x.max(0.0).min(1.0);
        self.pivot_y = pivot_y.max(0.0).min(1.0);
    }

    /// 反転を設定
    pub fn set_flip(&mut self, flip_x: bool, flip_y: bool) {
        self.flip_x = flip_x;
        self.flip_y = flip_y;
    }

    /// 可視性を設定
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// 透明度を設定
    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.max(0.0).min(1.0);
    }
}

/// スプライトマネージャー構造体
/// 
/// 複数のスプライトを管理します。
pub struct SpriteManager {
    /// スプライトのマップ
    sprites: HashMap<String, Sprite>,
    /// 画像のマップ
    images: HashMap<String, HtmlImageElement>,
}

impl SpriteManager {
    /// 新しいスプライトマネージャーを作成
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            images: HashMap::new(),
        }
    }

    /// スプライトを追加
    pub fn add_sprite(&mut self, id: String, sprite: Sprite) {
        self.sprites.insert(id, sprite);
    }

    /// スプライトを取得
    pub fn get_sprite(&self, id: &str) -> Option<&Sprite> {
        self.sprites.get(id)
    }

    /// スプライトを取得（可変）
    pub fn get_sprite_mut(&mut self, id: &str) -> Option<&mut Sprite> {
        self.sprites.get_mut(id)
    }

    /// スプライトを削除
    pub fn remove_sprite(&mut self, id: &str) {
        self.sprites.remove(id);
    }

    /// 画像を追加
    pub fn add_image(&mut self, id: String, image: HtmlImageElement) {
        self.images.insert(id, image);
    }

    /// 画像を取得
    pub fn get_image(&self, id: &str) -> Option<&HtmlImageElement> {
        self.images.get(id)
    }

    /// 画像を削除
    pub fn remove_image(&mut self, id: &str) {
        self.images.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::JsValue;

    #[test]
    fn test_sprite_creation() {
        let sprite = Sprite::new(
            "test_image".to_string(),
            100.0,
            100.0,
            0.0,
            0.0,
            100.0,
            100.0,
        );
        
        assert_eq!(sprite.image_id, "test_image");
        assert_eq!(sprite.width, 100.0);
        assert_eq!(sprite.height, 100.0);
        assert_eq!(sprite.scale_x, 1.0);
        assert_eq!(sprite.scale_y, 1.0);
        assert_eq!(sprite.rotation, 0.0);
        assert_eq!(sprite.pivot_x, 0.5);
        assert_eq!(sprite.pivot_y, 0.5);
        assert!(!sprite.flip_x);
        assert!(!sprite.flip_y);
        assert!(sprite.visible);
        assert_eq!(sprite.opacity, 1.0);
    }

    #[test]
    fn test_sprite_properties() {
        let mut sprite = Sprite::new(
            "test_image".to_string(),
            100.0,
            100.0,
            0.0,
            0.0,
            100.0,
            100.0,
        );
        
        // スケール設定
        sprite.set_scale(2.0, 3.0);
        assert_eq!(sprite.scale_x, 2.0);
        assert_eq!(sprite.scale_y, 3.0);
        
        // 回転設定
        sprite.set_rotation(1.57);
        assert_eq!(sprite.rotation, 1.57);
        
        // ピボット設定
        sprite.set_pivot(0.25, 0.75);
        assert_eq!(sprite.pivot_x, 0.25);
        assert_eq!(sprite.pivot_y, 0.75);
        
        // 反転設定
        sprite.set_flip(true, false);
        assert!(sprite.flip_x);
        assert!(!sprite.flip_y);
        
        // 可視性設定
        sprite.set_visible(false);
        assert!(!sprite.visible);
        
        // 透明度設定
        sprite.set_opacity(0.5);
        assert_eq!(sprite.opacity, 0.5);
        
        // 透明度の範囲チェック
        sprite.set_opacity(-1.0);
        assert_eq!(sprite.opacity, 0.0);
        sprite.set_opacity(2.0);
        assert_eq!(sprite.opacity, 1.0);
    }

    #[test]
    fn test_sprite_manager() {
        let mut manager = SpriteManager::new();
        
        let sprite = Sprite::new(
            "test_image".to_string(),
            100.0,
            100.0,
            0.0,
            0.0,
            100.0,
            100.0,
        );
        
        // スプライトの追加
        manager.add_sprite("test_sprite".to_string(), sprite);
        assert!(manager.get_sprite("test_sprite").is_some());
        
        // スプライトの取得（可変）
        if let Some(sprite) = manager.get_sprite_mut("test_sprite") {
            sprite.set_scale(2.0, 2.0);
            assert_eq!(sprite.scale_x, 2.0);
            assert_eq!(sprite.scale_y, 2.0);
        }
        
        // スプライトの削除
        manager.remove_sprite("test_sprite");
        assert!(manager.get_sprite("test_sprite").is_none());
    }
} 