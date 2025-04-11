//! アニメーションシステムモジュール
//! 
//! このモジュールは、スプライトのアニメーションを管理します。
//! フレーム管理、アニメーション再生、イベント処理などを行います。

use std::collections::HashMap;
use std::time::Duration;

/// アニメーションフレーム構造体
/// 
/// 個々のアニメーションフレームの情報を保持します。
pub struct AnimationFrame {
    pub sprite_id: String,
    pub duration: Duration,
}

/// アニメーション構造体
/// 
/// 一連のアニメーションフレームを管理します。
pub struct Animation {
    frames: Vec<AnimationFrame>,
    current_frame: usize,
    elapsed_time: Duration,
    looping: bool,
    playing: bool,
}

/// アニメーションマネージャー構造体
/// 
/// 複数のアニメーションを管理します。
pub struct AnimationManager {
    animations: HashMap<String, Animation>,
}

impl AnimationFrame {
    /// 新しいアニメーションフレームを作成
    pub fn new(sprite_id: String, duration: Duration) -> Self {
        Self {
            sprite_id,
            duration,
        }
    }
}

impl Animation {
    /// 新しいアニメーションを作成
    pub fn new(frames: Vec<AnimationFrame>, looping: bool) -> Self {
        Self {
            frames,
            current_frame: 0,
            elapsed_time: Duration::from_secs(0),
            looping,
            playing: false,
        }
    }

    /// アニメーションを再生
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// アニメーションを一時停止
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// アニメーションを停止
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_frame = 0;
        self.elapsed_time = Duration::from_secs(0);
    }

    /// アニメーションを更新
    pub fn update(&mut self, delta_time: Duration) -> Option<&str> {
        if !self.playing || self.frames.is_empty() {
            return None;
        }

        self.elapsed_time += delta_time;
        let current_frame = &self.frames[self.current_frame];

        if self.elapsed_time >= current_frame.duration {
            self.elapsed_time = Duration::from_secs(0);
            self.current_frame += 1;

            if self.current_frame >= self.frames.len() {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.playing = false;
                    self.current_frame = self.frames.len() - 1;
                }
            }
        }

        Some(&self.frames[self.current_frame].sprite_id)
    }

    /// 現在のフレームのスプライトIDを取得
    pub fn current_sprite_id(&self) -> Option<&str> {
        if self.frames.is_empty() {
            None
        } else {
            Some(&self.frames[self.current_frame].sprite_id)
        }
    }
}

impl AnimationManager {
    /// 新しいアニメーションマネージャーを作成
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
        }
    }

    /// アニメーションを追加
    pub fn add_animation(&mut self, id: String, animation: Animation) {
        self.animations.insert(id, animation);
    }

    /// アニメーションを取得
    pub fn get_animation(&mut self, id: &str) -> Option<&mut Animation> {
        self.animations.get_mut(id)
    }

    /// アニメーションを更新
    pub fn update(&mut self, delta_time: Duration) {
        for animation in self.animations.values_mut() {
            animation.update(delta_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_animation_creation() {
        let frames = vec![
            AnimationFrame::new("frame1".to_string(), Duration::from_millis(100)),
            AnimationFrame::new("frame2".to_string(), Duration::from_millis(100)),
        ];
        let animation = Animation::new(frames, true);
        assert_eq!(animation.frames.len(), 2);
    }

    #[test]
    fn test_animation_playback() {
        let frames = vec![
            AnimationFrame::new("frame1".to_string(), Duration::from_millis(100)),
            AnimationFrame::new("frame2".to_string(), Duration::from_millis(100)),
        ];
        let mut animation = Animation::new(frames, true);
        
        animation.play();
        assert!(animation.playing);
        
        let sprite_id = animation.update(Duration::from_millis(50));
        assert_eq!(sprite_id, Some("frame1"));
        
        let sprite_id = animation.update(Duration::from_millis(100));
        assert_eq!(sprite_id, Some("frame2"));
    }

    #[test]
    fn test_animation_manager() {
        let mut manager = AnimationManager::new();
        
        let frames = vec![
            AnimationFrame::new("frame1".to_string(), Duration::from_millis(100)),
            AnimationFrame::new("frame2".to_string(), Duration::from_millis(100)),
        ];
        let animation = Animation::new(frames, true);
        
        manager.add_animation("test_anim".to_string(), animation);
        assert!(manager.get_animation("test_anim").is_some());
        
        manager.update(Duration::from_millis(50));
    }
} 