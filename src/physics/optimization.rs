//! 物理最適化モジュール
//! 
//! このモジュールは、物理シミュレーションの最適化機能を提供します。
//! 空間分割、衝突フィルタリング、物理ステップの制御などの機能を含みます。

use std::collections::{HashMap, HashSet};
use crate::physics::{PhysicsEntity, Collision, collision::detect_collision};

/// 空間分割グリッド
/// 
/// 2次元空間を均一なグリッドセルに分割し、エンティティの空間的な近接性を効率的に
/// 判断するためのデータ構造です。これにより、潜在的な衝突ペアの数を大幅に削減できます。
pub struct SpatialGrid {
    /// セルのサイズ
    cell_size: f64,
    /// グリッドのセル（キー: セルID, 値: エンティティIDのリスト）
    cells: HashMap<(i32, i32), Vec<u32>>,
    /// エンティティとそれが所属するセルの対応（キー: エンティティID, 値: セルIDのリスト）
    entity_cells: HashMap<u32, Vec<(i32, i32)>>,
}

impl SpatialGrid {
    /// 新しい空間分割グリッドを作成
    pub fn new(cell_size: f64) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            entity_cells: HashMap::new(),
        }
    }
    
    /// 位置から対応するグリッドセルのインデックスを計算
    pub fn position_to_cell(&self, position: (f64, f64)) -> (i32, i32) {
        let cell_x = (position.0 / self.cell_size).floor() as i32;
        let cell_y = (position.1 / self.cell_size).floor() as i32;
        (cell_x, cell_y)
    }
    
    /// エンティティの範囲（AABB）から占有するセルのリストを取得
    pub fn get_cells_for_aabb(
        &self, 
        min_x: f64, 
        min_y: f64, 
        max_x: f64, 
        max_y: f64
    ) -> Vec<(i32, i32)> {
        let min_cell = self.position_to_cell((min_x, min_y));
        let max_cell = self.position_to_cell((max_x, max_y));
        
        let mut cells = Vec::new();
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                cells.push((x, y));
            }
        }
        
        cells
    }
    
    /// グリッドをクリア
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_cells.clear();
    }
    
    /// エンティティをグリッドに追加
    pub fn insert_entity(&mut self, entity: &PhysicsEntity) {
        // エンティティのAABBを計算
        let (min_x, min_y, max_x, max_y) = entity.get_aabb();
        
        // エンティティが占有するセルを取得
        let cells = self.get_cells_for_aabb(min_x, min_y, max_x, max_y);
        
        // 各セルにエンティティを追加
        for cell in &cells {
            self.cells.entry(*cell)
                .or_insert_with(Vec::new)
                .push(entity.entity_id);
        }
        
        // エンティティとセルの対応を保存
        self.entity_cells.insert(entity.entity_id, cells);
    }
    
    /// エンティティをグリッドから削除
    pub fn remove_entity(&mut self, entity_id: u32) {
        if let Some(cells) = self.entity_cells.remove(&entity_id) {
            for cell in cells {
                if let Some(entities) = self.cells.get_mut(&cell) {
                    entities.retain(|id| *id != entity_id);
                }
            }
        }
    }
    
    /// エンティティの更新（位置が変更された場合）
    pub fn update_entity(&mut self, entity: &PhysicsEntity) {
        self.remove_entity(entity.entity_id);
        self.insert_entity(entity);
    }
    
    /// 指定したエンティティと潜在的に衝突する可能性のあるエンティティのIDを取得
    pub fn get_potential_collisions(&self, entity_id: u32) -> HashSet<u32> {
        let mut result = HashSet::new();
        
        if let Some(cells) = self.entity_cells.get(&entity_id) {
            for cell in cells {
                if let Some(entities) = self.cells.get(cell) {
                    for other_id in entities {
                        if *other_id != entity_id {
                            result.insert(*other_id);
                        }
                    }
                }
            }
        }
        
        result
    }
    
    /// すべての潜在的な衝突ペアを取得
    pub fn get_all_potential_pairs(&self) -> Vec<(u32, u32)> {
        let mut pairs = Vec::new();
        let mut processed = HashSet::new();
        
        for (_, entities) in &self.cells {
            for i in 0..entities.len() {
                let entity_a = entities[i];
                for j in (i+1)..entities.len() {
                    let entity_b = entities[j];
                    let pair_key = if entity_a < entity_b {
                        (entity_a, entity_b)
                    } else {
                        (entity_b, entity_a)
                    };
                    
                    if !processed.contains(&pair_key) {
                        pairs.push(pair_key);
                        processed.insert(pair_key);
                    }
                }
            }
        }
        
        pairs
    }
}

