//! 物理演算モジュール
//! 
//! このモジュールは、物理エンティティの動力学をシミュレートします。
//! 剛体シミュレーション、力とトルクの計算、衝突応答などを提供します。

use crate::physics::{PhysicsEntity, Collision};

/// 衝突解決器
pub struct CollisionResolver {
    /// 位置補正の割合（0-1）
    position_correction_rate: f64,
    /// イテレーション回数
    iterations: usize,
}

impl CollisionResolver {
    /// 新しい衝突解決器を作成
    pub fn new() -> Self {
        Self {
            position_correction_rate: 0.4, // デフォルト値
            iterations: 4, // デフォルト値
        }
    }
    
    /// パラメータをカスタマイズした衝突解決器を作成
    pub fn with_params(position_correction_rate: f64, iterations: usize) -> Self {
        Self {
            position_correction_rate,
            iterations,
        }
    }
    
    /// 位置補正の割合を設定
    pub fn set_position_correction_rate(&mut self, rate: f64) {
        self.position_correction_rate = rate.clamp(0.0, 1.0);
    }
    
    /// イテレーション回数を設定
    pub fn set_iterations(&mut self, iterations: usize) {
        self.iterations = iterations;
    }
    
    /// 衝突を解決
    pub fn resolve_collision(
        &self,
        entity_a: &mut PhysicsEntity,
        entity_b: &mut PhysicsEntity,
        collision: &Collision,
    ) {
        // 静的物体の場合は処理を分岐
        if entity_a.is_static && entity_b.is_static {
            // 両方静的なら何もしない
            return;
        } else if entity_a.is_static {
            // Aが静的ならBのみ処理
            self.resolve_collision_one_sided(entity_b, entity_a, collision, true);
            return;
        } else if entity_b.is_static {
            // Bが静的ならAのみ処理
            self.resolve_collision_one_sided(entity_a, entity_b, collision, false);
            return;
        }
        
        // 両方動的な場合
        
        // 相対速度を計算
        let relative_velocity = (
            entity_b.velocity.0 - entity_a.velocity.0,
            entity_b.velocity.1 - entity_a.velocity.1,
        );
        
        // 法線方向の相対速度
        let velocity_along_normal = 
            relative_velocity.0 * collision.normal.0 + 
            relative_velocity.1 * collision.normal.1;
        
        // 既に離れる方向に動いている場合は処理しない
        if velocity_along_normal > 0.0 {
            return;
        }
        
        // 反発係数（2つの物体の反発係数の平均）
        let restitution = (entity_a.restitution + entity_b.restitution) * 0.5;
        
        // 質量に基づく衝撃スケール
        let inverse_mass_a = if entity_a.is_static { 0.0 } else { 1.0 / entity_a.mass };
        let inverse_mass_b = if entity_b.is_static { 0.0 } else { 1.0 / entity_b.mass };
        let inverse_mass_sum = inverse_mass_a + inverse_mass_b;
        
        if inverse_mass_sum <= 0.0 {
            return;
        }
        
        // 衝撃の大きさ
        let j = -(1.0 + restitution) * velocity_along_normal / inverse_mass_sum;
        
        // 衝撃を適用
        let impulse = (
            collision.normal.0 * j,
            collision.normal.1 * j,
        );
        
        // 速度を更新
        entity_a.velocity = (
            entity_a.velocity.0 - impulse.0 * inverse_mass_a,
            entity_a.velocity.1 - impulse.1 * inverse_mass_a,
        );
        
        entity_b.velocity = (
            entity_b.velocity.0 + impulse.0 * inverse_mass_b,
            entity_b.velocity.1 + impulse.1 * inverse_mass_b,
        );
        
        // 摩擦力の適用
        self.apply_friction(
            entity_a, entity_b, 
            collision, 
            inverse_mass_a, inverse_mass_b, 
            j
        );
        
        // 位置補正（めり込み解消）
        self.correct_position(entity_a, entity_b, collision);
    }
    
