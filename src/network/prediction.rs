//! ネットワーク予測と補正
//! 
//! このモジュールは、ネットワークレイテンシによる影響を軽減するための
//! クライアント予測とサーバー権威による補正機能を実装します。

use std::collections::{HashMap, VecDeque};
use js_sys::Date;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsValue;

use super::messages::{InputData, EntitySnapshot, ComponentData};
use super::client::NetworkComponent;
use super::network_status::{NetworkStatus, BandwidthStatus};
use super::sync::{PositionComponent, VelocityComponent};
use super::NetworkResource;
use crate::ecs::{World, Entity, Component, System, ResourceManager, Resource};
use crate::ecs::system::{SystemPhase, SystemPriority};

/// クライアント予測データ
#[derive(Debug, Clone)]
pub struct PredictionData {
    /// 入力履歴
    pub input_history: VecDeque<InputData>,
    /// 予測状態と実際の状態の差分
    pub state_delta: HashMap<String, f32>,
    /// 最後のサーバー確認シーケンス番号
    pub last_confirmed_sequence: u32,
    /// 補間係数
    pub interpolation_factor: f32,
    /// 予測ステップ数
    pub prediction_steps: u32,
    /// 遅延補正の強さ（0.0〜1.0）
    pub correction_strength: f32,
}

impl Default for PredictionData {
    fn default() -> Self {
        Self {
            input_history: VecDeque::with_capacity(30),
            state_delta: HashMap::new(),
            last_confirmed_sequence: 0,
            interpolation_factor: 0.2,
            prediction_steps: 3,
            correction_strength: 0.3,
        }
    }
}

/// クライアント予測システム
/// 
/// ネットワーク遅延に対応するため、クライアント側で自分のアクションを予測実行します。
/// サーバーからの確認が得られたら、正確な状態と予測状態の差を補正します。
pub struct ClientPrediction {
    /// 入力履歴の最大サイズ
    max_input_history: usize,
    /// 現在の予測データ
    prediction_data: HashMap<Entity, PredictionData>,
    /// 最後の更新時刻
    last_update: f64,
}

impl Default for ClientPrediction {
    fn default() -> Self {
        Self {
            max_input_history: 30,
            prediction_data: HashMap::new(),
            last_update: Date::now(),
        }
    }
}

impl System for ClientPrediction {
    fn name(&self) -> &'static str {
        "ClientPrediction"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(40) // ServerReconciliationより少し低い優先度
    }
    
    fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        let now = Date::now();
        let _elapsed = now - self.last_update;
        self.last_update = now;
        
        // リモートエンティティのクエリ - query_tupleを使用して修正
        let mut base_query = world.query_tuple::<NetworkComponent>();
        let query = base_query.filter(|_, network| network.is_synced && network.is_remote);
        
        // エンティティと予測データを事前に収集してから処理
        let entities_to_predict: Vec<(Entity, PredictionData)> = query.iter(world)
            .map(|(entity, _network)| {
                let prediction_data = self.prediction_data
                    .entry(entity)
                    .or_insert_with(PredictionData::default)
                    .clone();
                (entity, prediction_data)
            })
            .collect();
        
        // 収集したデータを使って処理
        for (entity, prediction_data) in entities_to_predict {
            self.predict_entity_state(world, entity, &prediction_data, delta_time);
        }
        
        Ok(())
    }
}

impl ClientPrediction {
    /// 新しいクライアント予測システムを作成
    pub fn new(max_history: usize) -> Self {
        Self {
            max_input_history: max_history,
            prediction_data: HashMap::new(),
            last_update: Date::now(),
        }
    }
    
    /// エンティティの状態を予測
    fn predict_entity_state(&mut self, _world: &mut World, _entity: Entity, _prediction_data: &PredictionData, _delta_time: f32) {
        // 予測計算のロジック実装
        // 実装例: 現在の位置と速度から次のフレームの位置を予測
        // これは実際の物理演算や入力処理を簡略化したもの
        
        // 実装なし - デモのプレースホルダー
    }
    
    /// 入力を登録
    pub fn register_input(&mut self, entity: Entity, input: InputData) {
        if let Some(prediction_data) = self.prediction_data.get_mut(&entity) {
            prediction_data.input_history.push_back(input);
            
            // 履歴のサイズを制限
            while prediction_data.input_history.len() > self.max_input_history {
                prediction_data.input_history.pop_front();
            }
        }
    }
    
    /// サーバーからの状態更新を処理
    pub fn apply_server_correction(&mut self, _world: &mut World, entity: Entity, _snapshot: &EntitySnapshot, sequence: u32) {
        if let Some(prediction_data) = self.prediction_data.get_mut(&entity) {
            // 確認されたシーケンス番号を更新
            prediction_data.last_confirmed_sequence = sequence;
            
            // スナップショットに基づいてエンティティを更新
            // ここでは簡略化のため、位置コンポーネントのみを処理
            
            // サーバーからの位置と現在の予測位置の差を計算
            // 実際にはより複雑な処理が必要になるが、基本原理は同じ
            
            // 差が大きい場合は直接状態を修正
            // 差が小さい場合は徐々に補正
            
            // 大幅なずれが検出された場合、予測モデルを再調整
        }
    }
}

/// サーバー再調整システム
/// 
/// サーバー側で、クライアントの予測とサーバーの権威的状態の違いを検出し、
/// 必要に応じてクライアントに修正を通知します。
pub struct ServerReconciliation {
    /// クライアントの入力履歴
    client_inputs: HashMap<u32, VecDeque<(InputData, u32)>>, // (入力, シーケンス番号)
    /// 最後の更新時刻
    last_update: f64,
    /// 補正メッセージを送信するための閾値
    correction_threshold: f32,
    /// 最大適用ステップ数（パフォーマンス調整用）
    max_steps_per_frame: usize,
}

