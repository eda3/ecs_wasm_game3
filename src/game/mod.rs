//! ゲームモジュール
//!
//! ゲームのコア機能を提供するモジュールです。
//! ゲーム状態の管理、リソースの管理、ゲーム固有のシステムとエンティティを提供します。

use wasm_bindgen::prelude::*;
use crate::ecs::{World, SystemProcessor};

/// ゲームモジュールのサブモジュール
pub mod resources;  // ゲームリソース管理
pub mod state;     // ゲーム状態管理
pub mod systems;   // ゲーム固有のシステム
pub mod entities;  // ゲーム固有のエンティティ

/// ゲームインスタンス
///
/// ゲームの状態とシステムを管理する主要な構造体です。
/// ゲームループの制御、リソースの管理、エンティティの管理を行います。
#[wasm_bindgen]
pub struct Game {
    world: World,
    system_processor: SystemProcessor,
    state: state::GameState,
}

#[wasm_bindgen]
impl Game {
    /// 新しいゲームインスタンスを作成します。
    ///
    /// # 引数
    ///
    /// * `canvas_id` - ゲームの描画先キャンバスのID
    ///
    /// # 戻り値
    ///
    /// 初期化されたGameインスタンス、または初期化エラー
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<Game, JsValue> {
        // キャンバス要素の取得
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("window is not available"))?;
        let document = window.document().ok_or_else(|| JsValue::from_str("document is not available"))?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("canvas element not found"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;

        // ゲームの初期化
        let world = World::new();
        let system_processor = SystemProcessor::new();
        let state = state::GameState::new(canvas)?;

        Ok(Game {
            world,
            system_processor,
            state,
        })
    }

    /// ゲームのメインループを1フレーム進めます。
    ///
    /// # 引数
    ///
    /// * `delta_time` - 前フレームからの経過時間（秒）
    pub fn update(&mut self, delta_time: f32) -> Result<(), JsValue> {
        // ゲーム状態の更新
        self.state.update(delta_time)?;

        // システムの実行
        self.system_processor.update(&mut self.world, delta_time);

        Ok(())
    }

    /// ゲームを描画します。
    pub fn render(&self) -> Result<(), JsValue> {
        self.state.render()
    }

    /// キー入力を処理します。
    ///
    /// # 引数
    ///
    /// * `key_code` - キーコード
    /// * `pressed` - キーが押されたかどうか
    pub fn handle_key_input(&mut self, key_code: u32, pressed: bool) -> Result<(), JsValue> {
        self.state.handle_key_input(key_code, pressed)
    }

    /// マウス入力を処理します。
    ///
    /// # 引数
    ///
    /// * `x` - マウスのX座標
    /// * `y` - マウスのY座標
    /// * `button` - マウスボタン
    /// * `pressed` - ボタンが押されたかどうか
    pub fn handle_mouse_input(&mut self, x: f32, y: f32, button: u8, pressed: bool) -> Result<(), JsValue> {
        self.state.handle_mouse_input(x, y, button, pressed)
    }
}

/// ゲームシステムを初期化します。
pub fn init_game_systems(world: &mut World) {
    // 各ゲームシステムを初期化して登録
    use systems::*;
    use resources::TimeResource; // TimeリソースをTimeResourceに修正
    
    // TimeResourceを登録
    world.insert_resource(TimeResource::default()); // add_resourceをinsert_resourceに、TimeをTimeResourceに修正
    
    // TimeSystemを登録
    world.register_system(TimeSystem::new());
    
    // GameStateSystemを登録
    world.register_system(GameStateSystem::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_game_creation() {
        // テスト用のキャンバスを作成
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.create_element("canvas").unwrap();
        canvas.set_id("test_canvas");
        document.body().unwrap().append_child(&canvas).unwrap();

        // ゲームインスタンスの作成
        let game = Game::new("test_canvas");
        assert!(game.is_ok());

        // テスト用のキャンバスを削除
        document.body().unwrap().remove_child(&canvas).unwrap();
    }
} 