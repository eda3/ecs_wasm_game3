//! 入力処理システムの実装
//! 
//! このモジュールは、キーボード、マウス、タッチなどの入力処理を担当します。
//! 入力状態の管理とイベントの処理を行います。

use crate::ecs::{Component, Entity, World};
use wasm_bindgen::prelude::*;

/// 入力状態を管理する構造体
#[derive(Debug)]
pub struct InputState {
    pub keys: [bool; 256],
    pub mouse_position: [f32; 2],
    pub mouse_buttons: [bool; 3],
    pub touch_points: Vec<TouchPoint>,
}

/// タッチポイントの情報
#[derive(Debug)]
pub struct TouchPoint {
    pub id: u32,
    pub position: [f32; 2],
    pub is_active: bool,
}

impl InputState {
    /// 新しい入力状態を作成
    pub fn new() -> Self {
        Self {
            keys: [false; 256],
            mouse_position: [0.0, 0.0],
            mouse_buttons: [false; 3],
            touch_points: Vec::new(),
        }
    }

    /// キーの状態を更新
    pub fn update_key(&mut self, key_code: u8, is_pressed: bool) {
        self.keys[key_code as usize] = is_pressed;
    }

    /// マウスの位置を更新
    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = [x, y];
    }

    /// マウスボタンの状態を更新
    pub fn update_mouse_button(&mut self, button: usize, is_pressed: bool) {
        if button < 3 {
            self.mouse_buttons[button] = is_pressed;
        }
    }

    /// タッチポイントを更新
    pub fn update_touch_point(&mut self, id: u32, x: f32, y: f32, is_active: bool) {
        if let Some(point) = self.touch_points.iter_mut().find(|p| p.id == id) {
            point.position = [x, y];
            point.is_active = is_active;
        } else {
            self.touch_points.push(TouchPoint {
                id,
                position: [x, y],
                is_active,
            });
        }
    }
}

/// 入力コンポーネント
#[derive(Debug, Component)]
pub struct InputComponent {
    pub is_controllable: bool,
    pub input_actions: Vec<InputAction>,
}

/// 入力アクション
#[derive(Debug)]
pub struct InputAction {
    pub name: String,
    pub key_codes: Vec<u8>,
    pub is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_creation() {
        let input_state = InputState::new();
        assert_eq!(input_state.keys.len(), 256);
        assert_eq!(input_state.mouse_buttons.len(), 3);
        assert!(input_state.touch_points.is_empty());
    }

    #[test]
    fn test_key_update() {
        let mut input_state = InputState::new();
        input_state.update_key(32, true); // Space key
        assert!(input_state.keys[32]);
    }
} 