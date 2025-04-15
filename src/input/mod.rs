//! 入力処理システムの実装
//! 
//! このモジュールは、キーボード、マウス、タッチなどの入力処理を担当します。
//! 入力状態の管理とイベントの処理を行います。また、キーコンフィグ機能や
//! 複雑な入力処理（キーリピート、ジェスチャー検出など）も提供します。

use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;
use web_sys::KeyboardEvent as WebKeyboardEvent;
use web_sys::console;
use js_sys::Date;
use wasm_bindgen::JsValue;

use crate::ecs::{Entity, System, World, SystemPhase, SystemPriority, ResourceManager, Resource};
use crate::ecs::component::Component;
use crate::ecs::query::Query;

pub mod key_codes;
pub mod gestures;

pub use key_codes::*;

/// キーボードイベント
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    /// イベントタイプ（keydown, keyup）
    pub event_type: String,
    /// キーコード
    pub key: String,
}

/// マウスイベント
#[derive(Debug, Clone)]
pub struct MouseEvent {
    /// イベントタイプ（mousedown, mouseup, mousemove）
    pub event_type: String,
    /// マウス位置
    pub position: (f32, f32),
    /// ボタン (0: 左, 1: 中, 2: 右)
    pub button: Option<i32>,
}

/// キーコード用の型エイリアス
pub type KeyCode = u32;

/// マウスボタン用の型エイリアス
pub type MouseButton = u8;

/// タッチID用の型エイリアス
pub type TouchId = i32;

/// 入力アクション名用の型エイリアス
pub type ActionName = String;

/// 入力状態を管理する構造体
#[derive(Debug, Clone)]
pub struct InputState {
    /// キーの現在の状態（押されているかどうか）
    pub keys_pressed: HashSet<KeyCode>,
    /// 前回のフレームでのキーの状態
    pub keys_previous: HashSet<KeyCode>,
    /// キーが押されてからの経過時間（ミリ秒）
    pub key_press_duration: HashMap<KeyCode, f64>,
    /// 最後にキーが押された時間（エポックからのミリ秒）
    pub key_last_press_time: HashMap<KeyCode, f64>,
    /// 最後にキーがリピートされた時間（エポックからのミリ秒）
    pub key_last_repeat_time: HashMap<KeyCode, f64>,
    
    /// マウスの現在位置
    pub mouse_position: (f32, f32),
    /// マウスの前回の位置
    pub mouse_previous: (f32, f32),
    /// マウスの移動速度（ピクセル/秒）
    pub mouse_velocity: (f32, f32),
    /// マウスボタンの状態
    pub mouse_buttons: HashMap<MouseButton, bool>,
    /// 前回のフレームでのマウスボタンの状態
    pub mouse_buttons_previous: HashMap<MouseButton, bool>,
    /// マウスホイールのデルタ値
    pub mouse_wheel_delta: f32,
    
    /// アクティブなタッチポイント
    pub touch_points: HashMap<TouchId, TouchPoint>,
    /// 前回のフレームでのタッチポイント
    pub touch_points_previous: HashMap<TouchId, TouchPoint>,
    /// ジェスチャー検出器
    pub gesture_detector: GestureDetector,
    
    /// 入力マッピング
    pub action_mapping: ActionMapping,
}

/// タッチポイントの情報
#[derive(Debug, Clone)]
pub struct TouchPoint {
    /// タッチの一意のID
    pub id: TouchId,
    /// タッチの現在位置
    pub position: (f32, f32),
    /// タッチの前回の位置
    pub previous_position: (f32, f32),
    /// タッチの移動速度
    pub velocity: (f32, f32),
    /// タッチの圧力（0.0～1.0）
    pub force: f32,
    /// タッチがアクティブかどうか
    pub is_active: bool,
    /// タッチが開始された時間（エポックからのミリ秒）
    pub start_time: f64,
}

/// ジェスチャータイプの列挙型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GestureType {
    /// タップ（短い押し）
    Tap,
    /// ダブルタップ
    DoubleTap,
    /// 長押し
    LongPress,
    /// スワイプ（上下左右）
    Swipe(SwipeDirection),
    /// ピンチ（拡大縮小）
    Pinch,
    /// 回転
    Rotate,
}

