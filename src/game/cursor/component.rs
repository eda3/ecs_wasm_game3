use crate::ecs::Component;
use wasm_bindgen::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// マウスカーソルコンポーネント
#[derive(Debug, Clone)]
pub struct MouseCursorComponent {
    /// プレイヤーID
    pub player_id: u32,
    /// X座標
    pub x: f32,
    /// Y座標
    pub y: f32,
    /// 表示状態
    pub visible: bool,
    /// カーソルの色 (RGB)
    pub color: (u8, u8, u8),
    /// カーソルの大きさ
    pub size: f32,
    /// カーソルの透明度
    pub opacity: f32,
}

impl MouseCursorComponent {
    /// 新しいマウスカーソルコンポーネントを作成
    pub fn new(player_id: u32, x: f32, y: f32) -> Self {
        // プレイヤーIDからカーソルの色を生成
        let color = Self::generate_color(player_id);
        
        Self {
            player_id,
            x,
            y,
            visible: true,
            color,
            size: 20.0,       // デフォルトは直径20px
            opacity: 0.5,     // デフォルトは50%の透明度
        }
    }
    
    /// プレイヤーIDからカーソルの色を生成する
    fn generate_color(player_id: u32) -> (u8, u8, u8) {
        // プレイヤーIDからハッシュ値を計算
        let mut hasher = DefaultHasher::new();
        player_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        // ハッシュ値から色相を決定（0-360）
        let hue = (hash % 360) as f32;
        let saturation = 0.8;  // 80%の彩度
        let value = 0.9;       // 90%の明度
        
        // HSVからRGBへ変換
        let c = value * saturation;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = value - c;
        
        let (r, g, b) = match hue as u32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        
        (
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }
    
    /// カーソル位置を更新
    pub fn update_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
    
    /// 表示状態を設定
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl Component for MouseCursorComponent {
    fn name() -> &'static str {
        "MouseCursorComponent"
    }
} 