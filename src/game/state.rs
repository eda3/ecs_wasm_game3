//! ゲーム状態管理モジュール
//! 
//! ゲームの状態を管理し、状態遷移を制御します。

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use log;

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
        // キャンバス情報をログ出力
        log::info!("🖼️ GameState::new() - キャンバスID: {}, サイズ: {}x{}", 
                   canvas.id(), canvas.width(), canvas.height());
        
        // 2D描画コンテキストの取得
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        
        log::info!("✅ 2D描画コンテキスト取得成功");

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
        // キャンバスの状態をログ出力
        log::info!("🖼️ render()開始: キャンバスサイズ {}x{}", 
                   self.canvas.width(), self.canvas.height());

        // キャンバスをクリア
        self.context.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );

        let result = match self.current_state {
            GameStateType::Splash => {
                log::info!("🎬 スプラッシュ画面のレンダリング開始");
                self.render_splash()
            },
            GameStateType::MainMenu => {
                log::info!("📋 メインメニューのレンダリング開始");
                self.render_main_menu()
            },
            GameStateType::Playing => {
                log::info!("🎮 プレイ中画面のレンダリング開始");
                self.render_playing()
            },
            GameStateType::Paused => {
                log::info!("⏸️ ポーズ画面のレンダリング開始");
                self.render_paused()
            },
            GameStateType::GameOver => {
                log::info!("🏁 ゲームオーバー画面のレンダリング開始");
                self.render_game_over()
            },
        };
        
        log::info!("✅ render()完了: 状態={:?}", self.current_state);
        result
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
    fn update_splash(&mut self, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: スプラッシュ画面の更新処理
        Ok(())
    }

    fn update_main_menu(&mut self, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: メインメニューの更新処理
        Ok(())
    }

    fn update_playing(&mut self, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: ゲームプレイ中の更新処理
        Ok(())
    }

    fn update_paused(&mut self, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: ポーズ中の更新処理
        Ok(())
    }

    fn update_game_over(&mut self, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: ゲームオーバーの更新処理
        Ok(())
    }

    // 各状態の描画処理
    fn render_splash(&self) -> Result<(), JsValue> {
        log::info!("🔍 render_splash: 描画処理開始");
        
        // 背景を黒で塗りつぶす
        self.context.set_fill_style_str("#000000");
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        log::info!("✓ 背景を黒で塗りつぶし完了");

        // タイトルを表示
        self.context.set_font("48px Arial");
        self.context.set_text_align("center");
        self.context.set_fill_style_str("#FFFFFF");
        
        let result = self.context.fill_text(
            "ECS Wasm Game",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() / 3) as f64,
        );
        log::info!("✓ タイトルテキスト描画: {:?}", result);

        // サブタイトルを表示
        self.context.set_font("24px Arial");
        let result = self.context.fill_text(
            "Press any key to continue",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() / 2) as f64,
        );
        log::info!("✓ サブタイトルテキスト描画: {:?}", result);
        
        log::info!("🏁 render_splash: 描画処理完了");
        Ok(())
    }

    fn render_main_menu(&self) -> Result<(), JsValue> {
        log::info!("🔍 render_main_menu: 描画処理開始");
        
        // 画面クリア
        self.context.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        log::info!("✓ 画面クリア完了");

        // 背景色設定
        self.context.set_fill_style_str("#333366"); // 濃い青
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        log::info!("✓ 背景色設定完了");

        // タイトルテキスト
        self.context.set_font("48px Arial");
        self.context.set_text_align("center");
        self.context.set_fill_style_str("white");
        let result = self.context.fill_text(
            "Main Menu",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() / 4) as f64,
        );
        log::info!("✓ タイトルテキスト描画: {:?}", result);

        // メニューオプション
        self.context.set_font("24px Arial");
        let options = vec![
            "1. New Game",
            "2. Multiplayer",
            "3. Options",
            "4. Exit",
        ];

        let spacing = 40.0;
        let start_y = (self.canvas.height() / 2) as f64;

        for (i, option) in options.iter().enumerate() {
            let result = self.context.fill_text(
                option,
                (self.canvas.width() / 2) as f64,
                start_y + (i as f64 * spacing),
            );
            log::info!("✓ メニューオプション「{}」描画: {:?}", option, result);
        }

        // 操作説明
        self.context.set_font("16px Arial");
        let result = self.context.fill_text(
            "Use keyboard (1-4) or mouse to select",
            (self.canvas.width() / 2) as f64,
            self.canvas.height() as f64 - 20.0,
        );
        log::info!("✓ 操作説明テキスト描画: {:?}", result);

        log::info!("🏁 render_main_menu: 描画処理完了");
        Ok(())
    }

    fn render_playing(&self) -> Result<(), JsValue> {
        // 画面クリア
        self.context.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );

        // 背景色設定
        self.context.set_fill_style_str("#222222"); // 暗めの背景
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );

        // プレーヤーの表示（簡易的な図形）
        self.context.begin_path();
        self.context.arc(
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() / 2) as f64,
            20.0,
            0.0,
            std::f64::consts::PI * 2.0,
        )?;
        self.context.set_stroke_style_str("#44cc44");
        self.context.stroke();

        // スコア表示
        self.context.set_font("16px Arial");
        self.context.set_text_align("left");
        self.context.set_fill_style_str("red");
        self.context.fill_text(
            "Score: 0",
            10.0,
            20.0,
        )?;

        // その他のゲーム情報
        self.context.set_text_align("right");
        self.context.fill_text(
            "Level: 1",
            (self.canvas.width() as f64) - 10.0,
            20.0,
        )?;

        // 操作説明
        self.context.set_text_align("center");
        self.context.set_fill_style_str("white");
        self.context.set_font("12px Arial");
        self.context.fill_text(
            "ESC: Pause | G: Game Over",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() as f64) - 10.0,
        )?;

        Ok(())
    }

    fn render_paused(&self) -> Result<(), JsValue> {
        // ポーズ画面のオーバーレイ
        self.context.set_fill_style_str("rgba(0, 0, 0, 0.5)");
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );

        // ポーズテキスト
        self.context.set_font("48px Arial");
        self.context.set_text_align("center");
        self.context.set_fill_style_str("white");
        self.context.fill_text(
            "PAUSED",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() / 2) as f64,
        )?;

        // 操作説明
        self.context.set_font("24px Arial");
        self.context.fill_text(
            "Press ESC or click to resume",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() as f64) / 2.0 + 50.0,
        )?;

        Ok(())
    }

    fn render_game_over(&self) -> Result<(), JsValue> {
        // 画面クリア
        self.context.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );

        // 背景色設定
        self.context.set_fill_style_str("#660000"); // 暗い赤色
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );

        // ゲームオーバーテキスト
        self.context.set_font("48px Arial");
        self.context.set_text_align("center");
        self.context.set_fill_style_str("white");
        self.context.fill_text(
            "GAME OVER",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() / 3) as f64,
        )?;

        // スコア表示
        self.context.set_font("24px Arial");
        self.context.fill_text(
            "Score: 0",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() / 2) as f64,
        )?;

        // 操作説明
        self.context.set_font("18px Arial");
        self.context.fill_text(
            "Press R to retry or M for main menu",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() as f64) / 3.0 * 2.0,
        )?;

        Ok(())
    }

    // 各状態のキー入力処理
    fn handle_splash_key(&mut self, _key_code: u32) -> Result<(), JsValue> {
        // どのキーでもスプラッシュ画面からメインメニューに遷移
        web_sys::console::log_1(&"スプラッシュ画面からメインメニューへ遷移します".into());
        self.current_state = GameStateType::MainMenu;
        Ok(())
    }

    fn handle_main_menu_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        // メインメニューでのキー入力処理
        match key_code {
            // 1キー: 新規ゲーム
            49 => {
                web_sys::console::log_1(&"新規ゲームを開始します".into());
                self.current_state = GameStateType::Playing;
            },
            // 2キー: マルチプレイヤー (同じくゲーム画面へ)
            50 => {
                web_sys::console::log_1(&"マルチプレイヤーモードを開始します".into());
                self.current_state = GameStateType::Playing;
            },
            // 3キー: オプション (未実装なので何もしない)
            51 => {
                web_sys::console::log_1(&"オプション機能は未実装です".into());
            },
            // 4キー: 終了 (スプラッシュ画面に戻る)
            52 => {
                web_sys::console::log_1(&"メニューから終了します".into());
                self.current_state = GameStateType::Splash;
            },
            // その他のキー
            _ => {}
        }
        Ok(())
    }

    fn handle_playing_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        // ゲームプレイ中のキー入力処理
        match key_code {
            // ESCキー: ポーズ
            27 => {
                web_sys::console::log_1(&"ゲームをポーズします".into());
                self.current_state = GameStateType::Paused;
            },
            // Gキー: ゲームオーバー (テスト用)
            71 => {
                web_sys::console::log_1(&"ゲームオーバーへ遷移します (テスト用)".into());
                self.current_state = GameStateType::GameOver;
            },
            // その他のキー: ゲームプレイのアクションなど
            _ => {}
        }
        Ok(())
    }

    fn handle_paused_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        // ポーズ中のキー入力処理
        match key_code {
            // ESCキー: ゲームに戻る
            27 => {
                web_sys::console::log_1(&"ゲームを再開します".into());
                self.current_state = GameStateType::Playing;
            },
            // Mキー: メインメニューに戻る
            77 => {
                web_sys::console::log_1(&"メインメニューに戻ります".into());
                self.current_state = GameStateType::MainMenu;
            },
            // その他のキー
            _ => {}
        }
        Ok(())
    }

    fn handle_game_over_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        // ゲームオーバー画面のキー入力処理
        match key_code {
            // Rキー: リトライ (ゲーム画面に戻る)
            82 => {
                web_sys::console::log_1(&"ゲームをリトライします".into());
                self.current_state = GameStateType::Playing;
            },
            // Mキー: メインメニューに戻る
            77 => {
                web_sys::console::log_1(&"メインメニューに戻ります".into());
                self.current_state = GameStateType::MainMenu;
            },
            // その他のキー
            _ => {}
        }
        Ok(())
    }

    // 各状態のマウス入力処理
    fn handle_splash_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        // スプラッシュ画面でのクリックはメインメニューへ移行
        web_sys::console::log_1(&"スプラッシュ画面をクリック: メインメニューへ遷移します".into());
        self.current_state = GameStateType::MainMenu;
        Ok(())
    }

    fn handle_main_menu_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        // メインメニューでのクリック位置に応じてアクション
        let canvas_height = self.canvas.height() as f32;
        
        // メニューオプションの領域を定義
        if _y >= 180.0 && _y <= 220.0 {
            // 「新規ゲーム」オプション
            web_sys::console::log_1(&"「新規ゲーム」をクリックしました".into());
            self.current_state = GameStateType::Playing;
        } else if _y >= 230.0 && _y <= 270.0 {
            // 「マルチプレイヤー」オプション
            web_sys::console::log_1(&"「マルチプレイヤー」をクリックしました".into());
            self.current_state = GameStateType::Playing;
        } else if _y >= 280.0 && _y <= 320.0 {
            // 「オプション」オプション
            web_sys::console::log_1(&"「オプション」機能は未実装です".into());
        } else if _y >= 330.0 && _y <= 370.0 {
            // 「終了」オプション
            web_sys::console::log_1(&"「終了」をクリックしました".into());
            self.current_state = GameStateType::Splash;
        }
        
        Ok(())
    }

    fn handle_playing_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        // ゲームプレイ中のマウス入力処理
        // 右クリックでポーズメニューを表示
        if _button == 2 {
            web_sys::console::log_1(&"右クリック: ゲームをポーズします".into());
            self.current_state = GameStateType::Paused;
        }
        Ok(())
    }

    fn handle_paused_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        // ポーズ中のマウス入力処理
        // クリックでゲームに戻る
        web_sys::console::log_1(&"ポーズ画面をクリック: ゲームを再開します".into());
        self.current_state = GameStateType::Playing;
        Ok(())
    }

    fn handle_game_over_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        // ゲームオーバー画面のマウス入力処理
        let canvas_height = self.canvas.height() as f32;
        
        // 画面上半分をクリックすると再挑戦
        if _y < canvas_height / 2.0 {
            web_sys::console::log_1(&"ゲームオーバー画面上部をクリック: リトライします".into());
            self.current_state = GameStateType::Playing;
        } else {
            // 画面下半分をクリックするとメインメニューに戻る
            web_sys::console::log_1(&"ゲームオーバー画面下部をクリック: メニューに戻ります".into());
            self.current_state = GameStateType::MainMenu;
        }
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