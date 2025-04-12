//! 物理システムモジュール
//! 
//! このモジュールは、ゲーム内の物理シミュレーションを担当します。
//! 衝突検出、剛体シミュレーション、物理エンティティの管理などの機能を提供します。

use std::collections::HashMap;

use crate::ecs::{World, Resource};

pub mod collision;
pub mod dynamics;
pub mod optimization;

pub use collision::{detect_collision, Collision, CollisionShape};
pub use dynamics::{apply_force, apply_gravity, apply_impulse, apply_torque, integrate, resolve_collision};
pub use optimization::{CollisionFilter, PhysicsStep, SpatialGrid, generate_collision_pairs};

/// 物理システムを初期化
pub fn init_physics_system(world: &mut World) {
    // 物理ワールドを作成してリソースとして登録
    let physics_world = PhysicsWorld::new();
    world.insert_resource(physics_world);
    
    // TODO: 必要に応じて物理システムを登録
    // world.register_system(PhysicsSystem::new());
}

/// 物理エンティティ
pub struct PhysicsEntity {
    /// エンティティID
    pub entity_id: u32,
    /// 位置 (x, y)
    pub position: (f64, f64),
    /// 速度 (vx, vy)
    pub velocity: (f64, f64),
    /// 加速度 (ax, ay)
    pub acceleration: (f64, f64),
    /// 質量
    pub mass: f64,
    /// 回転角（ラジアン）
    pub rotation: f64,
    /// 角速度
    pub angular_velocity: f64,
    /// 衝突形状
    pub shape: CollisionShape,
    /// 反発係数
    pub restitution: f64,
    /// 摩擦係数
    pub friction: f64,
    /// 静的オブジェクトかどうか
    pub is_static: bool,
}

impl PhysicsEntity {
    /// 新しい物理エンティティを作成
    pub fn new(entity_id: u32, position: (f64, f64), shape: CollisionShape) -> Self {
        Self {
            entity_id,
            position,
            velocity: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            mass: 1.0,
            rotation: 0.0,
            angular_velocity: 0.0,
            shape,
            restitution: 0.5,
            friction: 0.3,
            is_static: false,
        }
    }

    /// 静的オブジェクトとして設定
    pub fn set_static(&mut self, is_static: bool) -> &mut Self {
        self.is_static = is_static;
        
        // 静的オブジェクトの場合、速度と加速度をゼロにする
        if is_static {
            self.velocity = (0.0, 0.0);
            self.acceleration = (0.0, 0.0);
            self.angular_velocity = 0.0;
        }
        
        self
    }

    /// 質量を設定
    pub fn set_mass(&mut self, mass: f64) -> &mut Self {
        if mass > 0.0 {
            self.mass = mass;
        }
        self
    }

    /// 反発係数を設定
    pub fn set_restitution(&mut self, restitution: f64) -> &mut Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }

    /// 摩擦係数を設定
    pub fn set_friction(&mut self, friction: f64) -> &mut Self {
        self.friction = friction.max(0.0);
        self
    }

    /// 力を適用
    pub fn apply_force(&mut self, force: (f64, f64)) {
        if !self.is_static {
            dynamics::apply_force(self, force);
        }
    }

    /// 速度を設定
    pub fn set_velocity(&mut self, velocity: (f64, f64)) -> &mut Self {
        if !self.is_static {
            self.velocity = velocity;
        }
        self
    }

    /// エンティティのAABB（軸並行境界ボックス）を取得
    /// 
    /// # 戻り値
    /// 
    /// (min_x, min_y, max_x, max_y)
    pub fn get_aabb(&self) -> (f64, f64, f64, f64) {
        match &self.shape {
            CollisionShape::Circle { radius } => {
                (
                    self.position.0 - radius,
                    self.position.1 - radius,
                    self.position.0 + radius,
                    self.position.1 + radius,
                )
            },
            CollisionShape::AABB { width, height } => {
                let half_width = width / 2.0;
                let half_height = height / 2.0;
                (
                    self.position.0 - half_width,
                    self.position.1 - half_height,
                    self.position.0 + half_width,
                    self.position.1 + half_height,
                )
            },
            CollisionShape::Polygon { vertices } => {
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;
                
                // 全ての頂点をチェックして境界を計算
                for &(x, y) in vertices {
                    // 回転を適用
                    let cos_rot = self.rotation.cos();
                    let sin_rot = self.rotation.sin();
                    let rotated_x = x * cos_rot - y * sin_rot;
                    let rotated_y = x * sin_rot + y * cos_rot;
                    
                    // ワールド座標に変換
                    let world_x = self.position.0 + rotated_x;
                    let world_y = self.position.1 + rotated_y;
                    
                    // 境界を更新
                    min_x = min_x.min(world_x);
                    min_y = min_y.min(world_y);
                    max_x = max_x.max(world_x);
                    max_y = max_y.max(world_y);
                }
                
                (min_x, min_y, max_x, max_y)
            },
        }
    }
}

