//! ネットワーク同期システム
//! 
//! このモジュールは、エンティティの状態をネットワーク上で同期するための
//! システムを実装します。変更検出と差分同期に重点を置いています。

use std::collections::{HashMap, HashSet};
use js_sys::Date;

use super::messages::{EntitySnapshot, ComponentData};
use super::client::NetworkComponent;
use super::protocol::{NetworkMessage, MessageType};
use crate::ecs::{World, Entity, Component, System, Query, Changed, With, Resource};

/// 同期ポリシー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPolicy {
    /// 毎フレーム同期
    EveryFrame,
    /// 変更があった場合のみ同期
    OnChange,
    /// 一定間隔で同期
    Periodic,
    /// 距離ベースの同期（プレイヤーから遠いエンティティは更新頻度を下げる）
    DistanceBased,
}

/// コンポーネント同期設定
#[derive(Debug, Clone)]
pub struct ComponentSyncConfig {
    /// コンポーネント名
    pub name: String,
    /// 同期ポリシー
    pub policy: SyncPolicy,
    /// 同期間隔（ミリ秒、Periodicポリシーの場合に使用）
    pub interval: f64,
    /// 同期優先度（1-10、10が最高）
    pub priority: u8,
    /// 補間を適用するか
    pub interpolate: bool,
}

impl ComponentSyncConfig {
    /// 新しいコンポーネント同期設定を作成
    pub fn new(name: &str, policy: SyncPolicy) -> Self {
        Self {
            name: name.to_string(),
            policy,
            interval: 100.0,
            priority: 5,
            interpolate: true,
        }
    }
    
    /// 同期間隔を設定
    pub fn with_interval(mut self, interval: f64) -> Self {
        self.interval = interval;
        self
    }
    
    /// 優先度を設定
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }
    
    /// 補間フラグを設定
    pub fn with_interpolation(mut self, interpolate: bool) -> Self {
        self.interpolate = interpolate;
        self
    }
}

/// ネットワーク同期設定
#[derive(Debug, Clone, Resource)]
pub struct SyncConfig {
    /// 同期間隔（ミリ秒）
    pub sync_interval: f64,
    /// 帯域制限（1秒あたりの最大バイト数）
    pub bandwidth_limit: Option<usize>,
    /// 同期するコンポーネントの設定
    pub component_configs: HashMap<String, ComponentSyncConfig>,
    /// スナップショット圧縮を有効にするか
    pub compress_snapshots: bool,
    /// デバッグモードを有効にするか
    pub debug_mode: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        let mut component_configs = HashMap::new();
        
        // デフォルトのコンポーネント同期設定
        component_configs.insert(
            "Position".to_string(),
            ComponentSyncConfig::new("Position", SyncPolicy::OnChange)
                .with_priority(10)
                .with_interval(50.0)
        );
        
        component_configs.insert(
            "Velocity".to_string(),
            ComponentSyncConfig::new("Velocity", SyncPolicy::OnChange)
                .with_priority(9)
                .with_interval(100.0)
        );
        
        component_configs.insert(
            "Rotation".to_string(),
            ComponentSyncConfig::new("Rotation", SyncPolicy::OnChange)
                .with_priority(8)
        );
        
        component_configs.insert(
            "Health".to_string(),
            ComponentSyncConfig::new("Health", SyncPolicy::OnChange)
                .with_priority(7)
        );
        
        Self {
            sync_interval: 50.0, // デフォルトは20Hz
            bandwidth_limit: Some(10000), // 10KB/秒
            component_configs,
            compress_snapshots: false,
            debug_mode: false,
        }
    }
}

/// エンティティ同期状態
#[derive(Debug, Clone)]
struct EntitySyncState {
    /// 最後に同期した時刻
    last_sync_time: f64,
    /// 最後に送信したコンポーネントハッシュ
    last_component_hashes: HashMap<String, u64>,
    /// このフレームで同期されたか
    synced_this_frame: bool,
}

/// 同期システム
pub struct SyncSystem {
    /// 最後の更新時刻
    last_update: f64,
    /// エンティティ同期状態
    entity_states: HashMap<Entity, EntitySyncState>,
    /// 最後に送信したバイト数（帯域制限用）
    bytes_sent: usize,
    /// 前回の送信時刻
    last_send_time: f64,
    /// 同期の設定
    config: SyncConfig,
    /// サーバーモードかどうか
    is_server: bool,
}

