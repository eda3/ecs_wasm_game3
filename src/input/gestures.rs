//! ジェスチャー検出モジュール
//! 
//! このモジュールは、タッチスクリーンジェスチャーの検出と処理を担当します。
//! タップ、スワイプ、ピンチ、回転などのジェスチャーを検出します。

use std::time::Instant;

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
    pub last_tap_time: Option<Instant>,
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
    
    /// 特定のジェスチャーが検出されたかどうかを確認
    pub fn is_gesture_detected(&self, gesture_type: &GestureType) -> bool {
        self.detected_gestures.iter().any(|(g, _)| g == gesture_type)
    }
    
    /// 特定のジェスチャーの強度（0.0～1.0）を取得
    pub fn get_gesture_strength(&self, gesture_type: &GestureType) -> f32 {
        self.detected_gestures
            .iter()
            .find(|(g, _)| g == gesture_type)
            .map(|(_, strength)| *strength)
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gesture_detector_creation() {
        let detector = GestureDetector::new();
        assert_eq!(detector.detected_gestures.len(), 0);
        assert_eq!(detector.tap_duration_ms, 200);
    }
    
    #[test]
    fn test_gesture_detection() {
        let mut detector = GestureDetector::new();
        detector.detected_gestures.push((GestureType::Tap, 1.0));
        
        assert!(detector.is_gesture_detected(&GestureType::Tap));
        assert!(!detector.is_gesture_detected(&GestureType::DoubleTap));
        assert_eq!(detector.get_gesture_strength(&GestureType::Tap), 1.0);
        assert_eq!(detector.get_gesture_strength(&GestureType::DoubleTap), 0.0);
    }
} 