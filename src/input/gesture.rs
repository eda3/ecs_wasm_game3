//! ジェスチャー検出モジュール
//!
//! このモジュールはタッチ入力からジェスチャーを検出する機能を提供します。
//! サポートされるジェスチャー: タップ、ダブルタップ、スワイプ、ピンチ、ロングタップ

use crate::math::Vector2;
use std::time::{Duration, Instant};

/// ジェスチャーの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureType {
    /// シングルタップ
    Tap,
    /// ダブルタップ
    DoubleTap,
    /// 長押し
    LongPress,
    /// スワイプ (方向ベクトル付き)
    Swipe,
    /// ピンチイン/アウト (スケール値付き)
    Pinch,
    /// 回転 (角度付き)
    Rotate,
}

/// ジェスチャー情報
#[derive(Debug, Clone)]
pub struct GestureInfo {
    /// ジェスチャーの種類
    pub gesture_type: GestureType,
    /// ジェスチャーの位置
    pub position: Vector2,
    /// ジェスチャーの方向 (該当する場合)
    pub direction: Option<Vector2>,
    /// ジェスチャーの強度/大きさ
    pub magnitude: f32,
    /// ジェスチャーが発生した時間
    pub timestamp: Instant,
}

impl GestureInfo {
    /// 新しいジェスチャー情報を作成
    pub fn new(
        gesture_type: GestureType,
        position: Vector2,
        direction: Option<Vector2>,
        magnitude: f32,
    ) -> Self {
        Self {
            gesture_type,
            position,
            direction,
            magnitude,
            timestamp: Instant::now(),
        }
    }

    /// タップジェスチャーを作成
    pub fn tap(position: Vector2) -> Self {
        Self::new(GestureType::Tap, position, None, 1.0)
    }

    /// ダブルタップジェスチャーを作成
    pub fn double_tap(position: Vector2) -> Self {
        Self::new(GestureType::DoubleTap, position, None, 1.0)
    }

    /// 長押しジェスチャーを作成
    pub fn long_press(position: Vector2, duration: Duration) -> Self {
        Self::new(
            GestureType::LongPress,
            position,
            None,
            duration.as_secs_f32(),
        )
    }

    /// スワイプジェスチャーを作成
    pub fn swipe(start: Vector2, end: Vector2, speed: f32) -> Self {
        let direction = end - start;
        let normalized_direction = direction.normalize();
        Self::new(
            GestureType::Swipe,
            start,
            Some(normalized_direction),
            speed,
        )
    }

    /// ピンチジェスチャーを作成
    pub fn pinch(center: Vector2, scale_factor: f32) -> Self {
        Self::new(GestureType::Pinch, center, None, scale_factor)
    }

    /// 回転ジェスチャーを作成
    pub fn rotate(center: Vector2, angle: f32) -> Self {
        Self::new(GestureType::Rotate, center, None, angle)
    }
}

/// ジェスチャー検出器
pub struct GestureDetector {
    /// タップの時間閾値 (秒)
    tap_duration_threshold: f32,
    /// ダブルタップの時間閾値 (秒)
    double_tap_duration_threshold: f32,
    /// 長押しの時間閾値 (秒)
    long_press_duration_threshold: f32,
    /// スワイプの最小距離閾値
    swipe_min_distance: f32,
    /// ピンチの最小スケール変化
    pinch_min_scale_change: f32,
    /// 回転の最小角度変化 (ラジアン)
    rotation_min_angle_change: f32,
    /// 最後のタップ時間
    last_tap_time: Option<Instant>,
    /// 最後のタップ位置
    last_tap_position: Option<Vector2>,
    /// 現在処理中のタッチの開始時間
    touch_start_time: Option<Instant>,
    /// 現在処理中のタッチの開始位置
    touch_start_position: Option<Vector2>,
    /// 2本指のピンチ/回転の初期距離
    initial_pinch_distance: Option<f32>,
    /// 2本指のピンチ/回転の初期角度
    initial_rotation_angle: Option<f32>,
}

impl Default for GestureDetector {
    fn default() -> Self {
        Self {
            tap_duration_threshold: 0.2,
            double_tap_duration_threshold: 0.3,
            long_press_duration_threshold: 0.5,
            swipe_min_distance: 50.0,
            pinch_min_scale_change: 0.1,
            rotation_min_angle_change: 0.1,
            last_tap_time: None,
            last_tap_position: None,
            touch_start_time: None,
            touch_start_position: None,
            initial_pinch_distance: None,
            initial_rotation_angle: None,
        }
    }
}

impl GestureDetector {
    /// 新しいジェスチャー検出器を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// カスタム設定でジェスチャー検出器を作成
    pub fn with_config(
        tap_threshold: f32,
        double_tap_threshold: f32,
        long_press_threshold: f32,
        swipe_distance: f32,
        pinch_scale: f32,
        rotation_angle: f32,
    ) -> Self {
        Self {
            tap_duration_threshold: tap_threshold,
            double_tap_duration_threshold: double_tap_threshold,
            long_press_duration_threshold: long_press_threshold,
            swipe_min_distance: swipe_distance,
            pinch_min_scale_change: pinch_scale,
            rotation_min_angle_change: rotation_angle,
            ..Self::default()
        }
    }

    /// タッチの開始を処理
    pub fn touch_began(&mut self, position: Vector2) {
        let now = Instant::now();
        self.touch_start_time = Some(now);
        self.touch_start_position = Some(position);
    }