    /// 片方のエンティティのみに衝突応答を適用（静的物体との衝突用）
    fn resolve_collision_one_sided(
        &self,
        dynamic_entity: &mut PhysicsEntity,
        static_entity: &PhysicsEntity,
        collision: &Collision,
        is_flipped: bool,
    ) {
        // 相対速度を計算
        let normal = if is_flipped {
            // 法線の向きを反転
            (-collision.normal.0, -collision.normal.1)
        } else {
            collision.normal
        };
        
        // 法線方向の速度
        let velocity_along_normal = 
            dynamic_entity.velocity.0 * normal.0 + 
            dynamic_entity.velocity.1 * normal.1;
        
        // 既に離れる方向に動いている場合は処理しない
        if velocity_along_normal > 0.0 {
            return;
        }
        
        // 反発係数
        let restitution = static_entity.restitution * dynamic_entity.restitution;
        
        // 衝撃の計算
        let j = dynamic_entity.mass * -(1.0 + restitution) * velocity_along_normal;
        
        // 衝撃を適用
        let impulse = (normal.0 * j, normal.1 * j);
        
        // 速度を更新
        dynamic_entity.velocity = (
            dynamic_entity.velocity.0 + impulse.0 / dynamic_entity.mass,
            dynamic_entity.velocity.1 + impulse.1 / dynamic_entity.mass,
        );
        
        // 摩擦の適用（簡易版）
        let friction = (static_entity.friction + dynamic_entity.friction) * 0.5;
        
        // 接線方向を計算
        let tangent = (-normal.1, normal.0);
        
        // 接線方向の速度
        let velocity_along_tangent = 
            dynamic_entity.velocity.0 * tangent.0 + 
            dynamic_entity.velocity.1 * tangent.1;
        
        // 摩擦力を計算
        let friction_impulse = -friction * velocity_along_tangent * dynamic_entity.mass;
        
        // 摩擦力を適用
        dynamic_entity.velocity = (
            dynamic_entity.velocity.0 + tangent.0 * friction_impulse / dynamic_entity.mass,
            dynamic_entity.velocity.1 + tangent.1 * friction_impulse / dynamic_entity.mass,
        );
        
        // 位置補正（めり込み解消）
        let correction = (
            normal.0 * collision.penetration * self.position_correction_rate,
            normal.1 * collision.penetration * self.position_correction_rate,
        );
        
        dynamic_entity.position = (
            dynamic_entity.position.0 + correction.0,
            dynamic_entity.position.1 + correction.1,
        );
    }
    
    /// 摩擦力を適用
    fn apply_friction(
        &self,
        entity_a: &mut PhysicsEntity,
        entity_b: &mut PhysicsEntity,
        collision: &Collision,
        inverse_mass_a: f64,
        inverse_mass_b: f64,
        normal_impulse: f64,
    ) {
        // 相対速度を計算
        let relative_velocity = (
            entity_b.velocity.0 - entity_a.velocity.0,
            entity_b.velocity.1 - entity_a.velocity.1,
        );
        
        // 接線方向を計算
        let tangent = (-collision.normal.1, collision.normal.0);
        
        // 接線方向の相対速度
        let velocity_along_tangent = 
            relative_velocity.0 * tangent.0 + 
            relative_velocity.1 * tangent.1;
        
        // 摩擦係数（2つの物体の摩擦係数の平均）
        let friction = (entity_a.friction + entity_b.friction) * 0.5;
        
        // 摩擦力の大きさ（クーロン摩擦）
        let friction_impulse = velocity_along_tangent * friction;
        
        // 摩擦力は法線力に比例（摩擦係数倍）
        let max_friction = normal_impulse * friction;
        
        // 最大摩擦力でクランプ
        let clamped_friction = friction_impulse.clamp(-max_friction, max_friction);
        
        // 摩擦力の適用
        let impulse = (
            tangent.0 * -clamped_friction,
            tangent.1 * -clamped_friction,
        );
        
        // 速度を更新
        entity_a.velocity = (
            entity_a.velocity.0 - impulse.0 * inverse_mass_a,
            entity_a.velocity.1 - impulse.1 * inverse_mass_a,
        );
        
        entity_b.velocity = (
            entity_b.velocity.0 + impulse.0 * inverse_mass_b,
            entity_b.velocity.1 + impulse.1 * inverse_mass_b,
        );
    }
    