impl Default for ServerReconciliation {
    fn default() -> Self {
        Self {
            client_inputs: HashMap::new(),
            last_update: Date::now(),
            correction_threshold: 0.5,
            max_steps_per_frame: 30,
        }
    }
}

impl System for ServerReconciliation {
    fn name(&self) -> &'static str {
        "ServerReconciliation"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(50) // 中程度の優先度
    }
    
    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        let now = Date::now();
        self.last_update = now;
        
        // クライアント所有のエンティティを検出
        let owned_entities = self.get_client_owned_entities(world);
        
        // ネットワーク状態を先に確認（不変借用）
        let mut optimize_for_poor_network = false;
        {
            if let Some(network_status) = resources.get::<NetworkStatus>() {
                optimize_for_poor_network = network_status.bandwidth_status == BandwidthStatus::Limited || 
                                           network_status.bandwidth_status == BandwidthStatus::Critical;
            }
        }
        
        // ネットワーク送信キューを取得（可変借用）
        let mut send_queue_opt = resources.get_mut::<NetworkSendQueue>();
        let send_queue = match send_queue_opt {
            Some(ref mut queue) => queue,
            None => {
                #[cfg(feature = "debug_network")]
                web_sys::console::log_1(&"エラー: NetworkSendQueueが見つかりません。修正を送信できません。".into());
                return Ok(());
            }
        };
        
        // 各クライアント所有のエンティティに対して処理
        for (client_id, entity) in owned_entities {
            // クライアントの入力履歴を取得
            if let Some(inputs) = self.client_inputs.get(&client_id) {
                // 入力がない場合はスキップ
                if inputs.is_empty() {
                    continue;
                }
                
                // 状態を一時的に保存（権威的な初期状態）
                let initial_state = self.capture_entity_state(world, entity);
                
                // 入力を順番に適用する（最大でmax_steps_per_frameまで）
                let step_count = inputs.len().min(self.max_steps_per_frame);
                
                // シミュレーション用の時間ステップを計算
                let sim_delta_time = if step_count > 0 {
                    delta_time / step_count as f32
                } else {
                    delta_time
                };
                
                // 最終確認シーケンス番号を追跡
                let mut last_sequence = 0;
                
                // 各入力を適用
                for i in 0..step_count {
                    if let Some((input, sequence)) = inputs.get(i) {
                        // 入力をエンティティに適用
                        self.apply_input_to_entity(world, entity, input, sim_delta_time);
                        
                        // 最後のシーケンス番号を更新
                        last_sequence = *sequence;
                    }
                }
                
                // 最終状態を保存
                let final_state = self.capture_entity_state(world, entity);
                
                // 初期状態と最終状態を比較して違いがあるかをチェック
                if self.should_send_correction(&initial_state, &final_state) {
                    // エンティティのスナップショットを作成
                    let snapshot = self.create_entity_snapshot(world, entity, now);
                    
                    // ネットワーク品質が低い場合はスナップショットを最適化
                    let mut optimized_snapshot = snapshot.clone();
                    
                    // ネットワーク状態に基づいて最適化
                    if optimize_for_poor_network {
                        // 限られた帯域幅に対してスナップショットを最適化
                        self.optimize_snapshot_for_poor_network(&mut optimized_snapshot);
                    }
                    
                    // 修正データをキューに追加
                    #[cfg(feature = "debug_network")]
                    web_sys::console::log_1(&format!("ServerReconciliation: クライアント {} のエンティティ {} に修正を送信 (seq: {})",
                        client_id, entity.index() as u32, last_sequence).into());
                    
                    // 修正スナップショットを送信キューに追加
                    send_queue.queue_snapshot(client_id, entity, optimized_snapshot, last_sequence);
                    
                    // いくつかの主要コンポーネントについて、予測精度の分析を行う
                    if let (Some(initial_pos), Some(final_pos)) = (
                        initial_state.get("position"),
                        final_state.get("position")
                    ) {
                        // 位置の差異を計算して分析
                        let difference = self.calculate_component_difference("position", initial_pos, final_pos);
                        self.analyze_prediction_accuracy(client_id, "position", difference);
                    }
                    
                    // 速度の予測精度も分析
                    if let (Some(initial_vel), Some(final_vel)) = (
                        initial_state.get("velocity"),
                        final_state.get("velocity")
                    ) {
                        let difference = self.calculate_component_difference("velocity", initial_vel, final_vel);
                        self.analyze_prediction_accuracy(client_id, "velocity", difference);
                    }
                }
            }
        }
        
        // 古い入力をクリーンアップ
        self.cleanup_old_inputs();
        
        Ok(())
    }
}

impl ServerReconciliation {
    /// 新しいサーバー再調整システムを作成
    pub fn new() -> Self {
        ServerReconciliation {
            client_inputs: HashMap::new(),
            max_steps_per_frame: 5,
            correction_threshold: 0.5,
            last_update: Date::now(),
        }
    }
    
    /// 補正閾値を設定したインスタンスを作成
    pub fn with_threshold(threshold: f32) -> Self {
        let mut instance = Self::default();
        instance.correction_threshold = threshold;
        instance
    }
    
    /// クライアントからの入力を処理
    pub fn register_input(&mut self, client_id: u32, input: InputData, sequence: u32) {
        let inputs = self.client_inputs
            .entry(client_id)
            .or_insert_with(|| VecDeque::with_capacity(30));
            
        inputs.push_back((input, sequence));
        
        // 入力履歴のサイズを制限
        const MAX_INPUT_HISTORY: usize = 30;
        while inputs.len() > MAX_INPUT_HISTORY {
            inputs.pop_front();
        }
    }
    
    /// クライアント所有のエンティティを取得
    fn get_client_owned_entities(&self, _world: &World) -> Vec<(u32, Entity)> {
        // 実際の実装では、world から NetworkComponent と OwnershipComponent を持つエンティティを取得
        // ここでは簡略化のためにダミーデータを返す
        vec![]
    }
    