/// 物理ワールド
pub struct PhysicsWorld {
    /// 重力
    gravity: (f64, f64),
    /// 時間ステップ
    time_step: f64,
    /// 減衰係数
    damping: f64,
    /// 物理エンティティのマップ（キー: エンティティID, 値: エンティティ）
    entities: HashMap<u32, PhysicsEntity>,
    /// 空間分割グリッド
    spatial_grid: optimization::SpatialGrid,
    /// 衝突フィルター
    collision_filter: optimization::CollisionFilter,
    /// 物理ステップ
    physics_step: optimization::PhysicsStep,
}

impl PhysicsWorld {
    /// 新しい物理ワールドを作成
    pub fn new() -> Self {
        Self {
            gravity: (0.0, 9.8), // デフォルトの重力は下向き
            time_step: 1.0 / 60.0, // 60 FPS
            damping: 0.01,
            entities: HashMap::new(),
            spatial_grid: optimization::SpatialGrid::new(100.0),
            collision_filter: optimization::CollisionFilter::new(),
            physics_step: optimization::PhysicsStep::new(1.0 / 60.0, 5),
        }
    }

    /// 重力を設定
    pub fn set_gravity(&mut self, gravity: (f64, f64)) -> &mut Self {
        self.gravity = gravity;
        self
    }

    /// 時間ステップを設定
    pub fn set_time_step(&mut self, time_step: f64) -> &mut Self {
        self.time_step = time_step;
        self.physics_step = optimization::PhysicsStep::new(time_step, 5);
        self
    }

    /// 減衰係数を設定
    pub fn set_damping(&mut self, damping: f64) -> &mut Self {
        self.damping = damping.clamp(0.0, 1.0);
        self
    }

    /// エンティティを追加
    pub fn add_entity(&mut self, entity: PhysicsEntity) -> &mut Self {
        self.spatial_grid.add_entity(&entity);
        self.entities.insert(entity.entity_id, entity);
        self
    }

    /// エンティティを取得
    pub fn get_entity(&self, entity_id: u32) -> Option<&PhysicsEntity> {
        self.entities.get(&entity_id)
    }

    /// エンティティを可変で取得
    pub fn get_entity_mut(&mut self, entity_id: u32) -> Option<&mut PhysicsEntity> {
        self.entities.get_mut(&entity_id)
    }

    /// エンティティを削除
    pub fn remove_entity(&mut self, entity_id: u32) -> Option<PhysicsEntity> {
        self.entities.remove(&entity_id)
    }

    /// 衝突カテゴリを設定
    pub fn set_entity_category(&mut self, entity_id: u32, category: u32) -> &mut Self {
        self.collision_filter.set_entity_category(entity_id, category);
        self
    }

    /// 衝突マスクを設定
    pub fn set_entity_mask(&mut self, entity_id: u32, mask: u32) -> &mut Self {
        self.collision_filter.set_entity_mask(entity_id, mask);
        self
    }

    /// 物理シミュレーションを更新
    pub fn update(&mut self, delta_time: f64) {
        // 空間分割グリッドをクリア
        self.spatial_grid.clear();
        
        // エンティティを空間グリッドに追加
        for entity in self.entities.values() {
            self.spatial_grid.add_entity(entity);
        }
        
        // 物理ステップを更新
        let steps = self.physics_step.update(delta_time);
        
        for step in steps {
            // 衝突ペアを生成
            let entities_vec: Vec<PhysicsEntity> = self.entities.values().cloned().collect();
            let collision_pairs = optimization::generate_collision_pairs(&entities_vec, &self.spatial_grid, &self.collision_filter);
            
            // 衝突解決
            for pair in collision_pairs {
                if let (Some(entity_a), Some(entity_b)) = (self.entities.get(&pair.entity_a), self.entities.get(&pair.entity_b)) {
                    if let Some(collision) = collision::detect_collision(entity_a, entity_b) {
                        // 衝突情報をコピー
                        let collision_info = collision.clone();
                        
                        // エンティティを可変で取得
                        if let (Some(entity_a_mut), Some(entity_b_mut)) = (self.entities.get_mut(&pair.entity_a), self.entities.get_mut(&pair.entity_b)) {
                            // 衝突を解決
                            dynamics::resolve_collision(entity_a_mut, entity_b_mut, &collision_info);
                        }
                    }
                }
            }
            
            // 各エンティティを更新
            for entity in self.entities.values_mut() {
                if !entity.is_static {
                    // 重力を適用
                    dynamics::apply_gravity(entity, self.gravity);
                    
                    // 減衰を適用
                    dynamics::apply_damping(entity, self.damping);
                    
                    // 運動を積分
                    dynamics::integrate(entity, step);
                }
            }
        }
    }