    /// 位置補正（めり込み解消）
    fn correct_position(
        &self,
        entity_a: &mut PhysicsEntity,
        entity_b: &mut PhysicsEntity,
        collision: &Collision,
    ) {
        let inverse_mass_a = if entity_a.is_static { 0.0 } else { 1.0 / entity_a.mass };
        let inverse_mass_b = if entity_b.is_static { 0.0 } else { 1.0 / entity_b.mass };
        let inverse_mass_sum = inverse_mass_a + inverse_mass_b;
        
        if inverse_mass_sum <= 0.0 {
            return;
        }
        
        // スラック変数（小さな貫通は許容）
        const PENETRATION_SLACK: f64 = 0.01;
        
        let correction_magnitude = (collision.penetration - PENETRATION_SLACK)
            .max(0.0) * self.position_correction_rate / inverse_mass_sum;
        
        let correction_a = (
            -collision.normal.0 * correction_magnitude * inverse_mass_a,
            -collision.normal.1 * correction_magnitude * inverse_mass_a,
        );
        
        let correction_b = (
            collision.normal.0 * correction_magnitude * inverse_mass_b,
            collision.normal.1 * correction_magnitude * inverse_mass_b,
        );
        
        entity_a.position = (
            entity_a.position.0 + correction_a.0,
            entity_a.position.1 + correction_a.1,
        );
        
        entity_b.position = (
            entity_b.position.0 + correction_b.0,
            entity_b.position.1 + correction_b.1,
        );
    }
}

/// 積分器（運動方程式の数値積分）
pub struct Integrator {
    /// 最大速度
    max_velocity: f64,
    /// 最大角速度
    max_angular_velocity: f64,
}

impl Integrator {
    /// 新しい積分器を作成
    pub fn new() -> Self {
        Self {
            max_velocity: 1000.0,
            max_angular_velocity: 20.0,
        }
    }
    
    /// パラメータをカスタマイズした積分器を作成
    pub fn with_params(max_velocity: f64, max_angular_velocity: f64) -> Self {
        Self {
            max_velocity,
            max_angular_velocity,
        }
    }
    
    /// 最大速度を設定
    pub fn set_max_velocity(&mut self, max_velocity: f64) {
        self.max_velocity = max_velocity;
    }
    
    /// 最大角速度を設定
    pub fn set_max_angular_velocity(&mut self, max_angular_velocity: f64) {
        self.max_angular_velocity = max_angular_velocity;
    }
    
    /// 物理エンティティを更新（オイラー法）
    pub fn integrate(&self, entity: &mut PhysicsEntity, dt: f64, gravity: (f64, f64), damping: f64) {
        if entity.is_static {
            // 静的物体は更新しない
            return;
        }
        
        // 加速度を更新（重力を適用）
        entity.acceleration = (
            entity.acceleration.0 + gravity.0,
            entity.acceleration.1 + gravity.1,
        );
        
        // 速度を更新（オイラー法）
        entity.velocity = (
            entity.velocity.0 + entity.acceleration.0 * dt,
            entity.velocity.1 + entity.acceleration.1 * dt,
        );
        
        // 減衰を適用
        entity.velocity = (
            entity.velocity.0 * damping,
            entity.velocity.1 * damping,
        );
        
        // 速度の大きさをチェック
        let speed_squared = entity.velocity.0 * entity.velocity.0 + entity.velocity.1 * entity.velocity.1;
        if speed_squared > self.max_velocity * self.max_velocity {
            let speed = speed_squared.sqrt();
            entity.velocity = (
                entity.velocity.0 * self.max_velocity / speed,
                entity.velocity.1 * self.max_velocity / speed,
            );
        }
        
        // 位置を更新
        entity.position = (
            entity.position.0 + entity.velocity.0 * dt,
            entity.position.1 + entity.velocity.1 * dt,
        );
        
        // 角速度を制限
        entity.angular_velocity = entity.angular_velocity.clamp(
            -self.max_angular_velocity,
            self.max_angular_velocity,
        );
        
        // 回転を更新
        entity.rotation += entity.angular_velocity * dt;
        
        // 角度を正規化（0〜2π）
        while entity.rotation < 0.0 {
            entity.rotation += std::f64::consts::PI * 2.0;
        }
        while entity.rotation >= std::f64::consts::PI * 2.0 {
            entity.rotation -= std::f64::consts::PI * 2.0;
        }
        
        // 加速度をリセット（力の蓄積をクリア）
        entity.acceleration = (0.0, 0.0);
    }
}

