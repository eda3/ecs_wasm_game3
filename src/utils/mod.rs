//! ユーティリティモジュール
//! 
//! このモジュールには、ゲーム全体で使用される一般的なユーティリティ関数や構造体が含まれています。

pub mod math;
pub mod time;
pub mod id_generator;
pub mod logger;

// サブモジュールの再エクスポート
pub use math::*;
pub use time::*;
pub use id_generator::*;
pub use logger::*;

/// 2次元ベクトル用の補間関数
/// 
/// # 引数
/// 
/// * `start` - 開始ベクトル (x, y)
/// * `end` - 終了ベクトル (x, y)
/// * `t` - 補間パラメータ (0.0 から 1.0)
/// 
/// # 戻り値
/// 
/// * 補間された新しいベクトル
pub fn lerp_vec2(start: (f32, f32), end: (f32, f32), t: f32) -> (f32, f32) {
    let t = t.clamp(0.0, 1.0);
    (
        start.0 + (end.0 - start.0) * t,
        start.1 + (end.1 - start.1) * t,
    )
}

/// 値の補間
/// 
/// # 引数
/// 
/// * `start` - 開始値
/// * `end` - 終了値
/// * `t` - 補間パラメータ (0.0 から 1.0)
/// 
/// # 戻り値
/// 
/// * 補間された新しい値
pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    start + (end - start) * t
}

/// 角度の補間（ラジアン）
/// 
/// 最短経路で補間します
/// 
/// # 引数
/// 
/// * `start` - 開始角度（ラジアン）
/// * `end` - 終了角度（ラジアン）
/// * `t` - 補間パラメータ (0.0 から 1.0)
/// 
/// # 戻り値
/// 
/// * 補間された新しい角度（ラジアン）
pub fn lerp_angle(start: f32, end: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    
    // 角度の差を計算し、-PI〜PIの範囲に正規化
    let mut delta = end - start;
    while delta > std::f32::consts::PI {
        delta -= 2.0 * std::f32::consts::PI;
    }
    while delta < -std::f32::consts::PI {
        delta += 2.0 * std::f32::consts::PI;
    }
    
    // 補間
    start + delta * t
}

/// 配列内の値を二分探索で検索
/// 
/// # 引数
/// 
/// * `arr` - ソート済みの配列
/// * `target` - 検索対象の値
/// 
/// # 戻り値
/// 
/// * `Some(index)` - 見つかった場合、そのインデックス
/// * `None` - 見つからなかった場合
pub fn binary_search<T: PartialOrd>(arr: &[T], target: &T) -> Option<usize> {
    let mut left = 0;
    let mut right = arr.len();
    
    while left < right {
        let mid = left + (right - left) / 2;
        
        if &arr[mid] < target {
            left = mid + 1;
        } else if &arr[mid] > target {
            right = mid;
        } else {
            return Some(mid); // 一致する要素を見つけた
        }
    }
    
    None // 見つからなかった
}

/// ランダムなIDを生成
/// 
/// # 戻り値
/// 
/// * ランダムな文字列ID
pub fn generate_random_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // 現在のタイムスタンプを取得
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    
    // ランダムな部分を生成
    let random_part = (timestamp % 10000) as u32;
    
    format!("{:x}-{:x}", timestamp, random_part)
} 