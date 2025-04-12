//! 数学ユーティリティモジュール
//! 
//! このモジュールには、ゲーム内で使用される数学関連のユーティリティ関数が含まれています。

use std::f32::consts::{PI, TAU};

/// 度数法からラジアンに変換
/// 
/// # 引数
/// 
/// * `degrees` - 度数法の角度
/// 
/// # 戻り値
/// 
/// * ラジアン角
pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.0
}

/// ラジアンから度数法に変換
/// 
/// # 引数
/// 
/// * `radians` - ラジアン角
/// 
/// # 戻り値
/// 
/// * 度数法の角度
pub fn radians_to_degrees(radians: f32) -> f32 {
    radians * 180.0 / PI
}

/// 角度を0〜2πの範囲に正規化
/// 
/// # 引数
/// 
/// * `angle` - 正規化する角度（ラジアン）
/// 
/// # 戻り値
/// 
/// * 0〜2πの範囲に正規化された角度
pub fn normalize_angle(angle: f32) -> f32 {
    let mut result = angle % TAU;
    if result < 0.0 {
        result += TAU;
    }
    result
}

/// 2点間の距離を計算
/// 
/// # 引数
/// 
/// * `p1` - 1つ目の点の座標 (x, y)
/// * `p2` - 2つ目の点の座標 (x, y)
/// 
/// # 戻り値
/// 
/// * 2点間のユークリッド距離
pub fn distance(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    (dx * dx + dy * dy).sqrt()
}

/// 2点間の距離の二乗を計算（距離の比較用に最適化）
/// 
/// # 引数
/// 
/// * `p1` - 1つ目の点の座標 (x, y)
/// * `p2` - 2つ目の点の座標 (x, y)
/// 
/// # 戻り値
/// 
/// * 2点間のユークリッド距離の二乗
pub fn distance_squared(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    dx * dx + dy * dy
}

/// ベクトルの長さを計算
/// 
/// # 引数
/// 
/// * `vec` - ベクトル (x, y)
/// 
/// # 戻り値
/// 
/// * ベクトルの長さ
pub fn vector_length(vec: (f32, f32)) -> f32 {
    (vec.0 * vec.0 + vec.1 * vec.1).sqrt()
}

/// ベクトルの正規化
/// 
/// # 引数
/// 
/// * `vec` - 正規化するベクトル (x, y)
/// 
/// # 戻り値
/// 
/// * 長さが1の正規化されたベクトル
pub fn normalize_vector(vec: (f32, f32)) -> (f32, f32) {
    let length = vector_length(vec);
    if length == 0.0 {
        (0.0, 0.0)
    } else {
        (vec.0 / length, vec.1 / length)
    }
}

/// ベクトルの内積
/// 
/// # 引数
/// 
/// * `v1` - 1つ目のベクトル (x, y)
/// * `v2` - 2つ目のベクトル (x, y)
/// 
/// # 戻り値
/// 
/// * 内積の値
pub fn dot_product(v1: (f32, f32), v2: (f32, f32)) -> f32 {
    v1.0 * v2.0 + v1.1 * v2.1
}

/// ベクトルの外積
/// 
/// # 引数
/// 
/// * `v1` - 1つ目のベクトル (x, y)
/// * `v2` - 2つ目のベクトル (x, y)
/// 
/// # 戻り値
/// 
/// * 外積の値
pub fn cross_product(v1: (f32, f32), v2: (f32, f32)) -> f32 {
    v1.0 * v2.1 - v1.1 * v2.0
}

/// 2つのベクトル間の角度を計算（ラジアン）
/// 
/// # 引数
/// 
/// * `v1` - 1つ目のベクトル (x, y)
/// * `v2` - 2つ目のベクトル (x, y)
/// 
/// # 戻り値
/// 
/// * 2つのベクトル間の角度（ラジアン）
pub fn angle_between_vectors(v1: (f32, f32), v2: (f32, f32)) -> f32 {
    let dot = dot_product(v1, v2);
    let len1 = vector_length(v1);
    let len2 = vector_length(v2);
    
    if len1 == 0.0 || len2 == 0.0 {
        0.0
    } else {
        (dot / (len1 * len2)).acos()
    }
}

/// 値を指定された範囲に制限
/// 
/// # 引数
/// 
/// * `value` - 制限する値
/// * `min` - 最小値
/// * `max` - 最大値
/// 
/// # 戻り値
/// 
/// * min〜maxの範囲に制限された値
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.clamp(min, max)
} 