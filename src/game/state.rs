//! ゲーム状態管理モジュール
//! 
//! ゲームの状態を管理し、状態遷移を制御します。

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

/// ゲームの状態を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStateType {
    /// スプラッシュ画面
    Splash,
    /// メインメニュー
    MainMenu,
    /// ゲームプレイ中
    Playing,
    /// ポーズ中
    Paused,
    /// ゲームオーバー
    GameOver,
}

/// ゲーム状態を管理する構造体
pub struct GameState {
    /// 現在のゲーム状態
    current_state: GameStateType,
    /// 描画先のキャンバス
    canvas: HtmlCanvasElement,
    /// 2D描画コンテキスト
    context: web_sys::CanvasRenderingContext2d,
}

impl GameState {
    /// 新しいゲーム状態を作成します。
    /// 
    /// # 引数
    /// 
    /// * `canvas` - ゲームの描画先キャンバス
    /// 
    /// # 戻り値
    /// 
    /// 初期化されたGameStateインスタンス、または初期化エラー
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // 2D描画コンテキストの取得
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

        Ok(Self {
            current_state: GameStateType::Splash,
            canvas,
            context,
        })
    }

    /// ゲーム状態を更新します。
    /// 
    /// # 引数
    /// 
    /// * `delta_time` - 前フレームからの経過時間（秒）
    pub fn update(&mut self, delta_time: f32) -> Result<(), JsValue> {
        match self.current_state {
            GameStateType::Splash => self.update_splash(delta_time),
            GameStateType::MainMenu => self.update_main_menu(delta_time),
            GameStateType::Playing => self.update_playing(delta_time),
            GameStateType::Paused => self.update_paused(delta_time),
            GameStateType::GameOver => self.update_game_over(delta_time),
        }
    }

    /// ゲームを描画します。
    pub fn render(&self) -> Result<(), JsValue> {
        // キャンバスをクリア
        self.context.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );

        match self.current_state {
            GameStateType::Splash => self.render_splash(),
            GameStateType::MainMenu => self.render_main_menu(),
            GameStateType::Playing => self.render_playing(),
            GameStateType::Paused => self.render_paused(),
            GameStateType::GameOver => self.render_game_over(),
        }
    }

    /// キー入力を処理します。
    /// 
    /// # 引数
    /// 
    /// * `key_code` - キーコード
    /// * `pressed` - キーが押されたかどうか
    pub fn handle_key_input(&mut self, key_code: u32, pressed: bool) -> Result<(), JsValue> {
        if !pressed {
            return Ok(());
        }

        match self.current_state {
            GameStateType::Splash => self.handle_splash_key(key_code),
            GameStateType::MainMenu => self.handle_main_menu_key(key_code),
            GameStateType::Playing => self.handle_playing_key(key_code),
            GameStateType::Paused => self.handle_paused_key(key_code),
            GameStateType::GameOver => self.handle_game_over_key(key_code),
        }
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
        if !pressed {
            return Ok(());
        }

        match self.current_state {
            GameStateType::Splash => self.handle_splash_mouse(x, y, button),
            GameStateType::MainMenu => self.handle_main_menu_mouse(x, y, button),
            GameStateType::Playing => self.handle_playing_mouse(x, y, button),
            GameStateType::Paused => self.handle_paused_mouse(x, y, button),
            GameStateType::GameOver => self.handle_game_over_mouse(x, y, button),
        }
    }

    // 各状態の更新処理
    fn update_splash(&mut self, delta_time: f32) -> Result<(), JsValue> {
        // TODO: スプラッシュ画面の更新処理
        Ok(())
    }

    fn update_main_menu(&mut self, delta_time: f32) -> Result<(), JsValue> {
        // TODO: メインメニューの更新処理
        Ok(())
    }

    fn update_playing(&mut self, delta_time: f32) -> Result<(), JsValue> {
        // TODO: ゲームプレイ中の更新処理
        Ok(())
    }

    fn update_paused(&mut self, delta_time: f32) -> Result<(), JsValue> {
        // TODO: ポーズ中の更新処理
        Ok(())
    }

    fn update_game_over(&mut self, delta_time: f32) -> Result<(), JsValue> {
        // TODO: ゲームオーバーの更新処理
        Ok(())
    }

    // 各状態の描画処理
    fn render_splash(&self) -> Result<(), JsValue> {
        // TODO: スプラッシュ画面の描画処理
        Ok(())
    }

    fn render_main_menu(&self) -> Result<(), JsValue> {
        // TODO: メインメニューの描画処理
        Ok(())
    }

    fn render_playing(&self) -> Result<(), JsValue> {
        // TODO: ゲームプレイ中の描画処理
        Ok(())
    }

    fn render_paused(&self) -> Result<(), JsValue> {
        // TODO: ポーズ中の描画処理
        Ok(())
    }

    fn render_game_over(&self) -> Result<(), JsValue> {
        // TODO: ゲームオーバーの描画処理
        Ok(())
    }

    // 各状態のキー入力処理
    fn handle_splash_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        // TODO: スプラッシュ画面のキー入力処理
        Ok(())
    }

    fn handle_main_menu_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        // TODO: メインメニューのキー入力処理
        Ok(())
    }

    fn handle_playing_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        // TODO: ゲームプレイ中のキー入力処理
        Ok(())
    }

    fn handle_paused_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        // TODO: ポーズ中のキー入力処理
        Ok(())
    }

    fn handle_game_over_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        // TODO: ゲームオーバーのキー入力処理
        Ok(())
    }

    // 各状態のマウス入力処理
    fn handle_splash_mouse(&mut self, x: f32, y: f32, button: u8) -> Result<(), JsValue> {
        // TODO: スプラッシュ画面のマウス入力処理
        Ok(())
    }

    fn handle_main_menu_mouse(&mut self, x: f32, y: f32, button: u8) -> Result<(), JsValue> {
        // TODO: メインメニューのマウス入力処理
        Ok(())
    }

    fn handle_playing_mouse(&mut self, x: f32, y: f32, button: u8) -> Result<(), JsValue> {
        // TODO: ゲームプレイ中のマウス入力処理
        Ok(())
    }

    fn handle_paused_mouse(&mut self, x: f32, y: f32, button: u8) -> Result<(), JsValue> {
        // TODO: ポーズ中のマウス入力処理
        Ok(())
    }

    fn handle_game_over_mouse(&mut self, x: f32, y: f32, button: u8) -> Result<(), JsValue> {
        // TODO: ゲームオーバーのマウス入力処理
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_game_state_creation() {
        // テスト用のキャンバスを作成
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.create_element("canvas").unwrap();
        canvas.set_id("test_canvas");
        document.body().unwrap().append_child(&canvas).unwrap();

        // ゲーム状態の作成
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
        let game_state = GameState::new(canvas);
        assert!(game_state.is_ok());

        // テスト用のキャンバスを削除
        document.body().unwrap().remove_child(&canvas).unwrap();
    }
} 