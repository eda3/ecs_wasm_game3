//! ゲーム状態管理モジュール
//! 
//! ゲームの状態を管理し、状態遷移を制御します。
//! このファイルはゲーム全体の「状態」を制御する中心的な役割を持ちます。
//! 「状態」とは、ゲームが今どのような場面（スプラッシュ画面、メニュー、
//! 実際のプレイ中など）にあるかを表します。

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use log;

/// ゲームの状態を表す列挙型
/// 
/// ゲームには複数の状態があり、各状態ごとに異なる処理（描画・操作）が行われます。
/// 例えば、メインメニューではカード選択やデッキ構築を行い、
/// Playing（プレイ中）状態ではカードを出したり対戦したりします。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStateType {
    /// スプラッシュ画面
    /// ゲーム起動時に表示される初期画面です
    Splash,
    /// メインメニュー
    /// ゲームモード選択やデッキ編集などを行う画面です
    MainMenu,
    /// ゲームプレイ中
    /// 実際にカードゲームをプレイしている状態です
    Playing,
    /// ポーズ中
    /// ゲームを一時停止している状態です
    Paused,
    /// ゲームオーバー
    /// ゲームが終了した状態です（勝敗が決まった後など）
    GameOver,
}

/// ゲーム状態を管理する構造体
/// 
/// この構造体は現在のゲーム状態と描画に必要な情報を保持します。
/// 状態に応じた更新処理や描画処理を行うメソッドを提供します。
pub struct GameState {
    /// 現在のゲーム状態
    /// GameStateTypeのいずれかの値が入ります
    current_state: GameStateType,
    /// 描画先のキャンバス
    /// Webブラウザ上で描画を行うためのHTML要素です
    canvas: HtmlCanvasElement,
    /// 2D描画コンテキスト
    /// キャンバスに対して図形やテキストを描画するためのオブジェクトです
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
    /// 
    /// # 詳細
    /// 
    /// この関数はゲーム起動時に呼び出され、描画環境を初期化します。
    /// 最初の状態は「スプラッシュ画面」に設定されます。
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // キャンバス情報をログ出力
        log::info!("🖼️ GameState::new() - キャンバスID: {}, サイズ: {}x{}", 
                   canvas.id(), canvas.width(), canvas.height());
        
        // 2D描画コンテキストの取得
        // これによりキャンバスに図形やテキストを描画できるようになります
        let context = canvas
            .get_context("2d")?  // 2Dコンテキストを取得（失敗したら?でエラーを返す）
            .unwrap()            // Optionをアンラップ（Noneの場合はパニック）
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;  // 適切な型に変換
        
        log::info!("✅ 2D描画コンテキスト取得成功");

        // 初期状態ではスプラッシュ画面から始まります
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
    /// 
    /// # 詳細
    /// 
    /// この関数はゲームの各フレームで呼び出され、現在の状態に応じた
    /// 更新処理を行います。例えば、カードの移動やアニメーション更新などです。
    /// delta_timeを使うことで、フレームレートが変わっても一定の速度で
    /// ゲームが進行するようになります。
    pub fn update(&mut self, delta_time: f32) -> Result<(), JsValue> {
        // 現在の状態に応じた更新処理を呼び出します
        match self.current_state {
            GameStateType::Splash => self.update_splash(delta_time),
            GameStateType::MainMenu => self.update_main_menu(delta_time),
            GameStateType::Playing => self.update_playing(delta_time),
            GameStateType::Paused => self.update_paused(delta_time),
            GameStateType::GameOver => self.update_game_over(delta_time),
        }
    }