    /// エンティティの状態をキャプチャ
    fn capture_entity_state(&self, world: &World, entity: Entity) -> HashMap<String, ComponentData> {
        let mut components = HashMap::new();
        
        // 位置コンポーネントを取得（存在する場合）
        if let Some(position) = world.get_component::<PositionComponent>(entity) {
            components.insert("position".to_string(), ComponentData::Position {
                x: position.x,
                y: position.y,
                z: position.z,
            });
        }
        
        // 速度コンポーネントを取得（存在する場合）
        if let Some(velocity) = world.get_component::<VelocityComponent>(entity) {
            components.insert("velocity".to_string(), ComponentData::Velocity {
                x: velocity.x,
                y: velocity.y,
                z: velocity.z,
            });
        }
        
        // その他の同期が必要なコンポーネントを追加
        // ここではゲーム固有のコンポーネントを追加する
        // 例: 回転、状態、体力など
        
        components
    }
    
    /// 入力をエンティティに適用
    fn apply_input_to_entity(&self, world: &mut World, entity: Entity, input: &InputData, delta_time: f32) {
        // 入力を基にエンティティのコンポーネントを更新
        
        // 位置と速度の更新処理
        // まず必要なデータを収集
        let position_update = {
            let position = world.get_component::<PositionComponent>(entity);
            let velocity = world.get_component::<VelocityComponent>(entity);
            
            if let (Some(position), Some(velocity)) = (position, velocity) {
                // 入力に基づいた速度と位置の計算
                let (move_x, move_y) = input.movement;
                let speed = 5.0; // 適切なスピード係数（ゲームバランスに応じて調整）
                
                // 新しい速度値
                let new_vel_x = move_x * speed;
                let new_vel_y = move_y * speed;
                
                // 新しい位置値
                let new_pos_x = position.x + new_vel_x * delta_time;
                let new_pos_y = position.y + new_vel_y * delta_time;
                
                // Z軸の計算（必要な場合）
                let (new_pos_z, new_vel_z) = if let (Some(pos_z), Some(vel_z)) = (position.z, velocity.z) {
                    if pos_z > 0.0 {
                        // 空中にいる場合は重力を適用
                        let new_vel = vel_z - 9.8 * delta_time; // 重力加速度
                        let new_pos = pos_z + new_vel * delta_time;
                        (Some(if new_pos < 0.0 { 0.0 } else { new_pos }), Some(new_vel))
                    } else {
                        // 地面にいる場合
                        (Some(0.0), Some(0.0))
                    }
                } else {
                    (position.z, velocity.z)
                };
                
                Some((
                    (new_pos_x, new_pos_y, new_pos_z),
                    (new_vel_x, new_vel_y, new_vel_z)
                ))
            } else {
                None
            }
        };
        
        // 計算した値で実際にコンポーネントを更新
        if let Some(((new_pos_x, new_pos_y, new_pos_z), (new_vel_x, new_vel_y, new_vel_z))) = position_update {
            // 速度の更新
            if let Some(velocity) = world.get_component_mut::<VelocityComponent>(entity) {
                velocity.x = new_vel_x;
                velocity.y = new_vel_y;
                velocity.z = new_vel_z;
            }
            
            // 位置の更新
            if let Some(position) = world.get_component_mut::<PositionComponent>(entity) {
                position.x = new_pos_x;
                position.y = new_pos_y;
                position.z = new_pos_z;
            }
        }
        
        // アクション入力の処理
        for (action_name, is_active) in &input.actions {
            if *is_active {
                // 各アクションタイプに対応する処理
                match action_name.as_str() {
                    "jump" => {
                        // ジャンプ処理
                        // 先に位置情報を確認してからジャンプを適用
                        let should_jump = {
                            let position = world.get_component::<PositionComponent>(entity);
                            // 地面に近いかチェック
                            position.map_or(false, |pos| pos.z.map_or(true, |z| z <= 0.01))
                        };
                        
                        // ジャンプが可能な場合のみ速度を更新
                        if should_jump {
                            if let Some(velocity) = world.get_component_mut::<VelocityComponent>(entity) {
                                velocity.z = Some(10.0); // ジャンプ力
                            }
                        }
                    },
                    "attack" => {
                        // 攻撃処理
                        // 実際のゲームロジックに応じて実装
                    },
                    "use" => {
                        // アイテム使用処理
                        // 実際のゲームロジックに応じて実装
                    },
                    // その他のアクションタイプを追加
                    _ => {
                        // 未知のアクションは無視
                        #[cfg(feature = "debug_network")]
                        web_sys::console::log_1(&format!("未知のアクション: {}", action_name).into());
                    }
                }
            }
        }
    }
    
    /// 修正を送信するべきかを判断
    fn should_send_correction(
        &self,
        initial_state: &HashMap<String, ComponentData>,
        final_state: &HashMap<String, ComponentData>,
    ) -> bool {
        // 主要なコンポーネントについて、重要な違いがあるかをチェック
        
        // 位置の大きな変化があるか
        let position_diff = self.has_significant_difference(
            "position",
            initial_state,
            final_state,
            self.correction_threshold // 位置の閾値（ゲーム内単位）
        );
        
        // 速度の大きな変化があるか
        let velocity_diff = self.has_significant_difference(
            "velocity",
            initial_state,
            final_state,
            self.correction_threshold * 2.0 // 速度の閾値は位置より大きめに
        );
        
        // その他の重要なコンポーネントの変化...
        // ゲーム固有のロジックに応じて追加
        
        // 少なくとも1つの重要な変化があれば修正を送信
        position_diff || velocity_diff
    }
    