/// スワイプの方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeDirection {
    /// 上方向
    Up,
    /// 下方向
    Down,
    /// 左方向
    Left,
    /// 右方向
    Right,
}

/// ジェスチャー検出器
#[derive(Debug, Clone)]
pub struct GestureDetector {
    /// 検出されたジェスチャー
    pub detected_gestures: Vec<(GestureType, f32)>,
    /// タップの許容時間（ミリ秒）
    pub tap_duration_ms: u64,
    /// ダブルタップの最大間隔（ミリ秒）
    pub double_tap_interval_ms: u64,
    /// 長押しの時間（ミリ秒）
    pub long_press_duration_ms: u64,
    /// スワイプの最小距離（ピクセル）
    pub min_swipe_distance: f32,
    /// 最後のタップの時間
    pub last_tap_time: Option<f64>,
    /// 最後のタップの位置
    pub last_tap_position: Option<(f32, f32)>,
}

impl GestureDetector {
    /// 新しいジェスチャー検出器を作成
    pub fn new() -> Self {
        Self {
            detected_gestures: Vec::new(),
            tap_duration_ms: 200,
            double_tap_interval_ms: 300,
            long_press_duration_ms: 500,
            min_swipe_distance: 50.0,
            last_tap_time: None,
            last_tap_position: None,
        }
    }
    
    /// ジェスチャーを検出
    pub fn detect_gestures(&mut self, touch_points: &HashMap<TouchId, TouchPoint>, touch_points_previous: &HashMap<TouchId, TouchPoint>) {
        self.detected_gestures.clear();
        
        let now = Date::now();
        
        // タップとロングプレスの検出
        for (id, point) in touch_points.iter() {
            if !point.is_active && touch_points_previous.get(id).map_or(false, |p| p.is_active) {
                // タッチが終了した
                let duration = now - point.start_time;
                
                if duration <= self.tap_duration_ms as f64 {
                    // タップ検出
                    self.detected_gestures.push((GestureType::Tap, 1.0));
                    
                    // ダブルタップ検出
                    if let Some(last_time) = self.last_tap_time {
                        if now - last_time <= self.double_tap_interval_ms as f64 {
                            if let Some(last_pos) = self.last_tap_position {
                                let dx = point.position.0 - last_pos.0;
                                let dy = point.position.1 - last_pos.1;
                                let distance = (dx * dx + dy * dy).sqrt();
                                
                                if distance < 20.0 {
                                    self.detected_gestures.push((GestureType::DoubleTap, 1.0));
                                }
                            }
                        }
                    }
                    
                    self.last_tap_time = Some(now);
                    self.last_tap_position = Some(point.position);
                } else if duration >= self.long_press_duration_ms as f64 {
                    // 長押し検出
                    self.detected_gestures.push((GestureType::LongPress, 1.0));
                }
            }
        }
        
        // スワイプ検出
        for (id, point) in touch_points.iter() {
            if !point.is_active && touch_points_previous.get(id).map_or(false, |p| p.is_active) {
                // タッチが終了した
                if let Some(prev) = touch_points_previous.get(id) {
                    let dx = point.position.0 - prev.position.0;
                    let dy = point.position.1 - prev.position.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance >= self.min_swipe_distance {
                        // スワイプ方向を決定
                        let direction = if dx.abs() > dy.abs() {
                            if dx > 0.0 {
                                SwipeDirection::Right
                            } else {
                                SwipeDirection::Left
                            }
                        } else {
                            if dy > 0.0 {
                                SwipeDirection::Down
                            } else {
                                SwipeDirection::Up
                            }
                        };
                        
                        self.detected_gestures.push((GestureType::Swipe(direction), distance / 100.0));
                    }
                }
            }
        }
        
        // ピンチと回転の検出（2本指）
        if touch_points.len() >= 2 {
            let points: Vec<&TouchPoint> = touch_points.values().collect();
            let prev_points: Vec<&TouchPoint> = touch_points_previous.values().collect();
            
            if points.len() >= 2 && prev_points.len() >= 2 {
                // 現在の2点間の距離
                let dx1 = points[0].position.0 - points[1].position.0;
                let dy1 = points[0].position.1 - points[1].position.1;
                let current_distance = (dx1 * dx1 + dy1 * dy1).sqrt();
                
                // 前回の2点間の距離
                let dx2 = prev_points[0].position.0 - prev_points[1].position.0;
                let dy2 = prev_points[0].position.1 - prev_points[1].position.1;
                let prev_distance = (dx2 * dx2 + dy2 * dy2).sqrt();
                
                // ピンチ検出
                let distance_delta = current_distance - prev_distance;
                if distance_delta.abs() > 10.0 {
                    self.detected_gestures.push((GestureType::Pinch, distance_delta / 100.0));
                }
                
                // 回転検出
                let angle1 = dy1.atan2(dx1);
                let angle2 = dy2.atan2(dx2);
                let angle_delta = angle2 - angle1;
                
                if angle_delta.abs() > 0.1 {
                    self.detected_gestures.push((GestureType::Rotate, angle_delta));
                }
            }
        }
    }
    
