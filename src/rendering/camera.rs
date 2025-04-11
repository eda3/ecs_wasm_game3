//! カメラモジュール
//! 
//! ゲームのカメラシステムを管理します。
//! ワールド座標からスクリーン座標への変換や、
//! カメラの移動、ズームなどの機能を提供します。

use std::time::Duration;

/// カメラ構造体
/// 
/// ゲームのカメラを管理します。
pub struct Camera {
    position: (f64, f64),
    zoom: f64,
    target: Option<(f64, f64)>,
    smooth_speed: f64,
    bounds: Option<((f64, f64), (f64, f64))>,
}

impl Camera {
    /// 新しいカメラを作成
    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0),
            zoom: 1.0,
            target: None,
            smooth_speed: 0.1,
            bounds: None,
        }
    }

    /// カメラの位置を設定
    pub fn set_position(&mut self, x: f64, y: f64) {
        self.position = (x, y);
    }

    /// カメラのズームを設定
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.max(0.1);
    }

    /// カメラの追従対象を設定
    pub fn set_target(&mut self, x: f64, y: f64) {
        self.target = Some((x, y));
    }

    /// カメラの追従を解除
    pub fn clear_target(&mut self) {
        self.target = None;
    }

    /// カメラの移動範囲を設定
    pub fn set_bounds(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.bounds = Some(((min_x, min_y), (max_x, max_y)));
    }

    /// カメラの移動範囲を解除
    pub fn clear_bounds(&mut self) {
        self.bounds = None;
    }

    /// カメラを更新
    pub fn update(&mut self, delta_time: Duration) {
        if let Some((target_x, target_y)) = self.target {
            let delta_seconds = delta_time.as_secs_f64();
            let dx = (target_x - self.position.0) * self.smooth_speed * delta_seconds;
            let dy = (target_y - self.position.1) * self.smooth_speed * delta_seconds;
            
            self.position.0 += dx;
            self.position.1 += dy;
        }

        // 境界チェック
        if let Some(((min_x, min_y), (max_x, max_y))) = self.bounds {
            self.position.0 = self.position.0.max(min_x).min(max_x);
            self.position.1 = self.position.1.max(min_y).min(max_y);
        }
    }

    /// ワールド座標をスクリーン座標に変換
    pub fn world_to_screen(&self, x: f64, y: f64) -> (f64, f64) {
        let screen_x = (x - self.position.0) * self.zoom;
        let screen_y = (y - self.position.1) * self.zoom;
        (screen_x, screen_y)
    }

    /// スクリーン座標をワールド座標に変換
    pub fn screen_to_world(&self, x: f64, y: f64) -> (f64, f64) {
        let world_x = x / self.zoom + self.position.0;
        let world_y = y / self.zoom + self.position.1;
        (world_x, world_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new();
        assert_eq!(camera.position, (0.0, 0.0));
        assert_eq!(camera.zoom, 1.0);
        assert!(camera.target.is_none());
        assert!(camera.bounds.is_none());
    }

    #[test]
    fn test_camera_position() {
        let mut camera = Camera::new();
        camera.set_position(100.0, 200.0);
        assert_eq!(camera.position, (100.0, 200.0));
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::new();
        camera.set_zoom(2.0);
        assert_eq!(camera.zoom, 2.0);
        
        // 最小値チェック
        camera.set_zoom(0.0);
        assert_eq!(camera.zoom, 0.1);
    }

    #[test]
    fn test_camera_target() {
        let mut camera = Camera::new();
        camera.set_target(300.0, 400.0);
        assert_eq!(camera.target, Some((300.0, 400.0)));
        
        camera.clear_target();
        assert!(camera.target.is_none());
    }

    #[test]
    fn test_camera_bounds() {
        let mut camera = Camera::new();
        camera.set_bounds(0.0, 0.0, 800.0, 600.0);
        assert_eq!(camera.bounds, Some(((0.0, 0.0), (800.0, 600.0))));
        
        camera.clear_bounds();
        assert!(camera.bounds.is_none());
    }

    #[test]
    fn test_coordinate_conversion() {
        let mut camera = Camera::new();
        camera.set_position(100.0, 100.0);
        camera.set_zoom(2.0);
        
        // ワールド座標からスクリーン座標への変換
        let (screen_x, screen_y) = camera.world_to_screen(200.0, 200.0);
        assert_eq!(screen_x, 200.0);
        assert_eq!(screen_y, 200.0);
        
        // スクリーン座標からワールド座標への変換
        let (world_x, world_y) = camera.screen_to_world(200.0, 200.0);
        assert_eq!(world_x, 200.0);
        assert_eq!(world_y, 200.0);
    }
} 