/// 衝突フィルタリング
/// 
/// エンティティ間の衝突を選択的に制御するためのフィルタリングシステムです。
/// カテゴリとマスクを使用して、特定のエンティティ間の衝突を有効/無効にできます。
pub struct CollisionFilter {
    /// 衝突フィルターのマスク（キー: エンティティID, 値: フィルターマスク）
    masks: HashMap<u32, u32>,
    /// 衝突フィルターのカテゴリ（キー: エンティティID, 値: カテゴリ）
    categories: HashMap<u32, u32>,
}

impl CollisionFilter {
    /// 新しい衝突フィルターを作成
    pub fn new() -> Self {
        Self {
            masks: HashMap::new(),
            categories: HashMap::new(),
        }
    }
    
    /// エンティティのカテゴリを設定
    /// 
    /// # 引数
    /// 
    /// * `entity_id` - エンティティID
    /// * `category` - カテゴリビットマスク
    pub fn set_category(&mut self, entity_id: u32, category: u32) {
        self.categories.insert(entity_id, category);
    }
    
    /// エンティティのマスクを設定
    /// 
    /// # 引数
    /// 
    /// * `entity_id` - エンティティID
    /// * `mask` - 衝突マスクビットマスク
    pub fn set_mask(&mut self, entity_id: u32, mask: u32) {
        self.masks.insert(entity_id, mask);
    }
    
    /// 2つのエンティティ間の衝突が可能かどうかを判定
    /// 
    /// # 引数
    /// 
    /// * `entity_a_id` - エンティティAのID
    /// * `entity_b_id` - エンティティBのID
    /// 
    /// # 戻り値
    /// 
    /// 衝突が可能な場合はtrue、不可能な場合はfalse
    pub fn should_collide(&self, entity_a_id: u32, entity_b_id: u32) -> bool {
        let category_a = self.categories.get(&entity_a_id).copied().unwrap_or(0xFFFFFFFF);
        let mask_a = self.masks.get(&entity_a_id).copied().unwrap_or(0xFFFFFFFF);
        
        let category_b = self.categories.get(&entity_b_id).copied().unwrap_or(0xFFFFFFFF);
        let mask_b = self.masks.get(&entity_b_id).copied().unwrap_or(0xFFFFFFFF);
        
        // 双方向のマスクチェック
        (category_a & mask_b) != 0 && (category_b & mask_a) != 0
    }
    
    /// エンティティを削除
    pub fn remove_entity(&mut self, entity_id: u32) {
        self.categories.remove(&entity_id);
        self.masks.remove(&entity_id);
    }
}

/// カテゴリ定数（例）
pub mod category {
    pub const PLAYER: u32 = 0x0001;
    pub const ENEMY: u32 = 0x0002;
    pub const PLATFORM: u32 = 0x0004;
    pub const PROJECTILE: u32 = 0x0008;
    pub const SENSOR: u32 = 0x0010;
    pub const TRIGGER: u32 = 0x0020;
    pub const ALL: u32 = 0xFFFFFFFF;
}

/// 物理ステップ最適化
/// 
/// 物理シミュレーションの更新頻度と精度を制御するためのシステムです。
/// 固定時間ステップを使用して安定したシミュレーションを実現します。
pub struct PhysicsStep {
    /// 累積時間
    accumulated_time: f64,
    /// 固定時間ステップ
    pub fixed_time_step: f64,
    /// 最大ステップ数（1フレームあたり）
    pub max_steps_per_update: usize,
}

impl PhysicsStep {
    /// 新しい物理ステップ制御を作成
    pub fn new(fixed_time_step: f64, max_steps_per_update: usize) -> Self {
        Self {
            accumulated_time: 0.0,
            fixed_time_step,
            max_steps_per_update,
        }
    }
    
    /// デフォルト設定で物理ステップ制御を作成
    pub fn default() -> Self {
        Self::new(1.0 / 60.0, 5)
    }
    
    /// 累積時間を更新し、実行すべき物理ステップの数を取得
    /// 
    /// # 引数
    /// 
    /// * `delta_time` - 前回のフレームからの経過時間
    /// 
    /// # 戻り値
    /// 
    /// (ステップ数, 残り時間の補間係数)
    pub fn update(&mut self, delta_time: f64) -> (usize, f64) {
        self.accumulated_time += delta_time;
        
        // 安全のため、累積時間を制限
        let max_accumulated = self.fixed_time_step * self.max_steps_per_update as f64;
        if self.accumulated_time > max_accumulated {
            self.accumulated_time = max_accumulated;
        }
        
        // 実行すべきステップ数を計算
        let steps = (self.accumulated_time / self.fixed_time_step).floor() as usize;
        let steps = steps.min(self.max_steps_per_update);
        
        // 実行したステップ分の時間を減算
        self.accumulated_time -= steps as f64 * self.fixed_time_step;
        
        // 残りの時間の補間係数を計算（0.0〜1.0）
        let alpha = self.accumulated_time / self.fixed_time_step;
        
        (steps, alpha)
    }
    
