//! ネットワーク同期システム
//! 
//! このモジュールは、エンティティの状態をネットワーク上で同期するための
//! システムを実装します。変更検出と差分同期に重点を置いています。

use std::collections::{HashMap, HashSet};
use js_sys::Date;
use serde::{Serialize, Deserialize};

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
        let mut snapshot = EntitySnapshot::new(entity.id() as u64, now);
        
        // 各コンポーネントをスナップショットに追加
        // 実際のゲームでは、コンポーネントの具体的な型と値を取得する必要がある
        // ここでは簡略化のため、位置と速度のみをシミュレート
        
        // 例: 位置コンポーネント
        if let Some(position) = world.get_component::<PositionComponent>(entity) {
            snapshot.with_position([position.x, position.y, position.z]);
        }
        
        // 例: 速度コンポーネント
        if let Some(velocity) = world.get_component::<VelocityComponent>(entity) {
            snapshot.with_velocity([velocity.x, velocity.y, velocity.z]);
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
            .with_entity_id(snapshot.id)
            .with_components(snapshot.components);
            
        // メッセージのバイト数を計算（簡略化）
        let message_size = serde_json::to_string(&message).unwrap_or_default().len();
        
        // 帯域使用量を記録
        self.bytes_sent += message_size;
        
        message_size
    }
}

impl System for SyncSystem {
    fn name(&self) -> &'static str {
        "SyncSystem"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(200) // 通信は優先度高め
    }

    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
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
                return Ok(());
            }
        }
        
        // 同期対象のエンティティをクエリ
        let mut entities_to_sync = Vec::new();
        
        // EntityとNetworkComponentを持つエンティティを取得（クエリ動作を手動で実装）
        for entity in world.entities() {
            if let Some(network) = world.get_component::<NetworkComponent>(entity) {
                if network.is_synced && self.should_sync_entity(entity, network, now) {
                    entities_to_sync.push(entity);
                }
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
            
            for component in &snapshot.components {
                let hash = self.compute_component_hash(component);
                // コンポーネント名を取得（実際の実装ではコンポーネントから名前を取得する必要がある）
                let name = format!("Component_{}", component.component_type());
                let last_hash = state.last_component_hashes.get(&name).cloned().unwrap_or(0);
                
                // ハッシュが変わっていれば変更されたとみなす
                if hash != last_hash {
                    changed_components.insert(name.clone(), component.clone());
                    state.last_component_hashes.insert(name, hash);
                }
            }
            
            // 変更がある場合のみ同期
            if !changed_components.is_empty() {
                // 変更されたコンポーネントのみを含むスナップショットを作成
                let mut delta_snapshot = EntitySnapshot::new(snapshot.id);
                delta_snapshot.timestamp = now;
                
                for (_, component) in changed_components {
                    delta_snapshot.components.push(component);
                }
                
                // スナップショットを送信
                let bytes_sent = self.send_entity_sync(delta_snapshot);
                
                // 同期状態を更新
                state.last_sync_time = now;
                state.synced_this_frame = true;
                
                if self.config.debug_mode {
                    web_sys::console::log_1(&format!("エンティティ {:?} を同期: {}バイト", entity, bytes_sent).into());
                }
            }
        }
        
        // 未使用のエンティティ状態をクリーンアップ
        self.entity_states.retain(|entity, _| {
            world.get_component::<NetworkComponent>(*entity).is_some()
        });

        Ok(())
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

/// メッセージ圧縮機能をサポートするための各種構造体と実装
#[derive(Debug, Clone)]
pub struct DefaultMessageCompressor {
    /// 圧縮設定
    settings: CompressionSettings,
    /// エンティティごとの最後に送信した状態
    last_sent_states: HashMap<u32, EntitySnapshot>,
    /// 圧縮統計情報
    stats: CompressionStats,
}

/// 圧縮設定
#[derive(Debug, Clone)]
pub struct CompressionSettings {
    /// デルタ圧縮を有効にする（前回の値との差分のみを送信）
    enable_delta: bool,
    /// フィールドマスキングを有効にする（変更があったフィールドのみを送信）
    enable_field_masking: bool,
    /// 浮動小数点の量子化を有効にする
    enable_quantization: bool,
    /// 浮動小数点の精度（小数点以下の桁数）
    float_precision: u8,
    /// ベクトルの精度
    vector_precision: u8,
    /// 回転（クォータニオン）の精度
    rotation_precision: u8,
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            enable_delta: true,
            enable_field_masking: true,
            enable_quantization: true,
            float_precision: 2,    // 小数点以下2桁
            vector_precision: 2,   // ベクトルは小数点以下2桁
            rotation_precision: 3, // 回転は小数点以下3桁（精度重要）
        }
    }
}