    /// ゲームを描画します。
    /// 
    /// # 詳細
    /// 
    /// この関数はゲームの各フレームで呼び出され、現在の状態に応じた
    /// 描画処理を行います。スプラッシュ画面、メニュー、ゲーム画面など、
    /// 状態によって全く異なる見た目を描画します。
    pub fn render(&self) -> Result<(), JsValue> {
        // キャンバスの状態をログ出力
        log::info!("🖼️ render()開始: キャンバスサイズ {}x{}", 
                   self.canvas.width(), self.canvas.height());

        // キャンバスをクリア（前のフレームの描画内容を消去）
        self.context.clear_rect(
            0.0,  // 左上X座標
            0.0,  // 左上Y座標
            self.canvas.width() as f64,   // 幅
            self.canvas.height() as f64,  // 高さ
        );

        // 現在の状態に応じた描画処理を呼び出す
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
    /// * `key_code` - キーコード（どのキーが押されたか）
    /// * `pressed` - キーが押されたかどうか（true=押された、false=離された）
    /// 
    /// # 詳細
    /// 
    /// プレイヤーのキーボード入力を処理します。例えば：
    /// - メニューでの選択移動（矢印キー）
    /// - カードの選択や確定（エンターキー）
    /// - ポーズ/再開（ESCキー）
    /// 
    /// 現在の状態によって、同じキー入力でも異なる処理が行われます。
    pub fn handle_key_input(&mut self, key_code: u32, pressed: bool) -> Result<(), JsValue> {
        // キーが離された場合は何もしない（押されたときだけ処理）
        if !pressed {
            return Ok(());
        }

        // 現在の状態に応じたキー処理メソッドを呼び出す
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
    /// * `x` - マウスのX座標（キャンバス内の位置）
    /// * `y` - マウスのY座標（キャンバス内の位置）
    /// * `button` - マウスボタン（0=左、1=中、2=右）
    /// * `pressed` - ボタンが押されたかどうか
    /// 
    /// # 詳細
    /// 
    /// プレイヤーのマウス入力を処理します。カードゲームでは特に重要で：
    /// - カードの選択（クリック）
    /// - カードのドラッグ＆ドロップ
    /// - メニュー項目の選択
    /// などの操作に使われます。
    pub fn handle_mouse_input(&mut self, x: f32, y: f32, button: u8, pressed: bool) -> Result<(), JsValue> {
        // ボタンが離された場合は何もしない（押されたときだけ処理）
        if !pressed {
            return Ok(());
        }

        // 現在の状態に応じたマウス処理メソッドを呼び出す
        match self.current_state {
            GameStateType::Splash => self.handle_splash_mouse(x, y, button),
            GameStateType::MainMenu => self.handle_main_menu_mouse(x, y, button),
            GameStateType::Playing => self.handle_playing_mouse(x, y, button),
            GameStateType::Paused => self.handle_paused_mouse(x, y, button),
            GameStateType::GameOver => self.handle_game_over_mouse(x, y, button),
        }
    }

    // 各状態の更新処理
    // -----------------
    
    /// スプラッシュ画面の更新処理
    /// 
    /// スプラッシュ画面では、通常は時間経過やキー入力に応じて
    /// メインメニューに移行する処理を行います。
    /// アニメーションを表示する場合はここで更新します。
    fn update_splash(&mut self, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: スプラッシュ画面の更新処理
        // 例: 一定時間経過後にメインメニューに移行
        // if self.splash_timer > 3.0 {
        //     self.current_state = GameStateType::MainMenu;
        // }
        Ok(())
    }

    /// メインメニューの更新処理
    /// 
    /// メニュー項目のアニメーションや、選択カーソルの点滅などを
    /// 更新する処理をここに書きます。
    fn update_main_menu(&mut self, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: メインメニューの更新処理
        // 例: メニュー項目のアニメーション更新
        Ok(())
    }

    /// ゲームプレイ中の更新処理
    /// 
    /// カードゲームの実際のプレイ中に行われる更新処理です。
    /// - カードの移動アニメーション
    /// - 効果の発動
    /// - 相手プレイヤーの行動
    /// - ターン経過
    /// などを処理します。
    fn update_playing(&mut self, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: ゲームプレイ中の更新処理
        // 例: カードの移動アニメーション更新
        // for card in self.cards.iter_mut() {
        //     card.update_position(delta_time);
        // }
        Ok(())
    }

    /// ポーズ中の更新処理
    /// 
    /// ゲームが一時停止している間の処理です。
    /// 通常はあまり処理を行いませんが、ポーズメニューの
    /// アニメーションなどはここで更新します。
    fn update_paused(&mut self, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: ポーズ中の更新処理
        // ポーズ中は基本的にゲームの進行を止めるので
        // 最小限の処理だけを行います
        Ok(())
    }

    /// ゲームオーバー時の更新処理
    /// 
    /// ゲーム終了時（勝敗決定後）の処理です。
    /// スコア表示や結果画面のアニメーションなどを
    /// 更新する処理をここに書きます。
    fn update_game_over(&mut self, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: ゲームオーバーの更新処理
        // 例: 結果発表のアニメーション
        // self.result_animation.update(delta_time);
        Ok(())
    }

    // 各状態の描画処理
    // ---------------
    
    /// スプラッシュ画面の描画処理
    /// 
    /// ゲーム起動時に表示されるスプラッシュ画面（ロゴなど）を描画します。
    fn render_splash(&self) -> Result<(), JsValue> {
        log::info!("🔍 render_splash: 描画処理開始");
        
        // 背景を黒で塗りつぶす
        self.context.set_fill_style_str("#000000");  // 黒色を設定
        self.context.fill_rect(
            0.0,  // 左上X座標
            0.0,  // 左上Y座標
            self.canvas.width() as f64,   // 幅
            self.canvas.height() as f64,  // 高さ
        );
        log::info!("✓ 背景を黒で塗りつぶし完了");
        
        // ここにロゴや開始メッセージの描画処理を追加

        // タイトルを表示
        self.context.set_font("40px Arial");
        self.context.set_fill_style_str("#FFFFFF");  // 白色テキスト
        self.context.set_text_align("center");
        self.context.set_text_baseline("middle");
        
        // 画面中央にタイトルを描画
        let _ = self.context.fill_text(
            "カードゲーム",  // ゲームタイトル
            (self.canvas.width() / 2) as f64,  // X座標（中央）
            (self.canvas.height() / 2) as f64,  // Y座標（中央）
        );
        
        // 「Press Any Key」のメッセージを表示
        self.context.set_font("20px Arial");
        let _ = self.context.fill_text(
            "Press Any Key to Start",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() / 2 + 50) as f64,  // タイトルの下に表示
        );
        
        log::info!("✓ スプラッシュ画面の描画完了");
        Ok(())
    }

    /// メインメニュー画面の描画処理
    /// 
    /// ゲームのメインメニュー（モード選択、デッキ編集など）を描画します。
    /// メニュー項目やボタン、背景などを表示します。
    fn render_main_menu(&self) -> Result<(), JsValue> {
        // 背景色を設定（濃い青）
        self.context.set_fill_style_str("#001133");
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        
        // メニュータイトルを描画
        self.context.set_font("32px Arial");
        self.context.set_fill_style_str("#FFFFFF");
        self.context.set_text_align("center");
        let _ = self.context.fill_text(
            "メインメニュー",
            (self.canvas.width() / 2) as f64,
            50.0,
        );
        
        // メニュー項目を描画
        self.context.set_font("24px Arial");
        
        // この部分は実際のメニュー項目を表示する
        // メニュー項目はボタンや選択可能なテキストとして表示される
        let menu_items = [
            "プレイ開始",
            "デッキ編集",
            "オプション",
            "ルール説明",
            "終了"
        ];
        
        for (i, item) in menu_items.iter().enumerate() {
            let y = 150.0 + (i as f64) * 50.0;
            
            // メニュー項目の背景（選択中の項目は強調表示）
            // self.context.set_fill_style_str(if i == self.selected_menu_index { "#3355AA" } else { "#223366" });
            self.context.set_fill_style_str("#223366");  // 通常の項目の背景色
            
            // メニュー項目の背景を描画
            self.context.fill_rect(
                (self.canvas.width() as f64 / 2.0) - 150.0,
                y - 20.0,
                300.0,
                40.0,
            );
            
            // メニュー項目のテキストを描画
            self.context.set_fill_style_str("#FFFFFF");
            let _ = self.context.fill_text(
                item,
                (self.canvas.width() / 2) as f64,
                y,
            );
        }
        
        // 操作方法のヘルプテキスト
        self.context.set_font("16px Arial");
        self.context.set_fill_style_str("#AAAAAA");
        let _ = self.context.fill_text(
            "↑↓: 選択  Enter: 決定  Esc: 戻る",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() - 30) as f64,
        );
        
        Ok(())
    }