    /// 固定時間ステップを設定
    pub fn set_fixed_time_step(&mut self, fixed_time_step: f64) {
        self.fixed_time_step = fixed_time_step.max(0.001); // 最小値を制限
    }
    
    /// 最大ステップ数を設定
    pub fn set_max_steps_per_update(&mut self, max_steps: usize) {
        self.max_steps_per_update = max_steps.max(1); // 最小値を制限
    }
    
    /// 累積時間をリセット
    pub fn reset(&mut self) {
        self.accumulated_time = 0.0;
    }
}

/// 物理最適化システム
/// 
/// 空間分割、衝突フィルタリング、物理ステップ制御を組み合わせた
/// 包括的な物理最適化システムです。
pub struct PhysicsOptimizer {
    /// 空間分割グリッド
    pub spatial_grid: SpatialGrid,
    /// 衝突フィルタリング
    pub collision_filter: CollisionFilter,
    /// 物理ステップ制御
    pub physics_step: PhysicsStep,
}

impl PhysicsOptimizer {
    /// 新しい物理最適化システムを作成
    pub fn new(cell_size: f64, fixed_time_step: f64, max_steps: usize) -> Self {
        Self {
            spatial_grid: SpatialGrid::new(cell_size),
            collision_filter: CollisionFilter::new(),
            physics_step: PhysicsStep::new(fixed_time_step, max_steps),
        }
    }
    
    /// デフォルト設定で物理最適化システムを作成
    pub fn default() -> Self {
        Self::new(50.0, 1.0 / 60.0, 5)
    }
    
    /// エンティティのリストを空間分割グリッドに登録
    pub fn register_entities(&mut self, entities: &[PhysicsEntity]) {
        self.spatial_grid.clear();
        for entity in entities {
            self.spatial_grid.insert_entity(entity);
        }
    }
    
    /// 衝突検出の最適化（空間分割とフィルタリングを使用）
    pub fn detect_collisions(&self, entities: &HashMap<u32, PhysicsEntity>) -> Vec<(u32, u32, Collision)> {
        let mut collisions = Vec::new();
        
        // 潜在的な衝突ペアを取得
        let potential_pairs = self.spatial_grid.get_all_potential_pairs();
        
        for (entity_a_id, entity_b_id) in potential_pairs {
            // 衝突フィルタリングを適用
            if !self.collision_filter.should_collide(entity_a_id, entity_b_id) {
                continue;
            }
            
            // エンティティを取得
            if let (Some(entity_a), Some(entity_b)) = (entities.get(&entity_a_id), entities.get(&entity_b_id)) {
                // 衝突検出
                if let Some(collision) = detect_collision(entity_a, entity_b) {
                    collisions.push((entity_a_id, entity_b_id, collision));
                }
            }
        }
        
        collisions
    }
    
    /// 物理ステップ制御の更新
    pub fn update_step(&mut self, delta_time: f64) -> (usize, f64) {
        self.physics_step.update(delta_time)
    }
}

