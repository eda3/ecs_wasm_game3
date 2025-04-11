//! 衝突検出モジュール
//! 
//! このモジュールは、様々な形状間の衝突検出アルゴリズムを提供します。
//! AABB（軸並行境界ボックス）、円形、多角形の衝突検出をサポートしています。

use crate::physics::PhysicsEntity;
use std::f64::consts::PI;

/// 衝突形状
#[derive(Clone, Debug)]
pub enum CollisionShape {
    /// 円形
    Circle {
        /// 半径
        radius: f64,
    },
    /// AABB（軸並行境界ボックス）
    AABB {
        /// 幅
        width: f64,
        /// 高さ
        height: f64,
    },
    /// 多角形
    Polygon {
        /// 頂点座標のリスト（ローカル座標）
        vertices: Vec<(f64, f64)>,
    },
}

/// 衝突情報
#[derive(Clone, Debug)]
pub struct Collision {
    /// 衝突点の位置
    pub position: (f64, f64),
    /// 衝突の法線ベクトル（衝突面の垂直方向）
    pub normal: (f64, f64),
    /// 貫通深度
    pub penetration: f64,
}

/// 二つの物理エンティティ間の衝突を検出
pub fn detect_collision(entity_a: &PhysicsEntity, entity_b: &PhysicsEntity) -> Option<Collision> {
    match (&entity_a.shape, &entity_b.shape) {
        (CollisionShape::Circle { radius: radius_a }, CollisionShape::Circle { radius: radius_b }) => {
            detect_circle_circle(
                entity_a.position,
                *radius_a,
                entity_b.position,
                *radius_b,
            )
        }
        (CollisionShape::AABB { width: width_a, height: height_a }, CollisionShape::AABB { width: width_b, height: height_b }) => {
            detect_aabb_aabb(
                entity_a.position,
                *width_a,
                *height_a,
                entity_b.position,
                *width_b,
                *height_b,
            )
        }
        (CollisionShape::Circle { radius }, CollisionShape::AABB { width, height }) => {
            detect_circle_aabb(
                entity_a.position,
                *radius,
                entity_b.position,
                *width,
                *height,
            )
        }
        (CollisionShape::AABB { width, height }, CollisionShape::Circle { radius }) => {
            // AABBと円の衝突検出を反転
            if let Some(collision) = detect_circle_aabb(
                entity_b.position,
                *radius,
                entity_a.position,
                *width,
                *height,
            ) {
                // 法線ベクトルを反転
                Some(Collision {
                    position: collision.position,
                    normal: (-collision.normal.0, -collision.normal.1),
                    penetration: collision.penetration,
                })
            } else {
                None
            }
        }
        (CollisionShape::Polygon { vertices: vertices_a }, CollisionShape::Polygon { vertices: vertices_b }) => {
            detect_polygon_polygon(
                entity_a.position,
                entity_a.rotation,
                vertices_a,
                entity_b.position,
                entity_b.rotation,
                vertices_b,
            )
        }
        (CollisionShape::Circle { radius }, CollisionShape::Polygon { vertices }) => {
            detect_circle_polygon(
                entity_a.position,
                *radius,
                entity_b.position,
                entity_b.rotation,
                vertices,
            )
        }
        (CollisionShape::Polygon { vertices }, CollisionShape::Circle { radius }) => {
            // 多角形と円の衝突検出を反転
            if let Some(collision) = detect_circle_polygon(
                entity_b.position,
                *radius,
                entity_a.position,
                entity_a.rotation,
                vertices,
            ) {
                // 法線ベクトルを反転
                Some(Collision {
                    position: collision.position,
                    normal: (-collision.normal.0, -collision.normal.1),
                    penetration: collision.penetration,
                })
            } else {
                None
            }
        }
        (CollisionShape::AABB { width, height }, CollisionShape::Polygon { vertices }) => {
            // AABBを多角形に変換
            let aabb_vertices = vec![
                (-width / 2.0, -height / 2.0),
                (width / 2.0, -height / 2.0),
                (width / 2.0, height / 2.0),
                (-width / 2.0, height / 2.0),
            ];
            
            detect_polygon_polygon(
                entity_a.position,
                entity_a.rotation,
                &aabb_vertices,
                entity_b.position,
                entity_b.rotation,
                vertices,
            )
        }
        (CollisionShape::Polygon { vertices }, CollisionShape::AABB { width, height }) => {
            // AABBを多角形に変換
            let aabb_vertices = vec![
                (-width / 2.0, -height / 2.0),
                (width / 2.0, -height / 2.0),
                (width / 2.0, height / 2.0),
                (-width / 2.0, height / 2.0),
            ];
            
            detect_polygon_polygon(
                entity_a.position,
                entity_a.rotation,
                vertices,
                entity_b.position,
                entity_b.rotation,
                &aabb_vertices,
            )
        }
    }
}