    /// ゲームプレイ中の描画処理
    /// 
    /// カードゲームのメイン画面を描画します。
    /// - プレイヤーの手札
    /// - 場のカード
    /// - 相手の情報
    /// - ゲーム状態（ライフ、ターン数など）
    /// などを描画します。
    fn render_playing(&self) -> Result<(), JsValue> {
        // 背景色を設定（緑のテーブル）
        self.context.set_fill_style_str("#006622");
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        
        // ゲーム情報表示（ターン、プレイヤー名など）
        self.context.set_font("18px Arial");
        self.context.set_fill_style_str("#FFFFFF");
        self.context.set_text_align("left");
        let _ = self.context.fill_text(
            "ターン: 1 / プレイヤー1の番",
            20.0,
            30.0,
        );
        
        // スコア／ライフ表示
        self.context.set_text_align("right");
        let _ = self.context.fill_text(
            "ライフ: 20 / 20",
            (self.canvas.width() - 20) as f64,
            30.0,
        );
        
        // プレイエリアの境界線
        self.context.set_stroke_style_str("#FFFFFF");
        self.context.set_line_width(2.0);
        self.context.begin_path();
        self.context.move_to(0.0, (self.canvas.height() / 2) as f64);
        self.context.line_to(self.canvas.width() as f64, (self.canvas.height() / 2) as f64);
        self.context.stroke();
        
        // 相手のカード（裏向き）を描画
        // ここでは簡略化のために単純な四角形を描画
        self.context.set_fill_style_str("#3333AA");  // 相手カードの背景色
        
        // 相手の手札（カードの裏面）
        for i in 0..5 {
            let x = 100.0 + (i as f64) * 80.0;
            let y = 100.0;
            
            // カードの裏面を描画
            self.context.fill_rect(x, y, 70.0, 100.0);
            
            // カードの枠線
            self.context.stroke_rect(x, y, 70.0, 100.0);
        }
        
        // プレイヤーの手札（表向き）
        // 実際のゲームではカードデータに基づいて描画
        self.context.set_fill_style_str("#FFDD33");  // プレイヤーカードの背景色
        
        for i in 0..5 {
            let x = 100.0 + (i as f64) * 80.0;
            let y = (self.canvas.height() - 150) as f64;
            
            // カードを描画
            self.context.fill_rect(x, y, 70.0, 100.0);
            
            // カードの枠線
            self.context.stroke_rect(x, y, 70.0, 100.0);
            
            // カードの値やテキストを描画
            self.context.set_fill_style_str("#000000");
            self.context.set_font("14px Arial");
            self.context.set_text_align("center");
            
            // カード番号
            let _ = self.context.fill_text(
                &format!("カード{}", i+1),
                x + 35.0,
                y + 20.0,
            );
            
            // カードの効果テキスト
            self.context.set_font("10px Arial");
            let _ = self.context.fill_text(
                "効果テキスト",
                x + 35.0,
                y + 60.0,
            );
        }
        
        // 操作ガイド
        self.context.set_font("16px Arial");
        self.context.set_fill_style_str("#FFFFFF");
        self.context.set_text_align("center");
        let _ = self.context.fill_text(
            "クリック: カード選択  Enter: カードプレイ  Esc: メニュー",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() - 20) as f64,
        );
        