    /// タッチの移動を処理
    pub fn touch_moved(&mut self, current_position: Vector2) -> Option<GestureInfo> {
        if let Some(start_pos) = self.touch_start_position {
            let distance = (current_position - start_pos).length();
            
            // スワイプジェスチャーの検出
            if distance > self.swipe_min_distance {
                let start_time = self.touch_start_time.unwrap();
                let duration = Instant::now().duration_since(start_time).as_secs_f32();
                let speed = distance / duration;
                
                // スワイプの情報を返す
                let gesture = GestureInfo::swipe(start_pos, current_position, speed);
                
                // スワイプが検出されたらタッチ開始情報をリセット
                self.touch_start_time = None;
                self.touch_start_position = None;
                
                return Some(gesture);
            }
        }
        
        None
    }

    /// タッチの終了を処理
    pub fn touch_ended(&mut self, end_position: Vector2) -> Option<GestureInfo> {
        let now = Instant::now();
        
        if let (Some(start_time), Some(start_pos)) = (self.touch_start_time, self.touch_start_position) {
            let duration = now.duration_since(start_time).as_secs_f32();
            let distance = (end_position - start_pos).length();
            
            // タップの検出
            if duration < self.tap_duration_threshold && distance < 10.0 {
                // 前回のタップからの時間を確認
                if let Some(last_time) = self.last_tap_time {
                    let time_since_last_tap = now.duration_since(last_time).as_secs_f32();
                    
                    // ダブルタップの検出
                    if time_since_last_tap < self.double_tap_duration_threshold {
                        if let Some(last_pos) = self.last_tap_position {
                            let tap_distance = (end_position - last_pos).length();
                            
                            // 前回と今回のタップ位置が近い場合はダブルタップ
                            if tap_distance < 30.0 {
                                self.last_tap_time = None;
                                self.last_tap_position = None;
                                
                                let gesture = GestureInfo::double_tap(end_position);
                                return Some(gesture);
                            }
                        }
                    }
                }
                
                // シングルタップとして記録
                self.last_tap_time = Some(now);
                self.last_tap_position = Some(end_position);
                
                let gesture = GestureInfo::tap(end_position);
                return Some(gesture);
            }
            // 長押しの検出
            else if duration >= self.long_press_duration_threshold && distance < 10.0 {
                let gesture = GestureInfo::long_press(
                    end_position,
                    Duration::from_secs_f32(duration),
                );
                return Some(gesture);
            }
            // スワイプの検出
            else if distance >= self.swipe_min_distance {
                let speed = distance / duration;
                let gesture = GestureInfo::swipe(start_pos, end_position, speed);
                return Some(gesture);
            }
        }
        
        // どのジェスチャーも検出されなかった場合
        self.touch_start_time = None;
        self.touch_start_position = None;
        None
    }

    /// 2本指のピンチを処理
    pub fn handle_pinch(&mut self, touch1: Vector2, touch2: Vector2) -> Option<GestureInfo> {
        let current_distance = (touch2 - touch1).length();
        let center = (touch1 + touch2) * 0.5;
        
        if let Some(initial_distance) = self.initial_pinch_distance {
            let scale_factor = current_distance / initial_distance;
            
            // スケール変化が閾値を超えた場合
            if (scale_factor - 1.0).abs() > self.pinch_min_scale_change {
                let gesture = GestureInfo::pinch(center, scale_factor);
                self.initial_pinch_distance = Some(current_distance); // 継続的な検出のために更新
                return Some(gesture);
            }
        } else {
            // 初回の記録
            self.initial_pinch_distance = Some(current_distance);
        }
        
        None
    }

    /// 2本指の回転を処理
    pub fn handle_rotation(&mut self, touch1: Vector2, touch2: Vector2) -> Option<GestureInfo> {
        let center = (touch1 + touch2) * 0.5;
        let direction = touch2 - touch1;
        let current_angle = direction.y.atan2(direction.x);
        
        if let Some(initial_angle) = self.initial_rotation_angle {
            let angle_change = current_angle - initial_angle;
            
            // 角度変化が閾値を超えた場合
            if angle_change.abs() > self.rotation_min_angle_change {
                let gesture = GestureInfo::rotate(center, angle_change);
                self.initial_rotation_angle = Some(current_angle); // 継続的な検出のために更新
                return Some(gesture);
            }
        } else {
            // 初回の記録
            self.initial_rotation_angle = Some(current_angle);
        }
        
        None
    }

    /// 2本指のジェスチャー検出を終了
    pub fn end_multi_touch_gesture(&mut self) {
        self.initial_pinch_distance = None;
        self.initial_rotation_angle = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_tap_detection() {
        let mut detector = GestureDetector::default();
        let position = Vector2::new(100.0, 100.0);
        
        detector.touch_began(position);
        
        // 短い時間待機
        sleep(Duration::from_millis(100));
        
        let gesture = detector.touch_ended(position);
        assert!(gesture.is_some());
        
        let gesture = gesture.unwrap();
        assert_eq!(gesture.gesture_type, GestureType::Tap);
        assert_eq!(gesture.position, position);
    }

    #[test]
    fn test_swipe_detection() {
        let mut detector = GestureDetector::default();
        let start_pos = Vector2::new(100.0, 100.0);
        let end_pos = Vector2::new(200.0, 200.0);
        
        detector.touch_began(start_pos);
        
        // スワイプ距離が閾値を超えるまで移動
        let gesture = detector.touch_moved(end_pos);
        
        assert!(gesture.is_some());
        let gesture = gesture.unwrap();
        assert_eq!(gesture.gesture_type, GestureType::Swipe);
        assert_eq!(gesture.position, start_pos);
        assert!(gesture.direction.is_some());
    }
} 