/// 圧縮統計情報
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// 圧縮前の合計バイト数
    total_uncompressed_bytes: usize,
    /// 圧縮後の合計バイト数
    total_compressed_bytes: usize,
    /// 処理したメッセージ数
    message_count: usize,
    /// デルタ圧縮で省略されたフィールド数
    delta_skipped_fields: usize,
    /// マスキングで省略されたフィールド数
    masked_fields: usize,
    /// 量子化された値の数
    quantized_values: usize,
}

impl DefaultMessageCompressor {
    /// 新しいメッセージ圧縮機を作成
    pub fn new() -> Self {
        Self {
            settings: CompressionSettings::default(),
            last_sent_states: HashMap::new(),
            stats: CompressionStats::default(),
        }
    }
    
    /// カスタム設定でメッセージ圧縮機を作成
    pub fn with_settings(settings: CompressionSettings) -> Self {
        Self {
            settings,
            last_sent_states: HashMap::new(),
            stats: CompressionStats::default(),
        }
    }
    
    /// エンティティスナップショットを圧縮
    pub fn compress_snapshot(&mut self, snapshot: &mut EntitySnapshot) -> bool {
        let entity_id = snapshot.id;
        let had_previous = self.last_sent_states.contains_key(&(entity_id as u32));
        
        // 圧縮前のサイズを推定（実際の実装ではJSONエンコードなどで計算）
        let estimated_size_before = self.estimate_snapshot_size(snapshot);
        self.stats.total_uncompressed_bytes += estimated_size_before;
        
        // デルタ圧縮（前回の状態との差分のみを送信）
        if self.settings.enable_delta && had_previous {
            if let Some(last_snapshot) = self.last_sent_states.get(&(entity_id as u32)) {
                self.apply_delta_compression(snapshot, last_snapshot);
            }
        }
        
        // フィールドマスキング（変更があったフィールドのみを送信）
        if self.settings.enable_field_masking {
            self.apply_field_masking(snapshot);
        }
        
        // 浮動小数点の量子化（精度を落として送信量を減らす）
        if self.settings.enable_quantization {
            self.apply_quantization(snapshot);
        }
        
        // 圧縮後のサイズを推定
        let estimated_size_after = self.estimate_snapshot_size(snapshot);
        self.stats.total_compressed_bytes += estimated_size_after;
        self.stats.message_count += 1;
        
        // 圧縮結果をキャッシュに保存
        self.last_sent_states.insert(entity_id as u32, snapshot.clone());
        
        // データが削減されたかどうか
        estimated_size_after < estimated_size_before
    }
    
    /// スナップショットのサイズを推定（簡易実装）
    fn estimate_snapshot_size(&self, snapshot: &EntitySnapshot) -> usize {
        let mut size = 8; // ベースサイズ（entity_id + timestamp）
        
        // 各フィールドのサイズを加算
        if let Some(position) = &snapshot.position {
            size += std::mem::size_of::<[f32; 3]>();
        }
        
        if let Some(rotation) = &snapshot.rotation {
            size += std::mem::size_of::<[f32; 4]>();
        }
        
        if let Some(velocity) = &snapshot.velocity {
            size += std::mem::size_of::<[f32; 3]>();
        }
        
        if let Some(animation) = &snapshot.animation_state {
            size += animation.len();
        }
        
        if let Some(extra) = &snapshot.extra_data {
            for (key, value) in extra {
                size += key.len();
                
                // 値のサイズ（簡易推定）
                match value {
                    serde_json::Value::Null => size += 4,
                    serde_json::Value::Bool(_) => size += 1,
                    serde_json::Value::Number(n) => {
                        if n.is_i64() {
                            size += 8;
                        } else {
                            size += 8; // f64
                        }
                    },
                    serde_json::Value::String(s) => size += s.len(),
                    serde_json::Value::Array(a) => size += a.len() * 8, // 簡易推定
                    serde_json::Value::Object(o) => size += o.len() * 16, // 簡易推定
                }
            }
        }
        
        for component in &snapshot.components {
            size += std::mem::size_of::<ComponentData>();
        }
        
        size
    }
    
    /// デルタ圧縮を適用（変更がないフィールドを除外）
    fn apply_delta_compression(&mut self, current: &mut EntitySnapshot, previous: &EntitySnapshot) {
        // 各フィールドを比較し、変更がなければOptionalをNoneに設定
        if current.position == previous.position {
            current.position = None;
        }
        
        if current.rotation == previous.rotation {
            current.rotation = None;
        }
        
        if current.velocity == previous.velocity {
            current.velocity = None;
        }
        
        if current.animation_state == previous.animation_state {
            current.animation_state = None;
        }
        
        // コンポーネントのデルタ圧縮
        current.components.retain(|comp| !previous.components.contains(comp));
    }
    