    /// コンポーネントの違いが閾値を超えているかチェック
    fn has_significant_difference(
        &self, 
        component_name: &str,
        initial_state: &HashMap<String, ComponentData>,
        final_state: &HashMap<String, ComponentData>,
        threshold: f32
    ) -> bool {
        // 両方の状態にコンポーネントが存在するか確認
        let (initial, final_) = match (
            initial_state.get(component_name),
            final_state.get(component_name)
        ) {
            (Some(initial), Some(final_)) => (initial, final_),
            _ => return false, // どちらかが存在しない場合は重要な違いはない
        };
        
        // コンポーネントタイプに基づいて差異を計算
        match (initial, final_) {
            (ComponentData::Position { x: x1, y: y1, z: z1 },
             ComponentData::Position { x: x2, y: y2, z: z2 }) => {
                // 位置の差異を計算
                let dx = x1 - x2;
                let dy = y1 - y2;
                
                // 2D距離
                let distance_2d = (dx * dx + dy * dy).sqrt();
                
                // Z座標がある場合は3D距離を計算
                if let (Some(z1_val), Some(z2_val)) = (z1, z2) {
                    let dz = z1_val - z2_val;
                    let distance_3d = (dx * dx + dy * dy + dz * dz).sqrt();
                    distance_3d > threshold
                } else {
                    distance_2d > threshold
                }
            },
            (ComponentData::Velocity { x: x1, y: y1, z: z1 },
             ComponentData::Velocity { x: x2, y: y2, z: z2 }) => {
                // 速度の差異を計算（位置と同様）
                let dx = x1 - x2;
                let dy = y1 - y2;
                let difference = (dx * dx + dy * dy).sqrt();
                
                if let (Some(z1_val), Some(z2_val)) = (z1, z2) {
                    let dz = z1_val - z2_val;
                    let diff_3d = (dx * dx + dy * dy + dz * dz).sqrt();
                    diff_3d > threshold
                } else {
                    difference > threshold
                }
            },
            // その他のコンポーネントタイプの比較
            // 例: 回転、状態、体力など
            _ => false, // サポートされていないコンポーネントタイプ
        }
    }
    
    /// エンティティのスナップショットを作成
    fn create_entity_snapshot(&self, world: &World, entity: Entity, timestamp: f64) -> EntitySnapshot {
        // 新しいスナップショットを作成
        let mut snapshot = EntitySnapshot::new(entity.index() as u32, timestamp);
        
        // コンポーネントデータを収集
        let mut components = HashMap::new();
        
        // 位置コンポーネント
        if let Some(position) = world.get_component::<PositionComponent>(entity) {
            components.insert("position".to_string(), ComponentData::Position {
                x: position.x,
                y: position.y,
                z: position.z,
            });
        }
        
        // 速度コンポーネント
        if let Some(velocity) = world.get_component::<VelocityComponent>(entity) {
            components.insert("velocity".to_string(), ComponentData::Velocity {
                x: velocity.x,
                y: velocity.y,
                z: velocity.z,
            });
        }
        
        // その他の必要なコンポーネント
        // ゲーム固有のコンポーネントを追加
        
        // スナップショットにコンポーネントを設定
        snapshot.components = components;
        
        snapshot
    }
    
    /// 貧弱なネットワーク向けにスナップショットを最適化
    fn optimize_snapshot_for_poor_network(&self, snapshot: &mut EntitySnapshot) {
        // 重要なコンポーネント（常に送信する必要があるもの）
        let important_components = ["position", "velocity", "health"];
        
        // 非重要なコンポーネントを除外
        let components_to_remove: Vec<String> = snapshot.components.keys()
            .filter(|k| !important_components.contains(&k.as_str()))
            .cloned()
            .collect();
        
        // 非重要なコンポーネントを削除
        for key in components_to_remove {
            snapshot.components.remove(&key);
        }
        
        // 数値の精度を下げて帯域幅を節約
        // 位置コンポーネントの精度を下げる
        if let Some(ComponentData::Position { x, y, z }) = snapshot.components.get_mut("position") {
            *x = (*x * 10.0).round() / 10.0; // 小数第一位までの精度
            *y = (*y * 10.0).round() / 10.0;
            if let Some(ref mut z_val) = z {
                *z_val = (*z_val * 10.0).round() / 10.0;
            }
        }
        
        // 速度コンポーネントの精度を下げる
        if let Some(ComponentData::Velocity { x, y, z }) = snapshot.components.get_mut("velocity") {
            *x = (*x * 10.0).round() / 10.0;
            *y = (*y * 10.0).round() / 10.0;
            if let Some(ref mut z_val) = z {
                *z_val = (*z_val * 10.0).round() / 10.0;
            }
        }
    }
    
    /// コンポーネントの差異を計算
    fn calculate_component_difference(&self, _component_name: &str, initial: &ComponentData, final_: &ComponentData) -> f32 {
        match (initial, final_) {
            (ComponentData::Position { x: x1, y: y1, z: z1 },
             ComponentData::Position { x: x2, y: y2, z: z2 }) => {
                let dx = x1 - x2;
                let dy = y1 - y2;
                
                if let (Some(z1_val), Some(z2_val)) = (z1, z2) {
                    let dz = z1_val - z2_val;
                    (dx * dx + dy * dy + dz * dz).sqrt()
                } else {
                    (dx * dx + dy * dy).sqrt()
                }
            },
            (ComponentData::Velocity { x: x1, y: y1, z: z1 },
             ComponentData::Velocity { x: x2, y: y2, z: z2 }) => {
                let dx = x1 - x2;
                let dy = y1 - y2;
                
                if let (Some(z1_val), Some(z2_val)) = (z1, z2) {
                    let dz = z1_val - z2_val;
                    (dx * dx + dy * dy + dz * dz).sqrt()
                } else {
                    (dx * dx + dy * dy).sqrt()
                }
            },
            _ => 0.0, // サポートされていないコンポーネントタイプ
        }
    }
    
    /// 予測精度の分析
    fn analyze_prediction_accuracy(&self, _client_id: u32, _component: &str, difference: f32) {
        // ここで予測精度のログを記録したり分析データを蓄積したりします
        #[cfg(feature = "debug_network")]
        web_sys::console::log_1(&format!(
            "予測分析 - 差異: {:.3}",
            difference
        ).into());
        
        // 大きな差異がある場合、追加のデバッグ情報を記録
        if difference > 3.0 {
            #[cfg(feature = "debug_network")]
            web_sys::console::log_1(&"警告: 大きな予測誤差を検出".into());
        }
    }
    