/// 力の生成器
pub struct ForceGenerator {
    /// 重力定数
    gravity: (f64, f64),
}

impl ForceGenerator {
    /// 新しい力の生成器を作成
    pub fn new(gravity: (f64, f64)) -> Self {
        Self {
            gravity,
        }
    }
    
    /// 重力を設定
    pub fn set_gravity(&mut self, gravity: (f64, f64)) {
        self.gravity = gravity;
    }
    
    /// 重力を適用
    pub fn apply_gravity(&self, entity: &mut PhysicsEntity) {
        if entity.is_static {
            return;
        }
        
        let force = (
            self.gravity.0 * entity.mass,
            self.gravity.1 * entity.mass,
        );
        
        self.apply_force(entity, force);
    }
    
    /// 力を適用
    pub fn apply_force(&self, entity: &mut PhysicsEntity, force: (f64, f64)) {
        if entity.is_static {
            return;
        }
        
        // F = ma より a = F/m
        let acceleration = (
            force.0 / entity.mass,
            force.1 / entity.mass,
        );
        
        entity.acceleration = (
            entity.acceleration.0 + acceleration.0,
            entity.acceleration.1 + acceleration.1,
        );
    }
    
    /// トルクを適用
    pub fn apply_torque(&self, entity: &mut PhysicsEntity, torque: f64) {
        if entity.is_static {
            return;
        }
        
        // 慣性モーメントは質量に比例すると仮定
        let inertia = entity.mass; // 簡易的な計算
        
        entity.angular_velocity += torque / inertia;
    }
    
    /// 特定の位置に力を適用（トルクも生成）
    pub fn apply_force_at_point(
        &self,
        entity: &mut PhysicsEntity,
        force: (f64, f64),
        application_point: (f64, f64),
    ) {
        if entity.is_static {
            return;
        }
        
        // 力を適用
        self.apply_force(entity, force);
        
        // アームベクトル（物体の中心から力の作用点へのベクトル）
        let arm = (
            application_point.0 - entity.position.0,
            application_point.1 - entity.position.1,
        );
        
        // トルク = arm × force（2D外積）
        let torque = arm.0 * force.1 - arm.1 * force.0;
        
        self.apply_torque(entity, torque);
    }
    
    /// ばね力を適用
    pub fn apply_spring_force(
        &self,
        entity: &mut PhysicsEntity,
        anchor_point: (f64, f64),
        rest_length: f64,
        spring_constant: f64,
    ) {
        if entity.is_static {
            return;
        }
        
        // ばねのベクトル（アンカーから物体へ）
        let spring_vector = (
            entity.position.0 - anchor_point.0,
            entity.position.1 - anchor_point.1,
        );
        
        // ばねの現在の長さ
        let length = (spring_vector.0 * spring_vector.0 + spring_vector.1 * spring_vector.1).sqrt();
        
        if length <= 0.0001 {
            return; // 長さがゼロに近い場合は計算しない
        }
        
        // ばねの方向単位ベクトル
        let direction = (
            spring_vector.0 / length,
            spring_vector.1 / length,
        );
        
        // フックの法則: F = -k * (x - x0)
        // kはばね定数、xは現在の長さ、x0は自然長
        let force_magnitude = -spring_constant * (length - rest_length);
        
        // 力ベクトル
        let force = (
            direction.0 * force_magnitude,
            direction.1 * force_magnitude,
        );
        
        self.apply_force(entity, force);
    }
    