/// 円と円の衝突検出
fn detect_circle_circle(
    position_a: (f64, f64),
    radius_a: f64,
    position_b: (f64, f64),
    radius_b: f64,
) -> Option<Collision> {
    // 中心間の距離ベクトル
    let delta_x = position_b.0 - position_a.0;
    let delta_y = position_b.1 - position_a.1;
    
    // 中心間の距離の二乗
    let distance_squared = delta_x * delta_x + delta_y * delta_y;
    
    // 半径の合計
    let radius_sum = radius_a + radius_b;
    
    // 衝突判定
    if distance_squared >= radius_sum * radius_sum {
        return None;
    }
    
    // 実際の距離
    let distance = distance_squared.sqrt();
    
    // 貫通深度
    let penetration = radius_sum - distance;
    
    // 法線ベクトル（Aから見たB方向）
    let normal = if distance > 0.0001 {
        (delta_x / distance, delta_y / distance)
    } else {
        // 完全重なりの場合は上向きの法線を返す
        (0.0, 1.0)
    };
    
    // 衝突点（二つの円の境界が接する点）
    let collision_point = (
        position_a.0 + normal.0 * radius_a,
        position_a.1 + normal.1 * radius_a,
    );
    
    Some(Collision {
        position: collision_point,
        normal,
        penetration,
    })
}

/// AABBとAABBの衝突検出
fn detect_aabb_aabb(
    position_a: (f64, f64),
    width_a: f64,
    height_a: f64,
    position_b: (f64, f64),
    width_b: f64,
    height_b: f64,
) -> Option<Collision> {
    // Aの境界
    let a_min_x = position_a.0 - width_a / 2.0;
    let a_max_x = position_a.0 + width_a / 2.0;
    let a_min_y = position_a.1 - height_a / 2.0;
    let a_max_y = position_a.1 + height_a / 2.0;
    
    // Bの境界
    let b_min_x = position_b.0 - width_b / 2.0;
    let b_max_x = position_b.0 + width_b / 2.0;
    let b_min_y = position_b.1 - height_b / 2.0;
    let b_max_y = position_b.1 + height_b / 2.0;
    
    // 衝突判定（分離軸判定の否定）
    if a_max_x < b_min_x || a_min_x > b_max_x || a_max_y < b_min_y || a_min_y > b_max_y {
        return None;
    }
    
    // 各軸方向の重なり量
    let overlap_x = if position_a.0 < position_b.0 {
        a_max_x - b_min_x
    } else {
        b_max_x - a_min_x
    };
    
    let overlap_y = if position_a.1 < position_b.1 {
        a_max_y - b_min_y
    } else {
        b_max_y - a_min_y
    };
    
    // 貫通が最小の軸を選ぶ
    let (normal, penetration) = if overlap_x < overlap_y {
        // X軸方向の貫通
        let sign = if position_a.0 < position_b.0 { 1.0 } else { -1.0 };
        ((sign, 0.0), overlap_x)
    } else {
        // Y軸方向の貫通
        let sign = if position_a.1 < position_b.1 { 1.0 } else { -1.0 };
        ((0.0, sign), overlap_y)
    };
    
    // 衝突点（貫通の中心）
    let collision_point = if overlap_x < overlap_y {
        // X軸衝突
        if position_a.0 < position_b.0 {
            (a_max_x, position_a.1)
        } else {
            (a_min_x, position_a.1)
        }
    } else {
        // Y軸衝突
        if position_a.1 < position_b.1 {
            (position_a.0, a_max_y)
        } else {
            (position_a.0, a_min_y)
        }
    };
    
    Some(Collision {
        position: collision_point,
        normal,
        penetration,
    })
}