    /// フィールドマスキングを適用（重要でないフィールドを除外）
    fn apply_field_masking(&mut self, snapshot: &mut EntitySnapshot) {
        // 速度が小さい場合は省略
        if let Some(velocity) = &snapshot.velocity {
            if velocity.iter().all(|v| v.abs() < 0.01) {
                snapshot.velocity = None;
            }
        }
        
        // 重要でない他のフィールドを省略
        if self.settings.enable_field_masking {
            // 実際の実装に合わせて追加
        }
    }
    
    /// 浮動小数点の量子化を適用
    fn apply_quantization(&mut self, snapshot: &mut EntitySnapshot) {
        // 位置データの量子化
        if let Some(position) = &mut snapshot.position {
            for i in 0..position.len() {
                position[i] = round_to_precision(position[i], self.settings.vector_precision);
            }
        }
        
        // 回転データの量子化
        if let Some(rotation) = &mut snapshot.rotation {
            for i in 0..rotation.len() {
                rotation[i] = round_to_precision(rotation[i], self.settings.rotation_precision);
            }
        }
        
        // 速度データの量子化
        if let Some(velocity) = &mut snapshot.velocity {
            for i in 0..velocity.len() {
                velocity[i] = round_to_precision(velocity[i], self.settings.float_precision);
            }
        }
    }
    
    /// 圧縮率を計算
    pub fn compression_ratio(&self) -> f64 {
        if self.stats.total_uncompressed_bytes == 0 {
            return 1.0;
        }
        
        self.stats.total_compressed_bytes as f64 / self.stats.total_uncompressed_bytes as f64
    }
    
    /// 統計情報を取得
    pub fn get_stats(&self) -> &CompressionStats {
        &self.stats
    }
    
    /// 統計情報をリセット
    pub fn reset_stats(&mut self) {
        self.stats = CompressionStats::default();
    }
    
    /// キャッシュをクリア
    pub fn clear_cache(&mut self) {
        self.last_sent_states.clear();
    }
    
    /// エンティティのキャッシュを削除
    pub fn remove_entity(&mut self, entity_id: u32) {
        self.last_sent_states.remove(&entity_id);
    }
}

/// メッセージ圧縮のトレイト
pub trait MessageCompressor: Send + Sync {
    /// スナップショットを圧縮
    fn compress(&self, snapshot: &EntitySnapshot) -> EntitySnapshot;
    
    /// 圧縮効率を推定（0.0〜1.0、値が小さいほど効率が良い）
    fn estimate_efficiency(&self, snapshot: &EntitySnapshot) -> f32;
}

impl MessageCompressor for DefaultMessageCompressor {
    fn compress(&self, snapshot: &EntitySnapshot) -> EntitySnapshot {
        // 実装されたcompressメソッド
        let mut compressed = snapshot.clone();
        // 圧縮処理を行う
        // ...
        compressed
    }
    
    fn estimate_efficiency(&self, snapshot: &EntitySnapshot) -> f32 {
        // 圧縮効率を推定
        // 簡易実装
        0.5
    }
}

/// エンティティのスナップショットを表す構造体
/// 
/// ネットワーク上で送信するためのエンティティ状態のスナップショットを表現します。
/// 圧縮可能な形式でデータを保持します。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    /// エンティティID
    pub id: u64,
    /// スナップショット作成時刻（サーバー時間）
    pub timestamp: f64,
    /// 位置データ [x, y, z]
    pub position: Option<[f32; 3]>,
    /// 回転データ [x, y, z, w]（クォータニオン）
    pub rotation: Option<[f32; 4]>,
    /// 速度データ [x, y, z]
    pub velocity: Option<[f32; 3]>,
    /// アニメーション状態
    pub animation_state: Option<String>,
    /// その他の追加データ
    pub extra_data: Option<HashMap<String, serde_json::Value>>,
    /// コンポーネントデータ
    pub components: Vec<ComponentData>,
}

impl EntitySnapshot {
    /// 新しいエンティティスナップショットを作成
    pub fn new(entity_id: u64, timestamp: f64) -> Self {
        Self {
            id: entity_id,
            timestamp,
            position: None,
            rotation: None,
            velocity: None,
            animation_state: None,
            extra_data: None,
            components: Vec::new(),
        }
    }
    