    /// 抗力（空気抵抗など）を適用
    pub fn apply_drag_force(
        &self,
        entity: &mut PhysicsEntity,
        drag_coefficient: f64,
    ) {
        if entity.is_static {
            return;
        }
        
        // 速度の二乗に比例する抗力
        let speed_squared = entity.velocity.0 * entity.velocity.0 + entity.velocity.1 * entity.velocity.1;
        
        if speed_squared < 0.0001 {
            return; // 速度がほぼゼロなら無視
        }
        
        let speed = speed_squared.sqrt();
        
        // 速度の反対方向の単位ベクトル
        let direction = (
            -entity.velocity.0 / speed,
            -entity.velocity.1 / speed,
        );
        
        // 抗力の大きさ
        let force_magnitude = drag_coefficient * speed_squared;
        
        // 抗力ベクトル
        let force = (
            direction.0 * force_magnitude,
            direction.1 * force_magnitude,
        );
        
        self.apply_force(entity, force);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::collision::CollisionShape;
    
    #[test]
    fn test_integrator() {
        let mut entity = PhysicsEntity {
            entity_id: 1,
            position: (0.0, 0.0),
            velocity: (10.0, 5.0),
            acceleration: (0.0, 0.0),
            mass: 1.0,
            rotation: 0.0,
            angular_velocity: 0.1,
            shape: CollisionShape::Circle { radius: 10.0 },
            restitution: 0.5,
            friction: 0.3,
            is_static: false,
        };
        
        let integrator = Integrator::new();
        let gravity = (0.0, 9.8);
        let damping = 0.99;
        let dt = 0.016; // 16ms
        
        integrator.integrate(&mut entity, dt, gravity, damping);
        
        // 重力による加速
        assert!(entity.velocity.1 > 5.0);
        // 位置が更新されたことを確認
        assert!(entity.position.0 > 0.0);
        assert!(entity.position.1 > 0.0);
        // 回転が更新されたことを確認
        assert!(entity.rotation > 0.0);
    }
    
    #[test]
    fn test_force_generator() {
        let mut entity = PhysicsEntity {
            entity_id: 1,
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            mass: 2.0,
            rotation: 0.0,
            angular_velocity: 0.0,
            shape: CollisionShape::Circle { radius: 10.0 },
            restitution: 0.5,
            friction: 0.3,
            is_static: false,
        };
        
        let gravity = (0.0, 9.8);
        let force_gen = ForceGenerator::new(gravity);
        
        // 力を適用
        force_gen.apply_force(&mut entity, (10.0, 5.0));
        
        // 加速度 = 力 / 質量
        assert_eq!(entity.acceleration.0, 5.0); // 10.0 / 2.0
        assert_eq!(entity.acceleration.1, 2.5); // 5.0 / 2.0
        
        // トルクを適用
        force_gen.apply_torque(&mut entity, 6.0);
        
        // 角加速度の確認
        assert_eq!(entity.angular_velocity, 3.0); // 6.0 / 2.0
        
        // 重力を適用
        entity.acceleration = (0.0, 0.0); // リセット
        force_gen.apply_gravity(&mut entity);
        
        // 重力加速度の確認
        assert_eq!(entity.acceleration.0, 0.0);
        assert_eq!(entity.acceleration.1, 9.8);
    }
    
    #[test]
    fn test_collision_resolver() {
        let mut entity_a = PhysicsEntity {
            entity_id: 1,
            position: (0.0, 0.0),
            velocity: (5.0, 0.0),
            acceleration: (0.0, 0.0),
            mass: 1.0,
            rotation: 0.0,
            angular_velocity: 0.0,
            shape: CollisionShape::Circle { radius: 10.0 },
            restitution: 0.5,
            friction: 0.3,
            is_static: false,
        };
        
        let mut entity_b = PhysicsEntity {
            entity_id: 2,
            position: (15.0, 0.0),
            velocity: (-5.0, 0.0),
            acceleration: (0.0, 0.0),
            mass: 1.0,
            rotation: 0.0,
            angular_velocity: 0.0,
            shape: CollisionShape::Circle { radius: 10.0 },
            restitution: 0.5,
            friction: 0.3,
            is_static: false,
        };
        
        let collision = Collision {
            position: (7.5, 0.0),
            normal: (1.0, 0.0),
            penetration: 5.0,
        };
        
        let resolver = CollisionResolver::new();
        resolver.resolve_collision(&mut entity_a, &mut entity_b, &collision);
        
        // 衝突後、速度の向きが反転
        assert!(entity_a.velocity.0 < 0.0);
        assert!(entity_b.velocity.0 > 0.0);
        
        // 位置補正により衝突が解消される
        assert!(entity_a.position.0 < 0.0);
        assert!(entity_b.position.0 > 15.0);
    }
} 