/// 円とAABBの衝突検出
fn detect_circle_aabb(
    circle_pos: (f64, f64),
    radius: f64,
    aabb_pos: (f64, f64),
    width: f64,
    height: f64,
) -> Option<Collision> {
    // AABBの境界
    let aabb_min_x = aabb_pos.0 - width / 2.0;
    let aabb_max_x = aabb_pos.0 + width / 2.0;
    let aabb_min_y = aabb_pos.1 - height / 2.0;
    let aabb_max_y = aabb_pos.1 + height / 2.0;
    
    // 円の中心からAABBまでの最短距離を計算
    let closest_x = circle_pos.0.clamp(aabb_min_x, aabb_max_x);
    let closest_y = circle_pos.1.clamp(aabb_min_y, aabb_max_y);
    
    // 円の中心から最近点までの距離ベクトル
    let delta_x = circle_pos.0 - closest_x;
    let delta_y = circle_pos.1 - closest_y;
    
    // 距離の二乗
    let distance_squared = delta_x * delta_x + delta_y * delta_y;
    
    // 衝突判定
    if distance_squared > radius * radius {
        return None;
    }
    
    // 実際の距離
    let distance = distance_squared.sqrt();
    
    // 法線ベクトル（AABBから見た円方向）
    let normal = if distance > 0.0001 {
        // 円の中心が領域の外側
        (delta_x / distance, delta_y / distance)
    } else if closest_x == aabb_min_x {
        // 左端
        (-1.0, 0.0)
    } else if closest_x == aabb_max_x {
        // 右端
        (1.0, 0.0)
    } else if closest_y == aabb_min_y {
        // 上端
        (0.0, -1.0)
    } else if closest_y == aabb_max_y {
        // 下端
        (0.0, 1.0)
    } else {
        // 内部（異常値）
        let d_left = closest_x - aabb_min_x;
        let d_right = aabb_max_x - closest_x;
        let d_top = closest_y - aabb_min_y;
        let d_bottom = aabb_max_y - closest_y;
        
        // 最小距離の辺を選択
        let min_dist = d_left.min(d_right).min(d_top).min(d_bottom);
        
        if min_dist == d_left {
            (-1.0, 0.0)
        } else if min_dist == d_right {
            (1.0, 0.0)
        } else if min_dist == d_top {
            (0.0, -1.0)
        } else {
            (0.0, 1.0)
        }
    };
    
    // 貫通深度
    let penetration = radius - distance;
    
    // 衝突点
    let collision_point = (
        closest_x,
        closest_y,
    );
    
    Some(Collision {
        position: collision_point,
        normal: (-normal.0, -normal.1), // AABBから円への法線
        penetration,
    })
}

/// 座標を回転
fn rotate_point(point: (f64, f64), center: (f64, f64), angle: f64) -> (f64, f64) {
    let x = point.0 - center.0;
    let y = point.1 - center.1;
    
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    
    let rotated_x = x * cos_angle - y * sin_angle;
    let rotated_y = x * sin_angle + y * cos_angle;
    
    (rotated_x + center.0, rotated_y + center.1)
}

/// 多角形の法線を計算
fn get_polygon_normals(vertices: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut normals = Vec::with_capacity(vertices.len());
    
    for i in 0..vertices.len() {
        let j = (i + 1) % vertices.len();
        
        let edge_x = vertices[j].0 - vertices[i].0;
        let edge_y = vertices[j].1 - vertices[i].1;
        
        // 90度回転して法線を得る
        let length = (edge_x * edge_x + edge_y * edge_y).sqrt();
        if length > 0.0001 {
            normals.push((-edge_y / length, edge_x / length));
        }
    }
    
    normals
}

/// 多角形の世界座標の頂点を取得
fn get_world_vertices(
    position: (f64, f64),
    rotation: f64,
    vertices: &[(f64, f64)],
) -> Vec<(f64, f64)> {
    vertices
        .iter()
        .map(|&vertex| {
            // 回転
            let rotated = if rotation != 0.0 {
                rotate_point(vertex, (0.0, 0.0), rotation)
            } else {
                vertex
            };
            
            // 平行移動
            (rotated.0 + position.0, rotated.1 + position.1)
        })
        .collect()
}

/// 多角形を分離軸に射影
fn project_polygon(
    vertices: &[(f64, f64)],
    axis: (f64, f64),
) -> (f64, f64) {
    let mut min = f64::MAX;
    let mut max = f64::MIN;
    
    for vertex in vertices {
        // 点と軸の内積
        let projection = vertex.0 * axis.0 + vertex.1 * axis.1;
        
        min = min.min(projection);
        max = max.max(projection);
    }
    
    (min, max)
}

/// 円を分離軸に射影
fn project_circle(
    center: (f64, f64),
    radius: f64,
    axis: (f64, f64),
) -> (f64, f64) {
    // 中心の射影
    let projection = center.0 * axis.0 + center.1 * axis.1;
    
    // 半径分拡張
    (projection - radius, projection + radius)
}