    /// 古い入力をクリーンアップ
    fn cleanup_old_inputs(&mut self) {
        // 各クライアントの古い入力を削除
        for inputs in self.client_inputs.values_mut() {
            // キューが大きすぎる場合、古い入力を削除
            while inputs.len() > self.max_steps_per_frame * 2 {
                inputs.pop_front();
            }
        }
    }
}

// ComponentトレイトをNetworkComponentに実装
impl Component for NetworkComponent {
    fn name() -> &'static str {
        "NetworkComponent"
    }
}

// ComponentトレイトをPositionComponentに実装
impl Component for PositionComponent {
    fn name() -> &'static str {
        "PositionComponent"
    }
}

// ComponentトレイトをVelocityComponentに実装
impl Component for VelocityComponent {
    fn name() -> &'static str {
        "VelocityComponent"
    }
}

// ResourceトレイトをNetworkSendQueueに実装
impl Resource for NetworkSendQueue {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ResourceトレイトをNetworkStatusに実装
impl Resource for NetworkStatus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// ネットワーク送信キュー
/// 
/// サーバーからクライアントへのスナップショット送信を管理します
#[derive(Default)]
pub struct NetworkSendQueue {
    queue: Vec<(u32, Entity, EntitySnapshot, u32)>,
}

impl NetworkSendQueue {
    /// 新しい送信キューを作成
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
        }
    }
    
    /// 送信キューにスナップショットを追加
    pub fn queue_snapshot(&mut self, client_id: u32, entity: Entity, snapshot: EntitySnapshot, sequence: u32) {
        self.queue.push((client_id, entity, snapshot, sequence));
    }
    
    /// キューを処理して実際に送信
    pub fn process_queue(&mut self, network_client: &mut NetworkClient) {
        for (_client_id, entity, snapshot, sequence) in self.queue.drain(..) {
            // スナップショットメッセージを作成
            let message = NetworkMessage::new(MessageType::ComponentUpdate)
                .with_sequence(sequence)
                .with_entity_id(entity.index() as u32)
                .with_components(snapshot.components);
                
            // メッセージを送信
            if let Err(_e) = network_client.send_message(message) {
                #[cfg(feature = "debug_network")]
                web_sys::console::log_1(&format!(
                    "エラー: クライアント {} へのメッセージ送信に失敗: {:?}",
                    _client_id, _e
                ).into());
            }
        }
    }
    
    pub fn queue_message(&mut self, _client_id: u32, _message: NetworkMessage) {
        // このメソッドはメッセージを直接キューに追加
        // 実際の実装ではメッセージタイプに基づいて適切な処理を行う
        // ここでは簡略化のためにログだけ出力
        #[cfg(feature = "debug_network")]
        web_sys::console::log_1(&"Queued message".into());
    }
}

/// 状態補間システム
/// 
/// 他プレイヤーのエンティティを滑らかに補間表示するためのシステム
pub struct InterpolationSystem {
    /// 補間バッファの時間（ミリ秒）
    _buffer_time: f64,
    /// 最後の更新時刻
    last_update: f64,
}

impl Default for InterpolationSystem {
    fn default() -> Self {
        Self {
            _buffer_time: 100.0, // 100ms
            last_update: Date::now(),
        }
    }
}

impl System for InterpolationSystem {
    fn name(&self) -> &'static str {
        "InterpolationSystem"
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(10) // 適切な優先度を設定
    }

    fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        let now = Date::now();
        let _elapsed = now - self.last_update;
        self.last_update = now;
        
        // リモートエンティティのクエリ - query_tupleを使用して修正
        let mut base_query = world.query_tuple::<NetworkComponent>();
        let query = base_query.filter(|_, network| network.is_synced && network.is_remote);
            
        for (_entity, _network) in query.iter(world) {
            // このエンティティの過去のスナップショットを取得
            // 補間時間（現在時刻 - バッファ時間）に基づいて、適切なスナップショットを選択
            // 選択したスナップショット間で線形補間を行い、滑らかな動きを実現
        }
        
        Ok(())
    }
}

impl InterpolationSystem {
    /// 新しい補間システムを作成
    pub fn new(buffer_time: f64) -> Self {
        Self {
            _buffer_time: buffer_time,
            last_update: Date::now(),
        }
    }
}

/// ネットワークエンティティ同期システム
/// 
/// サーバーからクライアントへのエンティティ状態同期を管理します。
pub struct EntitySyncSystem {
    /// 最後の更新時刻
    last_update: f64,
    /// エンティティのスナップショット履歴
    entity_snapshots: HashMap<Entity, VecDeque<EntitySnapshot>>,
}

impl Default for EntitySyncSystem {
    fn default() -> Self {
        Self {
            last_update: Date::now(),
            entity_snapshots: HashMap::new(),
        }
    }
}

impl System for EntitySyncSystem {
    fn name(&self) -> &'static str {
        "EntitySyncSystem"
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(20) // InterpolationSystemより高い優先度
    }

    fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        let now = Date::now();
        self.last_update = now;
        
        // リモートエンティティのクエリ - query_tupleを使用して修正
        let mut base_query = world.query_tuple::<NetworkComponent>();
        let query = base_query.filter(|_, network| network.is_synced && network.is_remote);
            
        for (_entity, _network) in query.iter(world) {
            // このエンティティの過去のスナップショットを取得
            // 補間時間（現在時刻 - バッファ時間）に基づいて、適切なスナップショットを選択
            // 選択したスナップショット間で線形補間を行い、滑らかな動きを実現
        }

        Ok(())
    }
}

impl EntitySyncSystem {
    /// 新しいエンティティ同期システムを作成
    pub fn new() -> Self {
        Self {
            last_update: Date::now(),
            entity_snapshots: HashMap::new(),
        }
    }
    