    /// 特定のジェスチャーが検出されたかチェック
    pub fn is_gesture_detected(&self, gesture_type: &GestureType) -> bool {
        self.detected_gestures.iter().any(|(g, _)| g == gesture_type)
    }
    
    /// 特定のジェスチャーの強度を取得
    pub fn get_gesture_strength(&self, gesture_type: &GestureType) -> f32 {
        self.detected_gestures.iter()
            .find(|(g, _)| g == gesture_type)
            .map(|(_, strength)| *strength)
            .unwrap_or(0.0)
    }
}

/// キーコンフィグ
#[derive(Debug, Clone)]
pub struct KeyConfig {
    /// キーバインド（アクション名 -> キーコードのセット）
    pub key_bindings: HashMap<ActionName, HashSet<KeyCode>>,
    /// マウスバインド
    pub mouse_bindings: HashMap<ActionName, HashSet<MouseButton>>,
    /// キーリピート設定
    pub key_repeat: HashMap<KeyCode, KeyRepeatConfig>,
}

/// キーリピート設定
#[derive(Debug, Clone)]
pub struct KeyRepeatConfig {
    /// リピート開始までの遅延（ミリ秒）
    pub delay_ms: u64,
    /// リピート間隔（ミリ秒）
    pub interval_ms: u64,
}

/// 入力アクションマッピング
#[derive(Debug, Clone)]
pub struct ActionMapping {
    /// キーコンフィグ
    pub config: KeyConfig,
    /// アクティブなアクション
    pub active_actions: HashSet<ActionName>,
    /// 前回のフレームでのアクティブなアクション
    pub previous_actions: HashSet<ActionName>,
    /// アクションが開始された時間（エポックからのミリ秒）
    pub action_start_time: HashMap<ActionName, f64>,
    /// アクションの入力値（アナログ値、0.0～1.0）
    pub action_values: HashMap<ActionName, f32>,
}

impl ActionMapping {
    /// 新しいアクションマッピングを作成
    pub fn new() -> Self {
        Self {
            config: KeyConfig {
                key_bindings: HashMap::new(),
                mouse_bindings: HashMap::new(),
                key_repeat: HashMap::new(),
            },
            active_actions: HashSet::new(),
            previous_actions: HashSet::new(),
            action_start_time: HashMap::new(),
            action_values: HashMap::new(),
        }
    }
    
    /// アクションにキーをバインド
    pub fn bind_key(&mut self, action: &str, key_code: KeyCode) -> &mut Self {
        self.config.key_bindings
            .entry(action.to_string())
            .or_insert_with(HashSet::new)
            .insert(key_code);
        self
    }
    
    /// アクションからキーバインドを削除
    pub fn unbind_key(&mut self, action: &str, key_code: KeyCode) -> &mut Self {
        if let Some(bindings) = self.config.key_bindings.get_mut(action) {
            bindings.remove(&key_code);
        }
        self
    }
    
    /// アクションにマウスボタンをバインド
    pub fn bind_mouse_button(&mut self, action: &str, button: MouseButton) -> &mut Self {
        self.config.mouse_bindings
            .entry(action.to_string())
            .or_insert_with(HashSet::new)
            .insert(button);
        self
    }
    
    /// キーリピート設定を設定
    pub fn set_key_repeat(&mut self, key_code: KeyCode, delay_ms: u64, interval_ms: u64) -> &mut Self {
        self.config.key_repeat.insert(key_code, KeyRepeatConfig {
            delay_ms,
            interval_ms,
        });
        self
    }
    