/// 多角形と多角形の衝突検出
fn detect_polygon_polygon(
    position_a: (f64, f64),
    rotation_a: f64,
    vertices_a: &[(f64, f64)],
    position_b: (f64, f64),
    rotation_b: f64,
    vertices_b: &[(f64, f64)],
) -> Option<Collision> {
    if vertices_a.len() < 3 || vertices_b.len() < 3 {
        return None;
    }
    
    // 世界座標の頂点を取得
    let world_vertices_a = get_world_vertices(position_a, rotation_a, vertices_a);
    let world_vertices_b = get_world_vertices(position_b, rotation_b, vertices_b);
    
    // 法線ベクトルを取得
    let normals_a = get_polygon_normals(&world_vertices_a);
    let normals_b = get_polygon_normals(&world_vertices_b);
    
    // すべての法線を試す
    let all_normals = normals_a.iter().chain(normals_b.iter());
    
    let mut min_penetration = f64::MAX;
    let mut collision_normal = (0.0, 0.0);
    
    for &normal in all_normals {
        // 各多角形を法線に射影
        let projection_a = project_polygon(&world_vertices_a, normal);
        let projection_b = project_polygon(&world_vertices_b, normal);
        
        // 射影の重なりをチェック
        if projection_a.1 < projection_b.0 || projection_b.1 < projection_a.0 {
            // 重なりなし
            return None;
        }
        
        // 貫通深度
        let penetration = if projection_a.0 < projection_b.0 {
            projection_a.1 - projection_b.0
        } else {
            projection_b.1 - projection_a.0
        };
        
        // 最小の貫通を記録
        if penetration < min_penetration {
            min_penetration = penetration;
            
            // 法線の向きを確認
            let a_to_b_x = position_b.0 - position_a.0;
            let a_to_b_y = position_b.1 - position_a.1;
            let dot_product = a_to_b_x * normal.0 + a_to_b_y * normal.1;
            
            collision_normal = if dot_product < 0.0 {
                (-normal.0, -normal.1)
            } else {
                normal
            };
        }
    }
    
    // 衝突点（多角形の中心間を結ぶ線と貫通法線の交点）
    // 簡易的に、貫通が最小のポイントを取る
    let reference_normal = (-collision_normal.0, -collision_normal.1);
    
    let mut best_vertex = world_vertices_a[0];
    let mut best_dist = f64::MIN;
    
    for vertex in &world_vertices_a {
        let dist = vertex.0 * reference_normal.0 + vertex.1 * reference_normal.1;
        if dist > best_dist {
            best_dist = dist;
            best_vertex = *vertex;
        }
    }
    
    Some(Collision {
        position: best_vertex,
        normal: collision_normal,
        penetration: min_penetration,
    })
}