    /// エンティティのスナップショットを登録
    pub fn register_snapshot(&mut self, entity: Entity, snapshot: EntitySnapshot) {
        let snapshots = self.entity_snapshots
            .entry(entity)
            .or_insert_with(|| VecDeque::with_capacity(20));
            
        snapshots.push_back(snapshot);
        
        // スナップショット履歴のサイズを制限
        const MAX_SNAPSHOTS: usize = 20;
        while snapshots.len() > MAX_SNAPSHOTS {
            snapshots.pop_front();
        }
    }
}

/// 予測システム（クライアントとサーバーの両方で使用）
pub struct PredictionSystem {
    /// クライアント予測システム（クライアント側で使用）
    pub client_prediction: ClientPrediction,
    /// サーバー再調整システム（サーバー側で使用）
    pub server_reconciliation: ServerReconciliation,
    /// 状態補間システム（主にクライアント側で使用）
    pub interpolation: InterpolationSystem,
    /// エンティティ同期システム（主にサーバー側で使用）
    pub entity_sync: EntitySyncSystem,
    /// 実行モードを指定
    is_server: bool,
}

impl PredictionSystem {
    /// クライアントモードで予測システムを作成
    pub fn new_client() -> Self {
        Self {
            client_prediction: ClientPrediction::default(),
            server_reconciliation: ServerReconciliation::default(),
            interpolation: InterpolationSystem::default(),
            entity_sync: EntitySyncSystem::default(),
            is_server: false,
        }
    }
    
    /// サーバーモードで予測システムを作成
    pub fn new_server() -> Self {
        Self {
            client_prediction: ClientPrediction::default(),
            server_reconciliation: ServerReconciliation::default(),
            interpolation: InterpolationSystem::default(),
            entity_sync: EntitySyncSystem::default(),
            is_server: true,
        }
    }
}

impl System for PredictionSystem {
    fn name(&self) -> &'static str {
        "PredictionSystem"
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(10) // ネットワーク関連のため高い優先度
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }

    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        if self.is_server {
            // サーバーモードでの処理
            self.server_reconciliation.run(world, resources, delta_time)?;
            self.entity_sync.run(world, resources, delta_time)?;
        } else {
            // クライアントモードでの処理
            self.client_prediction.run(world, resources, delta_time)?;
            self.interpolation.run(world, resources, delta_time)?;
        }
        Ok(())
    }
}

/// 入力遅延補正システム
/// 
/// ネットワーク遅延による入力の遅れを補正し、
/// プレイヤー体験を向上させるためのシステムです。
pub struct InputLatencyCompensationSystem {
    /// 入力バッファ
    input_buffer: VecDeque<InputData>,
    /// ネットワーク品質モニタ
    network_monitor: Option<Arc<Mutex<NetworkQualityMonitor>>>,
    /// 補正設定
    compensation_settings: LatencyCompensationSettings,
    /// 最後の更新時刻
    last_update: f64,
}

/// 遅延補正設定
#[derive(Debug, Clone)]
pub struct LatencyCompensationSettings {
    /// 予測先行時間（ミリ秒）
    pub look_ahead_time: f64,
    /// 入力補間有効フラグ
    pub use_input_interpolation: bool,
    /// 入力平滑化係数（0.0〜1.0）
    pub input_smoothing_factor: f32,
    /// 入力予測有効フラグ
    pub use_input_prediction: bool,
    /// バッファサイズ
    pub buffer_size: usize,
}

impl Default for LatencyCompensationSettings {
    fn default() -> Self {
        Self {
            look_ahead_time: 100.0, // 100ms先読み
            use_input_interpolation: true,
            input_smoothing_factor: 0.5,
            use_input_prediction: true,
            buffer_size: 10,
        }
    }
}

impl Default for InputLatencyCompensationSystem {
    fn default() -> Self {
        Self {
            input_buffer: VecDeque::with_capacity(10),
            network_monitor: None,
            compensation_settings: LatencyCompensationSettings::default(),
            last_update: Date::now(),
        }
    }
}