    /// 2つのエンティティ間の衝突を検出
    pub fn check_collision(&self, entity_id_a: u32, entity_id_b: u32) -> Option<Collision> {
        if let (Some(entity_a), Some(entity_b)) = (self.entities.get(&entity_id_a), self.entities.get(&entity_id_b)) {
            collision::detect_collision(entity_a, entity_b)
        } else {
            None
        }
    }

    /// エンティティ数を取得
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// すべてのエンティティIDを取得
    pub fn get_all_entity_ids(&self) -> Vec<u32> {
        self.entities.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_creation() {
        let world = PhysicsWorld::new();
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_add_entity() {
        let mut world = PhysicsWorld::new();
        
        let entity = PhysicsEntity::new(
            1,
            (100.0, 100.0),
            CollisionShape::Circle { radius: 20.0 }
        );
        
        world.add_entity(entity);
        assert_eq!(world.entity_count(), 1);
        
        let retrieved_entity = world.get_entity(1);
        assert!(retrieved_entity.is_some());
        assert_eq!(retrieved_entity.unwrap().position, (100.0, 100.0));
    }

    #[test]
    fn test_gravity() {
        let mut world = PhysicsWorld::new();
        world.set_gravity((0.0, -9.8));
        
        let entity = PhysicsEntity::new(
            1,
            (100.0, 100.0),
            CollisionShape::Circle { radius: 20.0 }
        );
        
        world.add_entity(entity);
        
        // 1秒シミュレーション
        world.update(1.0);
        
        let entity = world.get_entity(1).unwrap();
        
        // 重力により速度が下向きに変化しているはず
        assert!(entity.velocity.1 < 0.0);
        
        // 位置も変化しているはず
        assert!(entity.position.1 < 100.0);
    }

    #[test]
    fn test_collision() {
        let mut world = PhysicsWorld::new();
        
        // 重力を無効化
        world.set_gravity((0.0, 0.0));
        
        // 2つの円形エンティティを作成（衝突する位置に配置）
        let entity1 = PhysicsEntity::new(
            1,
            (100.0, 100.0),
            CollisionShape::Circle { radius: 20.0 }
        );
        
        let mut entity2 = PhysicsEntity::new(
            2,
            (130.0, 100.0), // 距離が30なので、半径の合計（40）より小さい場合は衝突
            CollisionShape::Circle { radius: 20.0 }
        );
        
        // エンティティ2に初速を与える
        entity2.velocity = (-10.0, 0.0); // 左向きの速度
        
        world.add_entity(entity1);
        world.add_entity(entity2);
        
        // 衝突確認
        let collision = world.check_collision(1, 2);
        assert!(collision.is_some());
        
        // シミュレーション実行
        world.update(0.1);
        
        // 衝突により速度が変化するはず
        let entity1 = world.get_entity(1).unwrap();
        let entity2 = world.get_entity(2).unwrap();
        
        // エンティティ1は右向きの速度を持つようになる
        assert!(entity1.velocity.0 > 0.0);
        
        // エンティティ2は左向きの速度を持つが、絶対値は小さくなる
        assert!(entity2.velocity.0 < 0.0);
        
        // 位置も変化するはず
        assert!(entity1.position.0 < 100.0);
        assert!(entity2.position.0 > 130.0);
    }

    #[test]
    fn test_static_objects() {
        let mut world = PhysicsWorld::new();
        
        // 重力を設定
        world.set_gravity((0.0, 9.8));
        
        // 静的な地面を作成
        let mut ground = PhysicsEntity::new(
            1,
            (100.0, 200.0),
            CollisionShape::AABB {
                width: 200.0,
                height: 20.0,
            }
        );
        ground.set_static(true);
        
        // 落下するボールを作成
        let ball = PhysicsEntity::new(
            2,
            (100.0, 100.0),
            CollisionShape::Circle { radius: 10.0 }
        );
        
        world.add_entity(ground);
        world.add_entity(ball);
        
        // シミュレーション実行
        for _ in 0..10 {
            world.update(0.1);
        }
        
        // ボールは地面の上に止まるはず
        let ball = world.get_entity(2).unwrap();
        
        // 地面の位置（200）からボールの半径（10）を引いた位置の近くにいるはず
        assert!(ball.position.1 >= 180.0 && ball.position.1 <= 190.0);
        
        // 速度はほぼゼロになるはず（反発係数により少し跳ね返る可能性あり）
        assert!(ball.velocity.1.abs() < 1.0);
    }
} 