    /// タイムスタンプを設定したインスタンスを作成
    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }
    
    /// 位置を設定
    pub fn with_position(mut self, position: [f32; 3]) -> Self {
        self.position = Some(position);
        self
    }
    
    /// 回転を設定
    pub fn with_rotation(mut self, rotation: [f32; 4]) -> Self {
        self.rotation = Some(rotation);
        self
    }
    
    /// 速度を設定
    pub fn with_velocity(mut self, velocity: [f32; 3]) -> Self {
        self.velocity = Some(velocity);
        self
    }
    
    /// アニメーション状態を設定
    pub fn with_animation_state(mut self, state: &str) -> Self {
        self.animation_state = Some(state.to_string());
        self
    }
    
    /// 追加データを設定
    pub fn with_extra_data(mut self, data: HashMap<String, serde_json::Value>) -> Self {
        self.extra_data = Some(data);
        self
    }
    
    /// 追加データに項目を追加
    pub fn add_extra_data(&mut self, key: &str, value: serde_json::Value) {
        if self.extra_data.is_none() {
            self.extra_data = Some(HashMap::new());
        }
        
        if let Some(extra_data) = &mut self.extra_data {
            extra_data.insert(key.to_string(), value);
        }
    }
    
    /// スナップショットをJSONに変換
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
    
    /// JSONからスナップショットを復元
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
    
    /// スナップショットのバイトサイズを推定
    pub fn estimate_size(&self) -> usize {
        // 基本構造の固定サイズ（ID, タイムスタンプ）
        let mut size = 16; 
        
        // 位置データ (12バイト)
        if self.position.is_some() {
            size += 12;
        }
        
        // 回転データ (16バイト)
        if self.rotation.is_some() {
            size += 16;
        }
        
        // 速度データ (12バイト)
        if self.velocity.is_some() {
            size += 12;
        }
        
        // アニメーション状態 (可変長文字列)
        if let Some(anim) = &self.animation_state {
            size += anim.len();
        }
        
        // 追加データ (JSONオブジェクト)
        if let Some(extra) = &self.extra_data {
            size += serde_json::to_string(extra).unwrap_or_default().len();
        }
        
        size
    }
}

// 精度に基づいて値を丸める関数
fn round_to_precision(value: f32, precision: u8) -> f32 {
    if precision == 0 {
        value.round()
    } else {
        let factor = 10.0_f32.powi(precision as i32);
        (value * factor).round() / factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_snapshot() {
        let snapshot = EntitySnapshot::new(123, Date::now())
            .with_position([1.23456, 2.34567, 3.45678])
            .with_rotation([0.1234, 0.2345, 0.3456, 0.9876])
            .with_velocity([10.1234, 20.2345, 30.3456]);
            
        // スナップショットを検証
        assert_eq!(snapshot.id, 123);
        assert_eq!(snapshot.position.unwrap()[0], 1.23456);
        assert_eq!(snapshot.rotation.unwrap()[3], 0.9876);
        assert_eq!(snapshot.velocity.unwrap()[2], 30.3456);
    }
    
    #[test]
    fn test_message_compressor() {
        // 圧縮器を作成（位置は1桁、回転は2桁、速度は0桁）
        let compressor = DefaultMessageCompressor::new();
        
        // テスト用スナップショットを作成
        let snapshot = EntitySnapshot::new(1, Date::now())
            .with_position([1.23456, 2.34567, 3.45678])
            .with_rotation([0.1234, 0.2345, 0.3456, 0.9876])
            .with_velocity([10.1234, 20.2345, 30.3456]);
            
        // 圧縮を実行
        let compressed = compressor.compress(&snapshot);
        
        // 圧縮結果を検証
        assert_eq!(compressed.position.unwrap()[0], 1.2);  // 小数点以下1桁
        assert_eq!(compressed.position.unwrap()[1], 2.3);
        assert_eq!(compressed.position.unwrap()[2], 3.5);
        
        assert_eq!(compressed.rotation.unwrap()[0], 0.12); // 小数点以下2桁
        assert_eq!(compressed.rotation.unwrap()[1], 0.23);
        assert_eq!(compressed.rotation.unwrap()[2], 0.35);
        assert_eq!(compressed.rotation.unwrap()[3], 0.99);
        
        assert_eq!(compressed.velocity.unwrap()[0], 10.0); // 小数点以下0桁（整数）
        assert_eq!(compressed.velocity.unwrap()[1], 20.0);
        assert_eq!(compressed.velocity.unwrap()[2], 30.0);
        
        // 圧縮効率を検証
        let efficiency = compressor.estimate_efficiency(&snapshot);
        assert!(efficiency > 0.0 && efficiency <= 1.0);
    }
} 