impl System for InputLatencyCompensationSystem {
    fn name(&self) -> &'static str {
        "InputLatencyCompensationSystem"
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(20) // 中程度の優先度
    }

    fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        let now = Date::now();
        let _elapsed = now - self.last_update;
        self.last_update = now;
        
        // 先に可変借用が必要な入力リソースを取得
        let mut input_data = None;
        if let Some(input_resource) = world.get_resource_mut::<InputResource>() {
            input_data = Some(input_resource.get_current_input().clone());
        }
        
        // 入力データが取得できなかったら早期リターン
        let current_input = match input_data {
            Some(input) => input,
            None => return Ok(()),
        };
        
        // 次に不変借用のネットワークリソースを取得
        let network_resource = match world.get_resource::<NetworkResource>() {
            Some(resource) => resource,
            None => return Ok(()),
        };
        
        // ローカルプレイヤーエンティティを取得
        let local_entity = match network_resource.controlled_entity {
            Some(entity) => entity,
            None => return Ok(()),
        };
        
        // 入力バッファを更新
        self.update_input_buffer(current_input.clone());
        
        // RTTに基づいて先読み時間を調整
        let _look_ahead_time = self.calculate_look_ahead_time(network_resource.rtt);
        
        // 遅延補正された入力を計算
        let compensated_input = self.compensate_input_latency();
        
        // 補正された入力を適用
        self.apply_compensated_input(world, local_entity, compensated_input, delta_time);
        
        if let Some(monitor_lock) = &self.network_monitor {
            if let Ok(monitor) = monitor_lock.lock() {
                if monitor.packet_loss > 0.01 {
                    // Only apply compensation when there is packet loss
                    let _compensation_factor = (monitor.packet_loss * 10.0).min(0.2); // Cap at 20% compensation
                    
                    // Find entities with network component
                    for _entity in world.query_entities::<NetworkComponent>() {
                        // Apply input lag compensation here (simplified)
                        // In a real implementation, you'd adjust input processing timing
                        // based on the network conditions
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl InputLatencyCompensationSystem {
    /// 新しい入力遅延補正システムを作成
    pub fn new(settings: LatencyCompensationSettings) -> Self {
        Self {
            input_buffer: VecDeque::with_capacity(settings.buffer_size),
            network_monitor: None,
            compensation_settings: settings,
            last_update: Date::now(),
        }
    }
    
    /// ネットワーク品質モニタを設定
    pub fn with_network_monitor(mut self, monitor: Arc<Mutex<NetworkQualityMonitor>>) -> Self {
        self.network_monitor = Some(monitor);
        self
    }
    
    /// 入力バッファを更新
    fn update_input_buffer(&mut self, input: InputData) {
        self.input_buffer.push_back(input);
        
        // バッファサイズを制限
        while self.input_buffer.len() > self.compensation_settings.buffer_size {
            self.input_buffer.pop_front();
        }
    }
    
    /// 先読み時間を計算
    fn calculate_look_ahead_time(&self, rtt: f64) -> f64 {
        // 基本的な先読み時間
        let base_look_ahead = self.compensation_settings.look_ahead_time;
        
        // ネットワーク品質に基づいて先読み時間を調整
        if let Some(monitor_lock) = &self.network_monitor {
            if let Ok(monitor) = monitor_lock.lock() {
                // RTTとジッターに基づいて適応的に調整
                let adjusted_time = base_look_ahead + monitor.avg_rtt * 0.5 + monitor.jitter * 2.0;
                
                // 品質が低下した場合は先読み時間を増やす
                if monitor.packet_loss > 0.05 {
                    // f32からf64へ変換して計算
                    return adjusted_time * (1.0 + f64::from(monitor.packet_loss) * 2.0);
                } else {
                    return adjusted_time;
                }
            }
        }
        
        // デフォルトはRTTの半分を先読み時間に加算
        base_look_ahead + rtt * 0.5
    }
    
    /// 入力遅延を補正
    fn compensate_input_latency(&self) -> InputData {
        if self.input_buffer.is_empty() {
            return InputData::default();
        }
        
        // バッファが小さい場合は最新の入力を使用
        if self.input_buffer.len() < 2 || !self.compensation_settings.use_input_interpolation {
            return self.input_buffer.back().unwrap().clone();
        }
        
        if self.compensation_settings.use_input_prediction && self.input_buffer.len() >= 3 {
            // 入力予測: 直近の入力から次の入力を予測
            return self.predict_next_input();
        } else {
            // 入力補間: 直近の2つの入力を補間
            return self.interpolate_inputs();
        }
    }
    
    /// 入力の予測
    fn predict_next_input(&self) -> InputData {
        let len = self.input_buffer.len();
        let input1 = &self.input_buffer[len - 3];
        let input2 = &self.input_buffer[len - 2];
        let input3 = &self.input_buffer[len - 1];
        
        // 単純な線形予測
        let mut predicted_input = input3.clone();
        
        // 移動入力の予測
        let (m1, m2, m3) = (input1.movement, input2.movement, input3.movement);
        // 2つの差分から次の移動を予測
        let dx1 = m2.0 - m1.0;
        let dy1 = m2.1 - m1.1;
        let dx2 = m3.0 - m2.0;
        let dy2 = m3.1 - m2.1;
        
        // 加速度を計算
        let ax = dx2 - dx1;
        let ay = dy2 - dy1;
        
        // 予測移動値
        let px = m3.0 + dx2 + ax * 0.5;
        let py = m3.1 + dy2 + ay * 0.5;
        
        // 値を-1.0〜1.0の範囲に制限
        let px = px.max(-1.0).min(1.0);
        let py = py.max(-1.0).min(1.0);
        
        predicted_input.movement = (px, py);
        
        // アクション入力の予測（単純に最新を使用）
        
        predicted_input
    }
    
    /// 入力の補間
    fn interpolate_inputs(&self) -> InputData {
        let len = self.input_buffer.len();
        let input1 = &self.input_buffer[len - 2];
        let input2 = &self.input_buffer[len - 1];
        
        // 補間係数
        let t = self.compensation_settings.input_smoothing_factor;
        
        // 入力の補間
        let mut interpolated_input = input2.clone();
        
        // 移動入力の補間
        let (m1, m2) = (input1.movement, input2.movement);
        let x = m1.0 + (m2.0 - m1.0) * t;
        let y = m1.1 + (m2.1 - m1.1) * t;
        interpolated_input.movement = (x, y);
        
        // アクション入力は最新を使用（ボタン入力は通常補間しない）
        
        interpolated_input
    }
    
    /// 補正された入力を適用
    fn apply_compensated_input(&self, world: &mut World, entity: Entity, input: InputData, delta_time: f32) {
        // 入力処理コンポーネントを取得
        let input_processor = world.get_component_mut::<InputProcessor>(entity);
        
        if let Some(processor) = input_processor {
            // 補正された入力を処理
            processor.process_input(&input, delta_time);
        }
    }
}

/// 入力リソース（サンプル用）
#[derive(Resource)]
pub struct InputResource {
    current_input: InputData,
}

impl Default for InputResource {
    fn default() -> Self {
        Self {
            current_input: InputData::default(),
        }
    }
}

impl InputResource {
    /// 新しい入力リソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 現在の入力を取得
    pub fn get_current_input(&self) -> &InputData {
        &self.current_input
    }
    
    /// 入力を更新
    pub fn update_input(&mut self, input: InputData) {
        self.current_input = input;
    }
}

/// 入力処理コンポーネント（サンプル用）
pub struct InputProcessor {
    // 入力処理に必要なデータ
}

impl Component for InputProcessor {
    fn name() -> &'static str {
        "InputProcessor"
    }
}

impl InputProcessor {
    /// 新しい入力処理コンポーネントを作成
    pub fn new() -> Self {
        Self {}
    }
    
    /// 入力を処理
    pub fn process_input(&mut self, _input: &InputData, _delta_time: f32) {
        // 入力に基づいてエンティティを操作
        // 実際のゲームロジックに合わせて実装
    }
}

/// ネットワーク品質モニター
/// 
/// ネットワーク接続の品質を監視し、適応的な補正を可能にします。
#[derive(Debug)]
pub struct NetworkQualityMonitor {
    /// RTT測定値サンプル
    pub rtt_samples: VecDeque<f64>,
    /// 平均RTT
    pub avg_rtt: f64,
    /// 最小RTT
    pub min_rtt: f64,
    /// 最大RTT
    pub max_rtt: f64,
    /// パケットロス率（0.0〜1.0）
    pub packet_loss: f32,
    /// ジッター（RTTの変動）
    pub jitter: f64,
    /// 最後に受信したシーケンス番号
    pub last_sequence: u32,
    /// 欠損シーケンス番号
    pub missing_sequences: HashSet<u32>,
}

impl Default for NetworkQualityMonitor {
    fn default() -> Self {
        Self {
            rtt_samples: VecDeque::with_capacity(50),
            avg_rtt: 0.0,
            min_rtt: f64::MAX,
            max_rtt: 0.0,
            packet_loss: 0.0,
            jitter: 0.0,
            last_sequence: 0,
            missing_sequences: HashSet::new(),
        }
    }
}

impl NetworkQualityMonitor {
    /// 新しいネットワーク品質モニターを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// RTT値を更新
    pub fn update_rtt(&mut self, rtt: f64) {
        self.rtt_samples.push_back(rtt);
        
        if self.rtt_samples.len() > 50 {
            self.rtt_samples.pop_front();
        }
        
        // 統計を更新
        self.min_rtt = self.min_rtt.min(rtt);
        self.max_rtt = self.max_rtt.max(rtt);
        
        if !self.rtt_samples.is_empty() {
            let sum: f64 = self.rtt_samples.iter().sum();
            self.avg_rtt = sum / self.rtt_samples.len() as f64;
            
            // ジッター計算
            let mut jitter_sum = 0.0;
            let mut prev = None;
            
            for &sample in &self.rtt_samples {
                if let Some(p) = prev {
                    jitter_sum += ((sample - p) as f64).abs();
                }
                prev = Some(sample);
            }
            
            if self.rtt_samples.len() > 1 {
                self.jitter = jitter_sum / (self.rtt_samples.len() - 1) as f64;
            }
        }
    }
    
    /// シーケンス番号を更新
    pub fn update_sequence(&mut self, sequence: u32) {
        // 欠損シーケンス番号を検出
        if sequence > self.last_sequence + 1 {
            for seq in (self.last_sequence + 1)..sequence {
                self.missing_sequences.insert(seq);
            }
        }
        
        // シーケンス番号を記録
        if sequence > self.last_sequence {
            self.last_sequence = sequence;
        }
        
        // 受信したシーケンス番号を削除
        self.missing_sequences.remove(&sequence);
        
        // パケットロス率を計算
        if self.last_sequence > 0 {
            self.packet_loss = self.missing_sequences.len() as f32 / self.last_sequence as f32;
        }
    }
    
    /// 最適なバッファサイズを取得
    pub fn get_optimal_buffer_size(&self) -> usize {
        // ネットワーク品質に基づいて最適なバッファサイズを計算
        let base_size = 3;
        
        if self.avg_rtt < 50.0 && self.jitter < 10.0 && self.packet_loss < 0.01 {
            // 良好な接続
            base_size
        } else if self.avg_rtt < 100.0 && self.jitter < 20.0 && self.packet_loss < 0.05 {
            // 普通の接続
            base_size + 2
        } else {
            // 悪い接続
            base_size + 4
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prediction_data_default() {
        let data = PredictionData::default();
        assert_eq!(data.last_confirmed_sequence, 0);
        assert!(data.input_history.is_empty());
    }
    
    #[test]
    fn test_client_prediction_creation() {
        let system = ClientPrediction::new(50);
        assert_eq!(system.max_input_history, 50);
        assert!(system.prediction_data.is_empty());
    }
}

// ネットワークメッセージタイプの追加
#[derive(Debug, Clone)]
pub enum MessageType {
    Connect,
    ConnectResponse,
    Disconnect,
    ComponentUpdate,
    EntityCreate,
    EntityDelete,
    InputState,
    Ping,
    Pong,
    Error,
}

// ネットワークメッセージの構造体を追加
#[derive(Debug, Clone)]
pub struct NetworkMessage {
    #[allow(dead_code)]
    message_type: MessageType,
    sequence: Option<u32>,
    entity_id: Option<u32>,
    components: Option<HashMap<String, ComponentData>>,
    #[allow(dead_code)]
    timestamp: f64,
}

impl NetworkMessage {
    pub fn new(message_type: MessageType) -> Self {
        Self {
            message_type,
            sequence: None,
            entity_id: None,
            components: None,
            timestamp: Date::now(),
        }
    }
    
    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.sequence = Some(sequence);
        self
    }
    
    pub fn with_entity_id(mut self, entity_id: u32) -> Self {
        self.entity_id = Some(entity_id);
        self
    }
    
    pub fn with_components(mut self, components: HashMap<String, ComponentData>) -> Self {
        self.components = Some(components);
        self
    }
}

// ネットワーククライアントの構造体を追加
pub struct NetworkClient {
    // WebSocketクライアントの実装
}

impl NetworkClient {
    pub fn send_message(&mut self, _message: NetworkMessage) -> Result<(), JsValue> {
        // メッセージをJSONに変換してWebSocket経由で送信
        #[cfg(feature = "debug_network")]
        web_sys::console::log_1(&"メッセージ送信".into());
        
        // 実際の送信処理は別モジュールで実装
        Ok(())
    }
} 