    /// アクションが現在アクティブかどうかチェック
    pub fn is_action_active(&self, action: &str) -> bool {
        self.active_actions.contains(action)
    }
    
    /// アクションが今回のフレームで開始されたかどうかチェック
    pub fn is_action_just_pressed(&self, action: &str) -> bool {
        self.active_actions.contains(action) && !self.previous_actions.contains(action)
    }
    
    /// アクションが今回のフレームで終了したかどうかチェック
    pub fn is_action_just_released(&self, action: &str) -> bool {
        !self.active_actions.contains(action) && self.previous_actions.contains(action)
    }
    
    /// アクションの値を取得（アナログ値、0.0～1.0）
    pub fn get_action_value(&self, action: &str) -> f32 {
        *self.action_values.get(action).unwrap_or(&0.0)
    }
    
    /// アクションの持続時間を取得（ミリ秒）
    pub fn get_action_duration(&self, action: &str) -> Option<f64> {
        if let Some(start_time) = self.action_start_time.get(action) {
            if self.active_actions.contains(action) {
                // アクションがアクティブならば、現在時刻との差分を返す
                let now = Date::now();
                Some(now - start_time)
            } else {
                None // アクションが非アクティブならば持続時間なし
            }
        } else {
            None // 開始時間がなければ持続時間なし
        }
    }
}

impl InputState {
    /// 新しい入力状態を作成
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_previous: HashSet::new(),
            key_press_duration: HashMap::new(),
            key_last_press_time: HashMap::new(),
            key_last_repeat_time: HashMap::new(),
            
            mouse_position: (0.0, 0.0),
            mouse_previous: (0.0, 0.0),
            mouse_velocity: (0.0, 0.0),
            mouse_buttons: HashMap::new(),
            mouse_buttons_previous: HashMap::new(),
            mouse_wheel_delta: 0.0,
            
            touch_points: HashMap::new(),
            touch_points_previous: HashMap::new(),
            gesture_detector: GestureDetector::new(),
            
