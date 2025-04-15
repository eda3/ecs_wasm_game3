//! 時間ユーティリティモジュール
//! 
//! このモジュールには、ゲーム内で使用される時間関連のユーティリティ関数や構造体が含まれています。

use wasm_bindgen::prelude::*;
use web_sys::window;

/// ゲーム内時間管理用の構造体
#[derive(Debug, Clone)]
pub struct GameTime {
    /// ゲーム開始からの経過時間
    elapsed: f32,
    /// 前回のフレームからの経過時間
    delta: f32,
    /// ゲーム内時間スケール（1.0が通常速度）
    time_scale: f32,
    /// 開始タイムスタンプ（ブラウザのDate.now()値）
    start_time: f64,
    /// 最後の更新タイムスタンプ
    last_update: f64,
    /// 累積フレーム数
    frame_count: u64,
}

impl GameTime {
    /// 新しいGameTimeインスタンスを作成
    pub fn new() -> Self {
        let current_time = js_sys::Date::now();
        
        Self {
            elapsed: 0.0,
            delta: 0.0,
            time_scale: 1.0,
            start_time: current_time,
            last_update: current_time,
            frame_count: 0,
        }
    }
    
    /// 時間を更新
    pub fn update(&mut self) {
        let current_time = js_sys::Date::now();
        
        // デルタタイム（秒単位）を計算
        let raw_delta = (current_time - self.last_update) as f32 / 1000.0;
        
        // タイムスケールを適用
        self.delta = raw_delta * self.time_scale;
        
        // 経過時間を更新
        self.elapsed += self.delta;
        
        // 最終更新時間を保存
        self.last_update = current_time;
        
        // フレームカウントを増加
        self.frame_count += 1;
    }
    
    /// ゲーム開始からの経過時間（秒）を取得
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }
    
    /// 前回のフレームからの経過時間（秒）を取得
    pub fn delta(&self) -> f32 {
        self.delta
    }
    
    /// 時間スケールを設定
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }
    
    /// 時間スケールを取得
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }
    
    /// 現在のフレームレート（FPS）を計算
    pub fn fps(&self) -> f32 {
        if self.delta == 0.0 {
            0.0
        } else {
            1.0 / self.delta
        }
    }
    
    /// フレーム数を取得
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

/// タイマー構造体
#[derive(Debug, Clone)]
pub struct Timer {
    /// 持続時間
    duration: f32,
    /// 残り時間
    remaining: f32,
    /// タイマーが完了したかどうか
    completed: bool,
    /// 繰り返しかどうか
    repeat: bool,
}

impl Timer {
    /// 新しいタイマーを作成
    /// 
    /// # 引数
    /// 
    /// * `duration` - タイマーの持続時間（秒）
    /// * `repeat` - タイマーが完了後に自動的にリセットするかどうか
    pub fn new(duration: f32, repeat: bool) -> Self {
        Self {
            duration,
            remaining: duration,
            completed: false,
            repeat,
        }
    }
    
    /// タイマーを更新
    /// 
    /// # 引数
    /// 
    /// * `delta_time` - 経過時間（秒）
    /// 
    /// # 戻り値
    /// 
    /// * タイマーが今回の更新で完了したかどうか
    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.completed && !self.repeat {
            return false;
        }
        
        self.remaining -= delta_time;
        
        // 完了したかどうかをチェック
        if self.remaining <= 0.0 {
            self.completed = true;
            
            // 繰り返しの場合はリセット
            if self.repeat {
                self.remaining += self.duration;
                while self.remaining <= 0.0 {
                    self.remaining += self.duration;
                }
            } else {
                self.remaining = 0.0;
            }
            
            true
        } else {
            false
        }
    }
    
    /// タイマーをリセット
    pub fn reset(&mut self) {
        self.remaining = self.duration;
        self.completed = false;
    }
    
    /// タイマーが完了したかどうかを確認
    pub fn is_completed(&self) -> bool {
        self.completed
    }
    
    /// 残り時間を取得（秒）
    pub fn remaining(&self) -> f32 {
        self.remaining
    }
    
    /// 経過時間を取得（秒）
    pub fn elapsed(&self) -> f32 {
        self.duration - self.remaining
    }
    
    /// 進行度を取得（0.0〜1.0）
    pub fn progress(&self) -> f32 {
        if self.duration == 0.0 {
            1.0
        } else {
            (self.duration - self.remaining) / self.duration
        }
    }
}

/// 現在のブラウザ時間を取得（ミリ秒）
pub fn current_time_millis() -> f64 {
    js_sys::Date::now()
}

/// スリープ関数（非同期）
/// 
/// # 引数
/// 
/// * `ms` - スリープする時間（ミリ秒）
pub async fn sleep(ms: i32) -> Result<(), JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let window = window().unwrap();
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                ms,
            )
            .unwrap();
    });
    
    wasm_bindgen_futures::JsFuture::from(promise).await?;
    
    Ok(())
} 