        Ok(())
    }

    /// ポーズ画面の描画処理
    /// 
    /// ゲームが一時停止している間の画面を描画します。
    /// メニュー選択や設定変更などのUI要素を表示します。
    fn render_paused(&self) -> Result<(), JsValue> {
        // まず現在のゲーム画面を半透明で表示（背景として）
        self.render_playing()?;
        
        // 半透明の黒いオーバーレイを描画
        self.context.set_fill_style_str("rgba(0, 0, 0, 0.7)");
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        
        // ポーズメニューのタイトル
        self.context.set_font("36px Arial");
        self.context.set_fill_style_str("#FFFFFF");
        self.context.set_text_align("center");
        let _ = self.context.fill_text(
            "ゲーム一時停止",
            (self.canvas.width() / 2) as f64,
            120.0,
        );
        
        // ポーズメニューの選択項目
        self.context.set_font("24px Arial");
        let menu_items = ["ゲームに戻る", "オプション", "ゲームを終了"];
        
        for (i, item) in menu_items.iter().enumerate() {
            let y = 200.0 + (i as f64) * 50.0;
            
            // メニュー項目の背景
            self.context.set_fill_style_str("#223366");
            self.context.fill_rect(
                (self.canvas.width() as f64 / 2.0) - 150.0,
                y - 20.0,
                300.0,
                40.0,
            );
            
            // メニュー項目のテキスト
            self.context.set_fill_style_str("#FFFFFF");
            let _ = self.context.fill_text(
                item,
                (self.canvas.width() / 2) as f64,
                y,
            );
        }
        
        // 操作説明
        self.context.set_font("16px Arial");
        self.context.set_fill_style_str("#CCCCCC");
        let _ = self.context.fill_text(
            "↑↓: 選択  Enter: 決定  Esc: ゲームに戻る",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() - 50) as f64,
        );
        
        Ok(())
    }

    /// ゲームオーバー画面の描画処理
    /// 
    /// ゲームが終了した後の結果画面を描画します。
    /// 勝敗結果、スコア、統計情報などを表示します。
    fn render_game_over(&self) -> Result<(), JsValue> {
        // 背景を濃い色で描画
        self.context.set_fill_style_str("#111133");
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        
        // ゲームオーバーのタイトル
        self.context.set_font("48px Arial");
        self.context.set_fill_style_str("#FFCC00");  // 金色
        self.context.set_text_align("center");
        let _ = self.context.fill_text(
            "ゲーム終了",
            (self.canvas.width() / 2) as f64,
            100.0,
        );
        
        // 勝敗結果
        self.context.set_font("36px Arial");
        self.context.set_fill_style_str("#FFFFFF");
        let _ = self.context.fill_text(
            "あなたの勝利！",  // または "あなたの敗北..."
            (self.canvas.width() / 2) as f64,
            180.0,
        );
        
        // ゲーム統計情報
        self.context.set_font("24px Arial");
        let stats = [
            "プレイ時間: 5分23秒",
            "使用カード数: 12枚",
            "最大コンボ: 3連鎖",
            "スコア: 2850点",
        ];
        
        for (i, stat) in stats.iter().enumerate() {
            let y = 250.0 + (i as f64) * 40.0;
            let _ = self.context.fill_text(
                stat,
                (self.canvas.width() / 2) as f64,
                y,
            );
        }
        
        // 再戦/終了ボタン
        self.context.set_font("24px Arial");
        
        // 再戦ボタン
        self.context.set_fill_style_str("#2255AA");
        self.context.fill_rect(
            (self.canvas.width() as f64 / 2.0) - 200.0,
            350.0,
            180.0,
            50.0,
        );
        
        self.context.set_fill_style_str("#FFFFFF");
        let _ = self.context.fill_text(
            "もう一度",
            (self.canvas.width() as f64 / 2.0) - 110.0,
            380.0,
        );
        
        // 終了ボタン
        self.context.set_fill_style_str("#AA2255");
        self.context.fill_rect(
            (self.canvas.width() as f64 / 2.0) + 20.0,
            350.0,
            180.0,
            50.0,
        );
        
        self.context.set_fill_style_str("#FFFFFF");
        let _ = self.context.fill_text(
            "メニューへ",
            (self.canvas.width() as f64 / 2.0) + 110.0,
            380.0,
        );
        
        // 操作説明
        self.context.set_font("16px Arial");
        self.context.set_fill_style_str("#CCCCCC");
        let _ = self.context.fill_text(
            "Enter: 再戦  Esc: メニューへ戻る",
            (self.canvas.width() / 2) as f64,
            (self.canvas.height() - 50) as f64,
        );
        
        Ok(())
    }

    // キー入力処理
    // -----------
    
    /// スプラッシュ画面でのキー入力処理
    /// 
    /// どのキーが押されてもメインメニューに遷移します。
    fn handle_splash_key(&mut self, _key_code: u32) -> Result<(), JsValue> {
        log::info!("🔑 スプラッシュ画面でキー入力: {}", _key_code);
        
        // どのキーでもメインメニューに移動
        self.current_state = GameStateType::MainMenu;
        log::info!("🔄 状態遷移: スプラッシュ → メインメニュー");
        
        Ok(())
    }

    /// メインメニューでのキー入力処理
    /// 
    /// メニュー項目の選択や決定を処理します。
    /// - 上下キー: 選択項目の移動
    /// - Enter: 選択項目の決定
    /// - Esc: 前の画面に戻る
    fn handle_main_menu_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        log::info!("🔑 メインメニューでキー入力: {}", key_code);
        
        match key_code {
            // Enterキー
            13 => {
                // 選択されているメニュー項目に応じた処理
                // 例: 「プレイ開始」ならゲーム開始
                self.current_state = GameStateType::Playing;
                log::info!("🔄 状態遷移: メインメニュー → プレイ中");
            },
            // Escキー
            27 => {
                // スプラッシュ画面に戻る
                self.current_state = GameStateType::Splash;
                log::info!("🔄 状態遷移: メインメニュー → スプラッシュ");
            },
            // 上キー
            38 => {
                // 選択項目を上に移動
                log::info!("⬆️ メニュー選択を上に移動");
                // self.selected_menu_index = (self.selected_menu_index + self.menu_items.len() - 1) % self.menu_items.len();
            },
            // 下キー
            40 => {
                // 選択項目を下に移動
                log::info!("⬇️ メニュー選択を下に移動");
                // self.selected_menu_index = (self.selected_menu_index + 1) % self.menu_items.len();
            },
            _ => {
                log::info!("⚠️ 未処理のキー入力: {}", key_code);
            }
        }
        
        Ok(())
    }

    /// ゲームプレイ中のキー入力処理
    /// 
    /// カードゲームでのプレイヤー操作を処理します。
    /// - 方向キー: カードの選択
    /// - Enter: カードのプレイ
    /// - Esc: ポーズメニューを開く
    fn handle_playing_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        log::info!("🔑 ゲームプレイ中のキー入力: {}", key_code);
        
        match key_code {
            // Enterキー（カードプレイやアクション確定）
            13 => {
                log::info!("🃏 選択カードをプレイ");
                // カードプレイのロジック
            },
            // Escキー（ポーズメニューを開く）
            27 => {
                self.current_state = GameStateType::Paused;
                log::info!("🔄 状態遷移: プレイ中 → ポーズ");
            },
            // その他のキー入力（カード選択など）
            _ => {
                log::info!("⚠️ 未処理のキー入力: {}", key_code);
            }
        }
        
        Ok(())
    }

    /// ポーズ中のキー入力処理
    /// 
    /// ポーズメニューでの操作を処理します。
    /// - 上下キー: メニュー項目の選択
    /// - Enter: 選択項目の決定
    /// - Esc: ゲームに戻る
    fn handle_paused_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        log::info!("🔑 ポーズ中のキー入力: {}", key_code);
        
        match key_code {
            // Enterキー（メニュー項目の決定）
            13 => {
                // 選択されているメニュー項目に応じた処理
                // 例: 「ゲームに戻る」ならプレイ状態に戻る
                log::info!("✅ ポーズメニューの項目を選択");
                self.current_state = GameStateType::Playing;
                log::info!("🔄 状態遷移: ポーズ → プレイ中");
            },
            // Escキー（ゲームに戻る）
            27 => {
                self.current_state = GameStateType::Playing;
                log::info!("🔄 状態遷移: ポーズ → プレイ中");
            },
            // その他のキー操作
            _ => {
                log::info!("⚠️ 未処理のキー入力: {}", key_code);
            }
        }
        
        Ok(())
    }

    /// ゲームオーバー時のキー入力処理
    /// 
    /// ゲーム終了画面での操作を処理します。
    /// - Enter: 再戦
    /// - Esc: メインメニューに戻る
    fn handle_game_over_key(&mut self, key_code: u32) -> Result<(), JsValue> {
        log::info!("🔑 ゲームオーバー時のキー入力: {}", key_code);
        
        match key_code {
            // Enterキー（再戦）
            13 => {
                // 新しいゲームを開始
                self.current_state = GameStateType::Playing;
                log::info!("🔄 状態遷移: ゲームオーバー → プレイ中（再戦）");
            },
            // Escキー（メインメニューに戻る）
            27 => {
                self.current_state = GameStateType::MainMenu;
                log::info!("🔄 状態遷移: ゲームオーバー → メインメニュー");
            },
            // その他のキー操作
            _ => {
                log::info!("⚠️ 未処理のキー入力: {}", key_code);
            }
        }
        
        Ok(())
    }

    // マウス入力処理
    // -------------
    
    /// スプラッシュ画面でのマウス入力処理
    /// 
    /// クリックするとメインメニューに遷移します。
    fn handle_splash_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        log::info!("🖱️ スプラッシュ画面でマウスクリック: 座標({}, {}), ボタン{}", _x, _y, _button);
        
        // クリックでメインメニューに移動
        self.current_state = GameStateType::MainMenu;
        log::info!("🔄 状態遷移: スプラッシュ → メインメニュー");
        
        Ok(())
    }

    /// メインメニュー画面でのマウス入力処理
    /// 
    /// メニュー項目のクリックを処理します。
    /// 各メニュー項目の領域をクリックすると、対応する機能が実行されます。
    fn handle_main_menu_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        log::info!("🖱️ メインメニューでマウスクリック: 座標({}, {}), ボタン{}", _x, _y, _button);
        
        // メニュー項目のY座標範囲をチェック
        // クリックされた座標が各メニュー項目の範囲内かどうかを確認
        
        // 例: プレイ開始ボタン領域の判定
        let play_button_y = 150.0;
        if (_y >= play_button_y - 20.0 && _y <= play_button_y + 20.0) {
            // プレイ開始ボタンがクリックされた
            log::info!("✅ プレイ開始ボタンがクリックされました");
            self.current_state = GameStateType::Playing;
            log::info!("🔄 状態遷移: メインメニュー → プレイ中");
            return Ok(());
        }
        
        // 例: その他のメニューボタンの判定
        // ...
        
        // クリックが有効なUIコンポーネント上でなかった場合
        log::info!("⚠️ 有効なメニュー項目がクリックされませんでした");
        Ok(())
    }

    /// ゲームプレイ中のマウス入力処理
    /// 
    /// カードゲームのプレイ中、マウスクリックでカードを選択したり
    /// アクションを実行したりする処理を行います。
    fn handle_playing_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        log::info!("🖱️ ゲームプレイ中のマウスクリック: 座標({}, {}), ボタン{}", _x, _y, _button);
        
        // プレイヤーの手札領域をクリックしたかチェック
        let hand_cards_y = self.canvas.height() as f32 - 150.0;
        if (_y >= hand_cards_y && _y <= hand_cards_y + 100.0) {
            // X座標から何番目のカードがクリックされたかを計算
            let card_width = 80.0;
            let card_start_x = 100.0;
            
            // クリックされたカードのインデックスを計算
            let card_index = ((_x - card_start_x) / card_width) as usize;
            
            // 有効なカードインデックスかチェック（0〜4の範囲）
            if card_index < 5 {
                log::info!("🃏 カード {} がクリックされました", card_index + 1);
                // カード選択やプレイのロジックをここに実装
                // 例: self.select_card(card_index);
            }
        }
        
        // 場のカードや相手のカードなど、他の領域のクリック処理もここに実装
        
        Ok(())
    }

    /// ポーズ画面でのマウス入力処理
    /// 
    /// ポーズメニューの項目選択を処理します。
    fn handle_paused_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        log::info!("🖱️ ポーズ画面でのマウスクリック: 座標({}, {}), ボタン{}", _x, _y, _button);
        
        // ポーズメニュー項目のY座標範囲をチェック
        let resume_button_y = 200.0;
        if (_y >= resume_button_y - 20.0 && _y <= resume_button_y + 20.0) {
            // 「ゲームに戻る」ボタンがクリックされた
            log::info!("✅ ゲームに戻るボタンがクリックされました");
            self.current_state = GameStateType::Playing;
            log::info!("🔄 状態遷移: ポーズ → プレイ中");
            return Ok(());
        }
        
        // 他のポーズメニュー項目の処理
        // ...
        
        Ok(())
    }

    /// ゲームオーバー画面でのマウス入力処理
    /// 
    /// 結果画面での「再戦」「メニューへ」などのボタンクリックを処理します。
    fn handle_game_over_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        log::info!("🖱️ ゲームオーバー画面でのマウスクリック: 座標({}, {}), ボタン{}", _x, _y, _button);
        
        // 「もう一度」ボタンの判定
        let retry_button_x = (self.canvas.width() as f32 / 2.0) - 110.0;
        let retry_button_y = 380.0;
        
        if (_x >= retry_button_x - 90.0 && _x <= retry_button_x + 90.0 &&
            _y >= retry_button_y - 25.0 && _y <= retry_button_y + 25.0) {
            // 「もう一度」ボタンがクリックされた
            log::info!("✅ もう一度ボタンがクリックされました");
            self.current_state = GameStateType::Playing;
            log::info!("🔄 状態遷移: ゲームオーバー → プレイ中（再戦）");
            return Ok(());
        }
        
        // 「メニューへ」ボタンの判定
        let menu_button_x = (self.canvas.width() as f32 / 2.0) + 110.0;
        let menu_button_y = 380.0;
        
        if (_x >= menu_button_x - 90.0 && _x <= menu_button_x + 90.0 &&
            _y >= menu_button_y - 25.0 && _y <= menu_button_y + 25.0) {
            // 「メニューへ」ボタンがクリックされた
            log::info!("✅ メニューへボタンがクリックされました");
            self.current_state = GameStateType::MainMenu;
            log::info!("🔄 状態遷移: ゲームオーバー → メインメニュー");
            return Ok(());
        }
        
        Ok(())
    }
}

// ユニットテスト
// -------------
#[cfg(test)]
mod tests {
    use super::*;
    
    /// GameStateの作成テスト
    /// 
    /// この関数は実際のテストではなく、コンパイルが通るかをチェックする役割です。
    /// WebブラウザのAPIを使うため、実際のテスト実行はブラウザ環境でのみ可能です。
    #[test]
    fn test_game_state_creation() {
        // このテストは実際には実行されません（HtmlCanvasElementが必要なため）
        // コンパイル時チェックのみを目的としています
    }
} 