/// 円と多角形の衝突検出
fn detect_circle_polygon(
    circle_pos: (f64, f64),
    radius: f64,
    polygon_pos: (f64, f64),
    polygon_rot: f64,
    vertices: &[(f64, f64)],
) -> Option<Collision> {
    if vertices.len() < 3 {
        return None;
    }
    
    // 世界座標の頂点を取得
    let world_vertices = get_world_vertices(polygon_pos, polygon_rot, vertices);
    
    // 多角形の法線を取得
    let normals = get_polygon_normals(&world_vertices);
    
    // 円の中心から多角形の各頂点への方向ベクトル
    let vertex_to_circle_normals: Vec<(f64, f64)> = world_vertices
        .iter()
        .map(|&vertex| {
            let dx = circle_pos.0 - vertex.0;
            let dy = circle_pos.1 - vertex.1;
            let length = (dx * dx + dy * dy).sqrt();
            if length > 0.0001 {
                (dx / length, dy / length)
            } else {
                (0.0, 1.0)
            }
        })
        .collect();
    
    // すべての法線を試す
    let all_normals = normals.iter().chain(vertex_to_circle_normals.iter());
    
    let mut min_penetration = f64::MAX;
    let mut collision_normal = (0.0, 0.0);
    
    for &normal in all_normals {
        // 多角形と円を法線に射影
        let projection_polygon = project_polygon(&world_vertices, normal);
        let projection_circle = project_circle(circle_pos, radius, normal);
        
        // 射影の重なりをチェック
        if projection_polygon.1 < projection_circle.0 || projection_circle.1 < projection_polygon.0 {
            // 重なりなし
            return None;
        }
        
        // 貫通深度
        let penetration = if projection_polygon.0 < projection_circle.0 {
            projection_polygon.1 - projection_circle.0
        } else {
            projection_circle.1 - projection_polygon.0
        };
        
        // 最小の貫通を記録
        if penetration < min_penetration {
            min_penetration = penetration;
            
            // 法線の向きを確認
            let p_to_c_x = circle_pos.0 - polygon_pos.0;
            let p_to_c_y = circle_pos.1 - polygon_pos.1;
            let dot_product = p_to_c_x * normal.0 + p_to_c_y * normal.1;
            
            collision_normal = if dot_product < 0.0 {
                (-normal.0, -normal.1)
            } else {
                normal
            };
        }
    }
    
    // 最近接点を見つける
    let mut closest_point = world_vertices[0];
    let mut min_dist_squared = f64::MAX;
    
    for &vertex in &world_vertices {
        let dx = vertex.0 - circle_pos.0;
        let dy = vertex.1 - circle_pos.1;
        let dist_squared = dx * dx + dy * dy;
        
        if dist_squared < min_dist_squared {
            min_dist_squared = dist_squared;
            closest_point = vertex;
        }
    }
    
    // 衝突点（円から最も近い多角形の点）
    Some(Collision {
        position: closest_point,
        normal: collision_normal,
        penetration: min_penetration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_circle_circle_collision() {
        // 重なる二つの円
        let result = detect_circle_circle(
            (0.0, 0.0),
            10.0,
            (15.0, 0.0),
            10.0,
        );
        
        assert!(result.is_some());
        
        let collision = result.unwrap();
        assert_eq!(collision.normal, (1.0, 0.0));
        assert_eq!(collision.penetration, 5.0);
        
        // 重ならない二つの円
        let result = detect_circle_circle(
            (0.0, 0.0),
            10.0,
            (25.0, 0.0),
            10.0,
        );
        
        assert!(result.is_none());
    }
    
    #[test]
    fn test_aabb_aabb_collision() {
        // 重なる二つのAABB
        let result = detect_aabb_aabb(
            (0.0, 0.0),
            20.0,
            20.0,
            (15.0, 0.0),
            20.0,
            20.0,
        );
        
        assert!(result.is_some());
        
        let collision = result.unwrap();
        assert_eq!(collision.normal, (1.0, 0.0));
        assert_eq!(collision.penetration, 5.0);
        
        // 重ならない二つのAABB
        let result = detect_aabb_aabb(
            (0.0, 0.0),
            20.0,
            20.0,
            (25.0, 0.0),
            20.0,
            20.0,
        );
        
        assert!(result.is_none());
    }
    
    #[test]
    fn test_circle_aabb_collision() {
        // 重なる円とAABB
        let result = detect_circle_aabb(
            (0.0, 0.0),
            10.0,
            (15.0, 0.0),
            20.0,
            20.0,
        );
        
        assert!(result.is_some());
        
        let collision = result.unwrap();
        assert_eq!(collision.normal, (1.0, 0.0));
        assert_eq!(collision.penetration, 5.0);
        
        // 重ならない円とAABB
        let result = detect_circle_aabb(
            (0.0, 0.0),
            10.0,
            (25.0, 0.0),
            20.0,
            20.0,
        );
        
        assert!(result.is_none());
    }
    
    #[test]
    fn test_polygon_polygon_collision() {
        // 二つの正方形（多角形として表現）
        let square_a = vec![
            (-10.0, -10.0),
            (10.0, -10.0),
            (10.0, 10.0),
            (-10.0, 10.0),
        ];
        
        let square_b = vec![
            (-10.0, -10.0),
            (10.0, -10.0),
            (10.0, 10.0),
            (-10.0, 10.0),
        ];
        
        // 重なる二つの正方形
        let result = detect_polygon_polygon(
            (0.0, 0.0),
            0.0,
            &square_a,
            (15.0, 0.0),
            0.0,
            &square_b,
        );
        
        assert!(result.is_some());
        
        // 重ならない二つの正方形
        let result = detect_polygon_polygon(
            (0.0, 0.0),
            0.0,
            &square_a,
            (25.0, 0.0),
            0.0,
            &square_b,
        );
        
        assert!(result.is_none());
        
        // 回転させた正方形
        let result = detect_polygon_polygon(
            (0.0, 0.0),
            0.0,
            &square_a,
            (15.0, 0.0),
            PI / 4.0, // 45度回転
            &square_b,
        );
        
        assert!(result.is_some());
    }
    
    #[test]
    fn test_circle_polygon_collision() {
        // 正方形（多角形として表現）
        let square = vec![
            (-10.0, -10.0),
            (10.0, -10.0),
            (10.0, 10.0),
            (-10.0, 10.0),
        ];
        
        // 重なる円と正方形
        let result = detect_circle_polygon(
            (0.0, 0.0),
            15.0,
            (20.0, 0.0),
            0.0,
            &square,
        );
        
        assert!(result.is_some());
        
        // 重ならない円と正方形
        let result = detect_circle_polygon(
            (0.0, 0.0),
            10.0,
            (25.0, 0.0),
            0.0,
            &square,
        );
        
        assert!(result.is_none());
    }
} 