impl Default for SyncSystem {
    fn default() -> Self {
        Self {
            last_update: Date::now(),
            entity_states: HashMap::new(),
            bytes_sent: 0,
            last_send_time: Date::now(),
            config: SyncConfig::default(),
            is_server: false,
        }
    }
}

impl SyncSystem {
    /// 新しい同期システムを作成（クライアント用）
    pub fn new_client(config: SyncConfig) -> Self {
        Self {
            last_update: Date::now(),
            entity_states: HashMap::new(),
            bytes_sent: 0,
            last_send_time: Date::now(),
            config,
            is_server: false,
        }
    }
    
    /// 新しい同期システムを作成（サーバー用）
    pub fn new_server(config: SyncConfig) -> Self {
        Self {
            last_update: Date::now(),
            entity_states: HashMap::new(),
            bytes_sent: 0,
            last_send_time: Date::now(),
            config,
            is_server: true,
        }
    }
    
    /// エンティティが同期対象かチェック
    fn should_sync_entity(&self, entity: Entity, network: &NetworkComponent, now: f64) -> bool {
        // エンティティの同期状態を取得
        let state = match self.entity_states.get(&entity) {
            Some(state) => state,
            None => return true, // 初めて同期する場合は必ず同期
        };
        
        // 最後の同期からの経過時間を計算
        let elapsed = now - state.last_sync_time;
        
        // 同期間隔に達していない場合は同期しない
        if elapsed < self.config.sync_interval {
            return false;
        }
        
        // リモートエンティティはサーバーからのみ同期
        if self.is_server && network.is_remote {
            return false;
        }
        
        // ローカルエンティティはクライアントからのみ同期
        if !self.is_server && !network.is_remote {
            return false;
        }
        
        true
    }
    
    /// コンポーネントが同期対象かチェック
    fn should_sync_component(&self, component_name: &str, last_sync_time: f64, now: f64) -> bool {
        let config = match self.config.component_configs.get(component_name) {
            Some(config) => config,
            None => return false, // 設定がない場合は同期しない
        };
        
        match config.policy {
            SyncPolicy::EveryFrame => true,
            SyncPolicy::OnChange => true, // 変更検出は呼び出し元で行う
            SyncPolicy::Periodic => {
                let elapsed = now - last_sync_time;
                elapsed >= config.interval
            },
            SyncPolicy::DistanceBased => {
                // 距離ベースの同期は別途実装
                true
            }
        }
    }
    
    /// エンティティのスナップショットを作成
    fn create_entity_snapshot(&self, world: &World, entity: Entity, now: f64) -> EntitySnapshot {
        let mut snapshot = EntitySnapshot::new(entity.id() as u32, now);
        
        // 各コンポーネントをスナップショットに追加
        // 実際のゲームでは、コンポーネントの具体的な型と値を取得する必要がある
        // ここでは簡略化のため、位置と速度のみをシミュレート
        
        // 例: 位置コンポーネント
        if let Some(position) = world.get_component::<PositionComponent>(entity) {
            snapshot.add_component("Position", ComponentData::Position {
                x: position.x,
                y: position.y,
                z: Some(position.z),
            });
        }
        
        // 例: 速度コンポーネント
        if let Some(velocity) = world.get_component::<VelocityComponent>(entity) {
            snapshot.add_component("Velocity", ComponentData::Velocity {
                x: velocity.x,
                y: velocity.y,
                z: Some(velocity.z),
            });
        }
        
        // 他のコンポーネントも同様に追加
        
        snapshot
    }
    
    /// コンポーネントのハッシュ値を計算（変更検出用）
    fn compute_component_hash(&self, component_data: &ComponentData) -> u64 {
        // 実際のハッシュ計算はコンポーネントの内容に基づいて行う
        // ここでは簡略化のため、コンポーネントのシリアライズ値をハッシュ化
        
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&format!("{:?}", component_data), &mut hasher);
        std::hash::Hasher::finish(&hasher)
    }
    
    /// エンティティの同期メッセージを送信
    fn send_entity_sync(&mut self, snapshot: EntitySnapshot) -> usize {
        // 実際のメッセージ送信は別のシステムで行われるため、
        // ここではバイト数の計算のみを行う
        
        // スナップショットからコンポーネント更新メッセージを作成
        let message = NetworkMessage::new(MessageType::ComponentUpdate)
            .with_entity_id(snapshot.entity_id)
            .with_components(snapshot.components);
            
        // メッセージのバイト数を計算（簡略化）
        let message_size = serde_json::to_string(&message).unwrap_or_default().len();
        
        // 帯域使用量を記録
        self.bytes_sent += message_size;
        
        message_size
    }
}