/// 衝突ペアを生成します
///
/// エンティティのリストと空間分割グリッドを使用して、潜在的な衝突ペアを生成します。
/// 衝突フィルターが指定された場合、それを使用して衝突ペアをフィルタリングします。
///
/// # 引数
///
/// * `entities` - PhysicsEntityのベクター
/// * `spatial_grid` - 空間分割グリッド
/// * `collision_filter` - 衝突フィルター（省略可能）
///
/// # 戻り値
///
/// 衝突する可能性のあるエンティティのIDペアのベクター
pub fn generate_collision_pairs(
    entities: &Vec<PhysicsEntity>,
    spatial_grid: &SpatialGrid,
    collision_filter: &Option<CollisionFilter>,
) -> Vec<(u32, u32)> {
    let mut grid = SpatialGrid::new(spatial_grid.cell_size);
    
    // 空間分割グリッドにエンティティを登録
    for entity in entities {
        grid.insert_entity(entity);
    }
    
    // 潜在的な衝突ペアを取得
    let potential_pairs = grid.get_all_potential_pairs();
    
    // 衝突フィルターが指定されている場合、フィルタリングを適用
    if let Some(filter) = collision_filter {
        potential_pairs
            .into_iter()
            .filter(|(entity_a_id, entity_b_id)| {
                filter.should_collide(*entity_a_id, *entity_b_id)
            })
            .collect()
    } else {
        // フィルターがない場合はすべてのペアを返す
        potential_pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::collision::CollisionShape;
    
    // 物理エンティティを作成するヘルパー関数
    fn create_entity(id: u32, position: (f64, f64), radius: f64) -> PhysicsEntity {
        PhysicsEntity {
            entity_id: id,
            position,
            velocity: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            mass: 1.0,
            rotation: 0.0,
            angular_velocity: 0.0,
            shape: CollisionShape::Circle { radius },
            restitution: 0.5,
            friction: 0.3,
            is_static: false,
        }
    }
    
    #[test]
    fn test_spatial_grid() {
        let mut grid = SpatialGrid::new(10.0);
        
        let entity1 = create_entity(1, (5.0, 5.0), 2.0);
        let entity2 = create_entity(2, (25.0, 5.0), 2.0);
        let entity3 = create_entity(3, (7.0, 7.0), 2.0);
        
        grid.insert_entity(&entity1);
        grid.insert_entity(&entity2);
        grid.insert_entity(&entity3);
        
        // entity1とentity3は同じセルに存在するはず
        let collisions1 = grid.get_potential_collisions(1);
        assert!(collisions1.contains(&3));
        assert!(!collisions1.contains(&2));
        
        // entity2は別のセルにあるはず
        let collisions2 = grid.get_potential_collisions(2);
        assert!(collisions2.is_empty());
        
        // すべての潜在的なペアを取得
        let pairs = grid.get_all_potential_pairs();
        assert_eq!(pairs.len(), 1);
        assert!(pairs.contains(&(1, 3)) || pairs.contains(&(3, 1)));
    }
    
    #[test]
    fn test_collision_filter() {
        let mut filter = CollisionFilter::new();
        
        // プレイヤーとエネミーは衝突する
        filter.set_category(1, category::PLAYER);
        filter.set_mask(1, category::ENEMY);
        
        filter.set_category(2, category::ENEMY);
        filter.set_mask(2, category::PLAYER);
        
        assert!(filter.should_collide(1, 2));
        
        // プレイヤーとプラットフォームは衝突する
        filter.set_category(3, category::PLATFORM);
        filter.set_mask(3, category::PLAYER | category::ENEMY);
        
        assert!(filter.should_collide(1, 3));
        assert!(filter.should_collide(2, 3));
        
        // プロジェクタイルはエネミーとのみ衝突する
        filter.set_category(4, category::PROJECTILE);
        filter.set_mask(4, category::ENEMY);
        
        assert!(!filter.should_collide(1, 4)); // プレイヤーとプロジェクタイルは衝突しない
        assert!(filter.should_collide(2, 4));  // エネミーとプロジェクタイルは衝突する
        assert!(!filter.should_collide(3, 4)); // プラットフォームとプロジェクタイルは衝突しない
    }
    
    #[test]
    fn test_physics_step() {
        let mut step = PhysicsStep::new(1.0 / 60.0, 3);
        
        // 1フレーム分の更新
        let (steps, alpha) = step.update(1.0 / 60.0);
        assert_eq!(steps, 1);
        assert!(alpha < 0.001); // ほぼ0になるはず
        
        // 2.5フレーム分の更新
        let (steps, alpha) = step.update(2.5 / 60.0);
        assert_eq!(steps, 2);
        assert!(alpha > 0.49 && alpha < 0.51); // 約0.5になるはず
        
        // 大きなデルタタイムの場合、max_stepsで制限される
        let (steps, _) = step.update(10.0 / 60.0);
        assert_eq!(steps, 3); // max_stepsで制限
    }
    
    #[test]
    fn test_physics_optimizer() {
        let mut optimizer = PhysicsOptimizer::default();
        
        // エンティティを作成
        let entity1 = create_entity(1, (5.0, 5.0), 3.0);
        let entity2 = create_entity(2, (12.0, 5.0), 3.0); // 衝突する位置
        let entity3 = create_entity(3, (100.0, 100.0), 3.0); // 遠い位置
        
        let entities = vec![entity1.clone(), entity2.clone(), entity3.clone()];
        
        // エンティティを登録
        optimizer.register_entities(&entities);
        
        // カテゴリとマスクを設定
        optimizer.collision_filter.set_category(1, category::PLAYER);
        optimizer.collision_filter.set_mask(1, category::ALL);
        
        optimizer.collision_filter.set_category(2, category::ENEMY);
        optimizer.collision_filter.set_mask(2, category::ALL);
        
        optimizer.collision_filter.set_category(3, category::PLATFORM);
        optimizer.collision_filter.set_mask(3, category::ALL);
        
        // エンティティをハッシュマップに変換
        let mut entity_map = HashMap::new();
        entity_map.insert(1, entity1);
        entity_map.insert(2, entity2);
        entity_map.insert(3, entity3);
        
        // 衝突検出
        let collisions = optimizer.detect_collisions(&entity_map);
        
        // entity1とentity2は近いので衝突するはず
        assert_eq!(collisions.len(), 1);
        let (id1, id2, _) = &collisions[0];
        assert!((*id1 == 1 && *id2 == 2) || (*id1 == 2 && *id2 == 1));
    }
} 