            action_mapping: ActionMapping::new(),
        }
    }
    
    /// キーの状態を更新
    pub fn update_key(&mut self, key_code: KeyCode, is_pressed: bool) {
        let now = Date::now();
        
        if is_pressed {
            if !self.keys_pressed.contains(&key_code) {
                // キーが新しく押された
                self.key_last_press_time.insert(key_code, now);
                self.key_last_repeat_time.insert(key_code, now);
            }
            self.keys_pressed.insert(key_code);
        } else {
            self.keys_pressed.remove(&key_code);
        }
    }
    
    /// マウスの位置を更新
    pub fn update_mouse_position(&mut self, x: f32, y: f32, delta_time: f32) {
        self.mouse_previous = self.mouse_position;
        self.mouse_position = (x, y);
        
        // 速度を計算
        if delta_time > 0.0 {
            self.mouse_velocity = (
                (self.mouse_position.0 - self.mouse_previous.0) / delta_time,
                (self.mouse_position.1 - self.mouse_previous.1) / delta_time,
            );
        }
    }
    
    /// マウスボタンの状態を更新
    pub fn update_mouse_button(&mut self, button: MouseButton, is_pressed: bool) {
        self.mouse_buttons.insert(button, is_pressed);
    }
    
    /// マウスホイールの値を更新
    pub fn update_mouse_wheel(&mut self, delta: f32) {
        self.mouse_wheel_delta = delta;
    }
    
    /// タッチポイントを更新
    pub fn update_touch_point(&mut self, id: TouchId, x: f32, y: f32, force: f32, is_active: bool, delta_time: f32) {
        let now = Date::now();
        
        if let Some(point) = self.touch_points.get_mut(&id) {
            point.previous_position = point.position;
            point.position = (x, y);
            point.force = force;
            point.is_active = is_active;
            
            // 速度を計算
            if delta_time > 0.0 {
                point.velocity = (
                    (point.position.0 - point.previous_position.0) / delta_time,
                    (point.position.1 - point.previous_position.1) / delta_time,
                );
            }
        } else if is_active {
            // 新しいタッチポイントを追加
            self.touch_points.insert(id, TouchPoint {
                id,
                position: (x, y),
                previous_position: (x, y),
                velocity: (0.0, 0.0),
                force,
                is_active,
                start_time: now,
            });
        }
    }
    
    /// アクションの状態を更新
    pub fn update_actions(&mut self) {
        let now = Date::now();
        
        // 前回の状態を保存
        self.action_mapping.previous_actions = self.action_mapping.active_actions.clone();
        self.action_mapping.active_actions.clear();
        
        // キーバインドからアクションを更新
        for (action, key_codes) in &self.action_mapping.config.key_bindings {
            let is_active = key_codes.iter().any(|code| self.keys_pressed.contains(code));
            
            if is_active {
                if !self.action_mapping.active_actions.contains(action) {
                    // アクションが新しくアクティブになった
                    self.action_mapping.action_start_time.insert(action.clone(), now);
                }
                self.action_mapping.active_actions.insert(action.clone());
                
                // アナログ値（デジタルキーなので0か1）
                self.action_mapping.action_values.insert(action.clone(), 1.0);
            }
        }
        
        // マウスバインドからアクションを更新
        for (action, buttons) in &self.action_mapping.config.mouse_bindings {
            let is_active = buttons.iter().any(|button| *self.mouse_buttons.get(button).unwrap_or(&false));
            
            if is_active {
                if !self.action_mapping.active_actions.contains(action) {
                    // アクションが新しくアクティブになった
                    self.action_mapping.action_start_time.insert(action.clone(), now);
                }
                self.action_mapping.active_actions.insert(action.clone());
                
                // アナログ値（デジタルボタンなので0か1）
                self.action_mapping.action_values.insert(action.clone(), 1.0);
            }
        }
        
        // キーリピート処理
        for (key_code, repeat_config) in &self.action_mapping.config.key_repeat {
            if self.keys_pressed.contains(key_code) {
                if let Some(press_time) = self.key_last_press_time.get(key_code) {
                    let elapsed = now - press_time;
                    
                    if elapsed >= repeat_config.delay_ms as f64 {
                        if let Some(repeat_time) = self.key_last_repeat_time.get(key_code) {
                            let repeat_elapsed = now - repeat_time;
                            
                            if repeat_elapsed >= repeat_config.interval_ms as f64 {
                                // リピート発火
                                for (action, key_codes) in &self.action_mapping.config.key_bindings {
                                    if key_codes.contains(key_code) {
                                        self.action_mapping.active_actions.insert(action.clone());
                                        self.key_last_repeat_time.insert(*key_code, now);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// 入力状態を更新（フレーム毎に呼び出す）
    pub fn update(&mut self, delta_time: f32) {
        // ジェスチャー検出
        self.gesture_detector.detect_gestures(&self.touch_points, &self.touch_points_previous);
        
        // アクションの更新
        self.update_actions();
        
        // 前回の状態を更新
        self.keys_previous = self.keys_pressed.clone();
        self.mouse_buttons_previous = self.mouse_buttons.clone();
        self.touch_points_previous = self.touch_points.clone();
        
        // 非アクティブなタッチポイントを削除
        self.touch_points.retain(|_, point| point.is_active);
        
        // マウスホイールのリセット
        self.mouse_wheel_delta = 0.0;
    }
    
    /// キーが押されているかチェック
    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        self.keys_pressed.contains(&key_code)
    }
    
    /// キーが今回のフレームで押されたかチェック
    pub fn is_key_just_pressed(&self, key_code: KeyCode) -> bool {
        self.keys_pressed.contains(&key_code) && !self.keys_previous.contains(&key_code)
    }
    
    /// キーが今回のフレームで離されたかチェック
    pub fn is_key_just_released(&self, key_code: KeyCode) -> bool {
        !self.keys_pressed.contains(&key_code) && self.keys_previous.contains(&key_code)
    }
    
    /// マウスボタンが押されているかチェック
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        *self.mouse_buttons.get(&button).unwrap_or(&false)
    }
    
    /// マウスボタンが今回のフレームで押されたかチェック
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        *self.mouse_buttons.get(&button).unwrap_or(&false) && 
        !*self.mouse_buttons_previous.get(&button).unwrap_or(&false)
    }
    
    /// マウスボタンが今回のフレームで離されたかチェック
    pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        !*self.mouse_buttons.get(&button).unwrap_or(&false) && 
        *self.mouse_buttons_previous.get(&button).unwrap_or(&false)
    }
}

/// 入力コンポーネント
#[derive(Debug, Clone)]
pub struct InputComponent {
    /// このエンティティが入力を受け付けるかどうか
    pub is_controllable: bool,
    /// このエンティティに関連付けられた入力アクション
    pub action_handlers: HashMap<ActionName, ActionHandler>,
}

/// アクションハンドラー
pub type ActionHandler = fn(entity: Entity, world: &mut World, value: f32) -> Result<(), JsValue>;

impl Component for InputComponent {
    fn name() -> &'static str {
        "InputComponent"
    }
}

impl InputComponent {
    /// 新しい入力コンポーネントを作成
    pub fn new(is_controllable: bool) -> Self {
        Self {
            is_controllable,
            action_handlers: HashMap::new(),
        }
    }
    
    /// アクションハンドラーを追加
    pub fn add_action_handler(&mut self, action: &str, handler: ActionHandler) -> &mut Self {
        self.action_handlers.insert(action.to_string(), handler);
        self
    }
}

/// 入力処理システム
#[derive(Debug)]
pub struct InputSystem {
    /// 入力状態
    pub state: InputState,
}

impl InputSystem {
    /// 新しい入力処理システムを作成
    pub fn new() -> Self {
        let mut state = InputState::new();
        
        // デフォルトのキーバインドを設定
        Self::setup_default_bindings(&mut state);
        
        Self {
            state,
        }
    }
    
    /// キーボードイベントを処理
    pub fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        // イベントタイプに基づいて処理
        let is_pressed = event.event_type == "keydown";
        let key_code = event.key.parse::<KeyCode>().unwrap_or(0);
        
        // 入力状態を更新
        self.state.update_key(key_code, is_pressed);
    }
    
    /// マウスイベントを処理
    pub fn handle_mouse_event(&mut self, event: &MouseEvent) {
        // 位置の更新
        self.state.update_mouse_position(event.position.0, event.position.1, 0.016);
        
        // ボタンイベントの処理
        if let Some(button) = event.button {
            let button = button as MouseButton;
            let is_pressed = event.event_type == "mousedown";
            
            self.state.update_mouse_button(button, is_pressed);
        }
    }
    
    /// デフォルトのキーバインドを設定
    fn setup_default_bindings(state: &mut InputState) {
        // キーボード
        state.action_mapping.bind_key("move_up", KEY_W)
            .bind_key("move_up", KEY_UP)
            .bind_key("move_down", KEY_S)
            .bind_key("move_down", KEY_DOWN)
            .bind_key("move_left", KEY_A)
            .bind_key("move_left", KEY_LEFT)
            .bind_key("move_right", KEY_D)
            .bind_key("move_right", KEY_RIGHT)
            .bind_key("jump", KEY_SPACE)
            .bind_key("action", KEY_E)
            .bind_key("attack", MOUSE_LEFT.into());
        
        // マウス
        state.action_mapping.bind_mouse_button("attack", 0)
            .bind_mouse_button("aim", 1);
        
        // キーリピート設定
        state.action_mapping.set_key_repeat(KEY_W, 300, 100)
            .set_key_repeat(KEY_S, 300, 100)
            .set_key_repeat(KEY_A, 300, 100)
            .set_key_repeat(KEY_D, 300, 100);
    }
}

impl System for InputSystem {
    fn name(&self) -> &'static str {
        "InputSystem"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Input
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(0) // 入力処理は優先度0（最優先）
    }

    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        // 入力状態を更新
        self.state.update(delta_time);
        
        // 入力コンポーネントを持つエンティティを取得
        let mut query = Query::<InputComponent>::new();
        query.run(world)?;
        let entities = query.entities();
        
        // 実行するアクションを一時的に保存する構造
        struct ActionToExecute {
            entity: Entity,
            action: String,
            handler: ActionHandler,
            value: f32,
        }
        
        // 各エンティティのアクションハンドラーを収集
        let mut actions_to_execute = Vec::new();
        
        for entity in entities {
            if let Some(input_component) = world.get_component::<InputComponent>(entity) {
                if !input_component.is_controllable {
                    continue;
                }
                
                for (action, handler) in &input_component.action_handlers {
                    if self.state.action_mapping.is_action_active(action) {
                        let value = self.state.action_mapping.get_action_value(action);
                        actions_to_execute.push(ActionToExecute {
                            entity,
                            action: action.clone(),
                            handler: *handler,
                            value,
                        });
                    }
                }
            }
        }
        
        // 収集したアクションハンドラーを実行
        for action in actions_to_execute {
            (action.handler)(action.entity, world, action.value)?;
        }
        
        Ok(())
    }
}