impl System for SyncSystem {
    fn run(&mut self, world: &mut World, delta_time: f32) {
        let now = Date::now();
        let elapsed = now - self.last_update;
        self.last_update = now;
        
        // 今回のフレームでの帯域制限チェック
        if let Some(limit) = self.config.bandwidth_limit {
            let time_slice = now - self.last_send_time;
            if time_slice >= 1000.0 {
                // 1秒経過したらリセット
                self.bytes_sent = 0;
                self.last_send_time = now;
            } else if self.bytes_sent >= limit {
                // 帯域制限に達した場合は同期をスキップ
                if self.config.debug_mode {
                    web_sys::console::log_1(&"帯域制限に達したため、同期をスキップします".into());
                }
                return;
            }
        }
        
        // 同期対象のエンティティをクエリ
        let query = world.query::<(Entity, &NetworkComponent)>()
            .filter(|_, network| network.is_synced);
            
        // 今回のフレームで同期するエンティティを収集
        let mut entities_to_sync = Vec::new();
        
        for (entity, network) in query.iter(world) {
            if self.should_sync_entity(entity, network, now) {
                entities_to_sync.push(entity);
            }
        }
        
        // エンティティの同期状態を更新
        for entity in entities_to_sync {
            let state = self.entity_states
                .entry(entity)
                .or_insert_with(|| EntitySyncState {
                    last_sync_time: 0.0,
                    last_component_hashes: HashMap::new(),
                    synced_this_frame: false,
                });
                
            // スナップショットを作成
            let snapshot = self.create_entity_snapshot(world, entity, now);
            
            // 変更されたコンポーネントを検出
            let mut changed_components = HashMap::new();
            
            for (name, component) in &snapshot.components {
                let hash = self.compute_component_hash(component);
                let last_hash = state.last_component_hashes.get(name).cloned().unwrap_or(0);
                
                // ハッシュが変わっていれば変更されたとみなす
                if hash != last_hash {
                    changed_components.insert(name.clone(), component.clone());
                    state.last_component_hashes.insert(name.clone(), hash);
                }
            }
            
            // 変更がある場合のみ同期
            if !changed_components.is_empty() {
                // 変更されたコンポーネントのみを含むスナップショットを作成
                let mut delta_snapshot = EntitySnapshot::new(entity.id() as u32, now);
                delta_snapshot.owner_id = snapshot.owner_id;
                
                for (name, component) in changed_components {
                    delta_snapshot.add_component(&name, component);
                }
                
                // スナップショットを送信
                let bytes_sent = self.send_entity_sync(delta_snapshot);
                
                // 同期状態を更新
                state.last_sync_time = now;
                state.synced_this_frame = true;
                
                if self.config.debug_mode {
                    web_sys::console::log_1(&format!("エンティティ {} を同期: {}バイト", entity.id(), bytes_sent).into());
                }
            }
        }
        
        // 未使用のエンティティ状態をクリーンアップ
        self.entity_states.retain(|entity, _| {
            world.get_component::<NetworkComponent>(*entity).is_some()
        });
    }
}

/// 位置コンポーネント（サンプル用）
#[derive(Debug, Clone, Component)]
struct PositionComponent {
    x: f32,
    y: f32,
    z: f32,
}

/// 速度コンポーネント（サンプル用）
#[derive(Debug, Clone, Component)]
struct VelocityComponent {
    x: f32,
    y: f32,
    z: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_sync_config() {
        let config = ComponentSyncConfig::new("Position", SyncPolicy::OnChange)
            .with_priority(10)
            .with_interval(50.0)
            .with_interpolation(true);
            
        assert_eq!(config.name, "Position");
        assert_eq!(config.policy, SyncPolicy::OnChange);
        assert_eq!(config.priority, 10);
        assert_eq!(config.interval, 50.0);
        assert!(config.interpolate);
    }
    
    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        
        assert_eq!(config.sync_interval, 50.0);
        assert!(config.component_configs.contains_key("Position"));
        assert!(config.component_configs.contains_key("Velocity"));
        assert!(config.component_configs.contains_key("Rotation"));
        assert!(config.component_configs.contains_key("Health"));
    }
} 