//! ゲームリソース管理モジュール
//! 
//! ゲームで使用する共有リソースを管理します。

use wasm_bindgen::prelude::*;
use crate::ecs::Resource;
use std::sync::Arc;
use wasm_bindgen::JsValue;

/// ゲーム設定リソース
/// 
/// ゲームの基本的な設定を保持します。
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// ゲームのタイトル
    pub title: String,
    /// 画面の幅
    pub width: u32,
    /// 画面の高さ
    pub height: u32,
    /// フレームレート制限
    pub target_fps: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            title: "ECS WebAssembly Game".to_string(),
            width: 800,
            height: 600,
            target_fps: 60,
        }
    }
}

impl Resource for GameConfig {}

/// 時間管理リソース
/// 
/// ゲームの時間に関する情報を管理します。
#[derive(Debug, Clone)]
pub struct TimeResource {
    /// ゲーム開始からの経過時間（秒）
    pub elapsed_time: f32,
    /// 前フレームからの経過時間（秒）
    pub delta_time: f32,
    /// 時間のスケール（スローモーション等に使用）
    pub time_scale: f32,
}

impl Default for TimeResource {
    fn default() -> Self {
        Self {
            elapsed_time: 0.0,
            delta_time: 0.0,
            time_scale: 1.0,
        }
    }
}

impl Resource for TimeResource {}

/// 入力状態リソース
/// 
/// キーボードとマウスの入力状態を管理します。
#[derive(Debug, Clone, Default)]
pub struct InputState {
    /// 押されているキーの集合
    pub pressed_keys: std::collections::HashSet<u32>,
    /// マウスのX座標
    pub mouse_x: f32,
    /// マウスのY座標
    pub mouse_y: f32,
    /// 押されているマウスボタンの集合
    pub pressed_buttons: std::collections::HashSet<u8>,
}

impl Resource for InputState {}

/// アセット管理リソース
/// 
/// ゲームで使用する画像やサウンドなどのアセットを管理します。
/// WebAssemblyのスレッド制約に対応するため、パスのみを保持します。
#[derive(Debug, Clone, Default)]
pub struct AssetManager {
    /// 画像のパスマップ（ID -> パス）
    pub image_paths: std::collections::HashMap<String, String>,
    /// サウンドのパスマップ（ID -> パス）
    pub sound_paths: std::collections::HashMap<String, String>,
    /// リソースが読み込まれたかどうか
    pub loaded: bool,
}

impl AssetManager {
    /// 新しいアセットマネージャーを作成
    pub fn new() -> Self {
        Self {
            image_paths: std::collections::HashMap::new(),
            sound_paths: std::collections::HashMap::new(),
            loaded: false,
        }
    }

    /// 画像パスを登録
    pub fn register_image(&mut self, id: String, path: String) {
        self.image_paths.insert(id, path);
    }

    /// サウンドパスを登録
    pub fn register_sound(&mut self, id: String, path: String) {
        self.sound_paths.insert(id, path);
    }

    /// 画像パスを取得
    pub fn get_image_path(&self, id: &str) -> Option<&String> {
        self.image_paths.get(id)
    }

    /// サウンドパスを取得
    pub fn get_sound_path(&self, id: &str) -> Option<&String> {
        self.sound_paths.get(id)
    }
}

impl Resource for AssetManager {}

/// ゲーム統計リソース
/// 
/// ゲームの統計情報を管理します。
#[derive(Debug, Clone, Default)]
pub struct GameStats {
    /// フレーム数
    pub frame_count: u64,
    /// 平均FPS
    pub average_fps: f32,
    /// 最小FPS
    pub min_fps: f32,
    /// 最大FPS
    pub max_fps: f32,
    /// 最後のフレーム時間（秒）
    pub last_frame_time: f32,
}

impl Resource for GameStats {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert_eq!(config.title, "ECS WebAssembly Game");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.target_fps, 60);
    }

    #[test]
    fn test_time_resource_default() {
        let time = TimeResource::default();
        assert_eq!(time.elapsed_time, 0.0);
        assert_eq!(time.delta_time, 0.0);
        assert_eq!(time.time_scale, 1.0);
    }

    #[test]
    fn test_input_state_default() {
        let input = InputState::default();
        assert!(input.pressed_keys.is_empty());
        assert_eq!(input.mouse_x, 0.0);
        assert_eq!(input.mouse_y, 0.0);
        assert!(input.pressed_buttons.is_empty());
    }

    #[test]
    fn test_asset_manager_default() {
        let assets = AssetManager::default();
        assert!(assets.image_paths.is_empty());
        assert!(assets.sound_paths.is_empty());
    }

    #[test]
    fn test_game_stats_default() {
        let stats = GameStats::default();
        assert_eq!(stats.frame_count, 0);
        assert_eq!(stats.average_fps, 0.0);
        assert_eq!(stats.min_fps, 0.0);
        assert_eq!(stats.max_fps, 0.0);
        assert_eq!(stats.last_frame_time, 0.0);
    }
} 