/// 入力システムを初期化してワールドに登録します。
pub fn init_input_system(world: &mut World) {
    // InputResourceを作成して登録
    let input_resource = InputResource::new();
    world.insert_resource(input_resource);

    // 既存の InputSystem の登録は不要になる（InputResourceに含まれるため）
    // world.register_system(InputSystem::new());
    // 既存の InputState の登録も不要になる（InputResourceに含まれるため）
    // world.insert_resource(InputState::new());

    log::info!("⌨️ InputResource を初期化して登録しました");
}

/// 入力リソース
/// 入力状態を管理するリソース
#[derive(Debug, Resource)]
pub struct InputResource {
    /// 入力状態
    pub state: InputState,
    /// 入力システム
    pub system: InputSystem,
}

impl InputResource {
    /// 新しい入力リソースを作成
    pub fn new() -> Self {
        Self {
            state: InputState::new(),
            system: InputSystem::new(),
        }
    }
    
    /// キーボードイベントを処理
    pub fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        self.system.handle_keyboard_event(event);
    }
    
    /// マウスイベントを処理
    pub fn handle_mouse_event(&mut self, event: &MouseEvent) {
        self.system.handle_mouse_event(event);
    }
    
    /// 入力状態を更新
    pub fn update(&mut self, delta_time: f32) {
        self.state.update(delta_time);
    }
    
    /// マウス位置を設定
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        // delta_timeは小さな値を使用（実際の時間は不明なため）
        self.state.update_mouse_position(x, y, 0.016);
    }
    
    /// マウス位置を取得
    pub fn get_mouse_position(&self) -> (f32, f32) {
        self.state.mouse_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_state_creation() {
        let input_state = InputState::new();
        assert!(input_state.keys_pressed.is_empty());
        assert!(input_state.mouse_buttons.is_empty());
        assert!(input_state.touch_points.is_empty());
    }
    
    #[test]
    fn test_key_update() {
        let mut input_state = InputState::new();
        input_state.update_key(32, true); // Space key
        assert!(input_state.is_key_pressed(32));
        assert!(input_state.is_key_just_pressed(32));
        
        // 2回目のフレーム更新
        input_state.keys_previous = input_state.keys_pressed.clone();
        assert!(input_state.is_key_pressed(32));
        assert!(!input_state.is_key_just_pressed(32));
        
        // キーを離す
        input_state.update_key(32, false);
        assert!(!input_state.is_key_pressed(32));
        assert!(input_state.is_key_just_released(32));
    }
    
    #[test]
    fn test_action_mapping() {
        let mut action_mapping = ActionMapping::new();
        
        // アクションにキーをバインド
        action_mapping.bind_key("jump", 32);
        
        let mut input_state = InputState::new();
        input_state.action_mapping = action_mapping;
        
        // キーを押す
        input_state.update_key(32, true);
        input_state.update_actions();
        
        assert!(input_state.action_mapping.is_action_active("jump"));
        assert!(input_state.action_mapping.is_action_just_pressed("jump"));
        assert_eq!(input_state.action_mapping.get_action_value("jump"), 1.0);
        
        // 2回目のフレーム更新
        input_state.action_mapping.previous_actions = input_state.action_mapping.active_actions.clone();
        input_state.update_actions();
        
        assert!(input_state.action_mapping.is_action_active("jump"));
        assert!(!input_state.action_mapping.is_action_just_pressed("jump"));
        
        // キーを離す
        input_state.update_key(32, false);
        input_state.update_actions();
        
        assert!(!input_state.action_mapping.is_action_active("jump"));
        assert!(input_state.action_mapping.is_action_just_released("jump"));
    }
} 