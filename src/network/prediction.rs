//! ネットワーク予測と補正
//! 
//! このモジュールは、ネットワークレイテンシによる影響を軽減するための
//! クライアント予測とサーバー権威による補正機能を実装します。

use std::collections::{HashMap, VecDeque};
use js_sys::Date;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

use super::messages::{InputData, EntitySnapshot, ComponentData};
use super::client::NetworkComponent;
use super::network_status::{NetworkStatus, BandwidthStatus};
use super::sync::{PositionComponent, VelocityComponent};
use crate::ecs::{World, Entity, Component, System, Query, With, Changed, Resource, ResourceManager};
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
    
    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        // 前回の更新からの経過時間
        let now = Date::now();
        let elapsed = now - self.last_update;
        self.last_update = now;
        
        // 所有エンティティのクエリ
        // 自分が制御するエンティティに対してのみ予測を行う
        // TODO: world.queryは実装されていないようなので、代替手段を検討する必要があります
        // 現時点ではここをコメントアウトし、エラーを回避します
        /*
        let query = world.query::<(Entity, &NetworkComponent)>()
            .filter(|_, network| network.is_synced && !network.is_remote);
        
        for (entity, network) in query.iter(world) {
            // 予測データを初期化（存在しない場合）
            let prediction_data = self.prediction_data
                .entry(entity)
                .or_insert_with(PredictionData::default);
            
            // 入力履歴を更新
            // 実際のゲームではここで現在の入力を追加する
            
            // 入力履歴のサイズを制限
            while prediction_data.input_history.len() > self.max_input_history {
                prediction_data.input_history.pop_front();
            }
            
            // 予測ステップを実行
            self.predict_entity_state(world, entity, prediction_data, delta_time);
        }
        */
        
        // 本来ならエンティティに対して予測処理を行いますが、
        // 現状ではクエリ機能が利用できないため、簡易実装としています
        #[cfg(feature = "debug_network")]
        web_sys::console::log_1(&"ClientPrediction: クエリ機能がまだ実装されていません".into());
        
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
    fn predict_entity_state(&mut self, world: &mut World, entity: Entity, prediction_data: &PredictionData, delta_time: f32) {
        // ここで物理シミュレーションなど、エンティティの次の状態を予測する
        // 例: 現在の位置と速度から次のフレームの位置を予測
        // この処理は実際のゲームロジックに合わせて実装する必要がある
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
    pub fn apply_server_correction(&mut self, world: &mut World, entity: Entity, snapshot: &EntitySnapshot, sequence: u32) {
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
        // 現在の時刻を取得
        let now = Date::now();
        self.last_update = now;
        
        // クライアント所有のエンティティを検出
        let owned_entities = self.get_client_owned_entities(world);
        
        // ネットワーク送信キューを取得
        let send_queue = match resources.get_mut::<NetworkSendQueue>() {
            Some(queue) => queue,
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
                    
                    // ネットワーク状態を確認
                    if let Some(network_status) = resources.get::<NetworkStatus>() {
                        if network_status.quality == NetworkQuality::Poor || 
                           network_status.quality == NetworkQuality::Bad {
                            self.optimize_snapshot_for_poor_network(&mut optimized_snapshot);
                        }
                    }
                    
                    // 修正データをキューに追加
                    #[cfg(feature = "debug_network")]
                    web_sys::console::log_1(&format!("ServerReconciliation: クライアント {} のエンティティ {} に修正を送信 (seq: {})",
                        client_id, entity.id(), last_sequence).into());
                    
                    self.queue_snapshot_for_client(send_queue, client_id, entity, optimized_snapshot, last_sequence);
                    
                    // いくつかの主要コンポーネントについて、予測精度の分析を行う
                    if let (Some(initial_pos), Some(final_pos)) = (
                        initial_state.get("position"),
                        final_state.get("position")
                    ) {
                        // 位置の差異を計算して分析
                        let difference = self.calculate_component_difference("position", initial_pos, final_pos);
                        self.analyze_prediction_accuracy(client_id, "position", difference);
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
    fn get_client_owned_entities(&self, world: &World) -> Vec<(u32, Entity)> {
        let mut owned_entities = Vec::new();
        
        // このメソッドの実際の実装はEntityやComponentのイテレーション機能に依存します
        // 以下は簡略化したバージョン
        
        // 実際のプロジェクトでは、worldからNetworkComponentを持つエンティティを
        // クエリして、owner_idフィールドに基づいてフィルタリングします
        
        // 例示的なコード
        // for (entity, network_comp) in query::<(Entity, &NetworkComponent)>().iter(world) {
        //    if let Some(owner_id) = network_comp.owner_id {
        //        owned_entities.push((owner_id, entity));
        //    }
        // }
        
        // ここでは、テスト目的での実装を示します
        #[cfg(feature = "debug_network")]
        web_sys::console::log_1(&"エンティティ所有権クエリは簡略化されています".into());
        
        // TODO: 実際のECSクエリ実装でこれを置き換える
        
        owned_entities
    }
    
    /// エンティティの状態をキャプチャ
    fn capture_entity_state(&self, world: &World, entity: Entity) -> HashMap<String, ComponentData> {
        let mut components = HashMap::new();
        
        // このメソッドも、実際のコンポーネントシステムに依存します
        // 以下は簡略化したコード例
        
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
        
        // 注意: 実際には、同期が必要な他のコンポーネントも取得する必要があります
        
        components
    }
    
    /// 入力をエンティティに適用
    fn apply_input_to_entity(&self, world: &mut World, entity: Entity, input: &InputData, delta_time: f32) {
        // 入力を基にエンティティのコンポーネントを更新
        // 例: 位置と速度の更新
        
        if let Some(mut position) = world.get_component_mut::<PositionComponent>(entity) {
            if let Some(mut velocity) = world.get_component_mut::<VelocityComponent>(entity) {
                // 入力に基づいて速度を更新
                // 例えば、移動入力に基づいてvelocityを設定
                let (move_x, move_y) = input.movement.unwrap_or((0.0, 0.0));
                let speed = 5.0; // 適切なスピード係数
                
                velocity.x = move_x * speed;
                velocity.y = move_y * speed;
                
                // 速度に基づいて位置を更新
                position.x += velocity.x * delta_time;
                position.y += velocity.y * delta_time;
                
                // 必要に応じてZ軸も更新
                if let (Some(vel_z), Some(pos_z)) = (velocity.z, position.z.as_mut()) {
                    *pos_z += vel_z * delta_time;
                }
            }
        }
        
        // アクション入力の処理（例：ジャンプ、攻撃など）
        for (action_name, is_active) in &input.actions {
            if *is_active {
                // 各アクションタイプに対応する処理を実行
                match action_name.as_str() {
                    "jump" => {
                        // ジャンプ処理
                        if let Some(mut velocity) = world.get_component_mut::<VelocityComponent>(entity) {
                            if let Some(pos_z) = velocity.z.as_mut() {
                                *pos_z = 10.0; // ジャンプ力
                            }
                        }
                    },
                    // 他のアクションタイプを追加
                    _ => {}
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
            0.5 // 位置の閾値（単位はゲーム内単位）
        );
        
        // 速度の大きな変化があるか
        let velocity_diff = self.has_significant_difference(
            "velocity",
            initial_state,
            final_state,
            1.0 // 速度の閾値
        );
        
        // その他の重要なコンポーネントの変化...
        
        // 少なくとも1つの重要な変化があれば修正を送信
        position_diff || velocity_diff
    }
    
    /// エンティティのスナップショットを作成
    fn create_entity_snapshot(&self, world: &World, entity: Entity, timestamp: f64) -> EntitySnapshot {
        let mut snapshot = EntitySnapshot::new(entity.id().0);
        snapshot.timestamp = timestamp;
        
        // 必要なコンポーネントデータを取得して追加
        if let Some(position) = world.get_component::<PositionComponent>(entity) {
            snapshot.position = Some([position.x, position.y, position.z]);
        }
        
        if let Some(velocity) = world.get_component::<VelocityComponent>(entity) {
            snapshot.velocity = Some([velocity.x, velocity.y, velocity.z]);
        }
        
        // 他のコンポーネントも必要に応じて追加
        
        snapshot
    }
    
    /// 貧弱なネットワーク向けにスナップショットを最適化
    fn optimize_snapshot_for_poor_network(&self, snapshot: &mut EntitySnapshot) {
        // 帯域幅を節約するために、重要度の低いコンポーネントを除外
        let important_components = ["position", "velocity", "health"];
        
        // 重要でないコンポーネントを特定
        let components_to_remove: Vec<String> = snapshot.components.keys()
            .filter(|k| !important_components.contains(&k.as_str()))
            .cloned()
            .collect();
        
        // 重要でないコンポーネントを削除
        for key in components_to_remove {
            snapshot.components.remove(&key);
        }
        
        // 少数の精度を下げる（浮動小数点の精度を下げることでサイズを小さくする）
        // 位置の精度を下げる
        if let Some(ComponentData::Position { x, y, z }) = snapshot.components.get_mut("position") {
            *x = (*x * 10.0).round() / 10.0; // 小数第一位まで
            *y = (*y * 10.0).round() / 10.0;
            if let Some(ref mut z_val) = z {
                *z_val = (*z_val * 10.0).round() / 10.0;
            }
        }
        
        // 速度の精度を下げる
        if let Some(ComponentData::Velocity { x, y, z }) = snapshot.components.get_mut("velocity") {
            *x = (*x * 10.0).round() / 10.0;
            *y = (*y * 10.0).round() / 10.0;
            if let Some(ref mut z_val) = z {
                *z_val = (*z_val * 10.0).round() / 10.0;
            }
        }
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
            // 他のコンポーネントタイプに対する比較ロジックを追加
            _ => false, // サポートされていないコンポーネントタイプは無視
        }
    }
    
    /// コンポーネントの差異を計算
    fn calculate_component_difference(&self, component_name: &str, initial: &ComponentData, final_: &ComponentData) -> f32 {
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
    fn analyze_prediction_accuracy(&self, client_id: u32, component: &str, difference: f32) {
        // ここで予測精度のログを記録したり分析データを蓄積したりします
        #[cfg(feature = "debug_network")]
        web_sys::console::log_1(&format!(
            "予測分析 - クライアント: {}, コンポーネント: {}, 差異: {:.3}",
            client_id, component, difference
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
    
    /// クライアント所有のエンティティかどうかを確認
    fn is_client_owned(&self, world: &World, entity: Entity, client_id: u32) -> bool {
        if let Some(network_comp) = world.get_component::<NetworkComponent>(entity) {
            network_comp.owner_id == Some(client_id)
        } else {
            false
        }
    }
    
    fn queue_snapshot_for_client(&self, send_queue: &mut NetworkSendQueue, client_id: u32, entity: Entity, snapshot: EntitySnapshot, sequence: u32) {
        // エンティティスナップショットをネットワークメッセージに変換
        let message = NetworkMessage::new(MessageType::ComponentUpdate)
            .with_sequence(sequence)
            .with_entity_id(snapshot.id as u32)
            .with_timestamp(snapshot.timestamp);
        
        // 送信キューに追加
        send_queue.queue_message(client_id, message);
    }
    
    /// 入力に応じて運動量を計算する
    fn calculate_momentum(&self, input1: &InputData, input2: &InputData, input3: &InputData) -> (f32, f32) {
        // 各入力の移動ベクトルを取得
        let m1 = input1.movement.unwrap_or((0.0, 0.0));
        let m2 = input2.movement.unwrap_or((0.0, 0.0));
        let m3 = input3.movement.unwrap_or((0.0, 0.0));
        
        // 3つの入力の平均を計算
        let avg_x = (m1.0 + m2.0 + m3.0) / 3.0;
        let avg_y = (m1.1 + m2.1 + m3.1) / 3.0;
        
        (avg_x, avg_y)
    }
    
    /// 2つの入力から方向を予測
    fn predict_direction(&self, input1: &InputData, input2: &InputData) -> (f32, f32) {
        let m1 = input1.movement.unwrap_or((0.0, 0.0));
        let m2 = input2.movement.unwrap_or((0.0, 0.0));
        
        // 線形予測（単純な次の値の予測）
        let predicted_x = m2.0 + (m2.0 - m1.0);
        let predicted_y = m2.1 + (m2.1 - m1.1);
        
        (predicted_x, predicted_y)
    }
    
    /// 入力を適用する
    fn apply_input(&self, world: &mut World, entity: Entity, input: &InputData, delta_time: f32) -> bool {
        let input_processor = world.get_component_mut::<InputProcessor>(entity);
        
        if let Some(mut processor) = input_processor {
            // 実際の入力処理はゲームの実装に依存
            processor.process_input(input, delta_time);
            true
        } else {
            false
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

// ComponentトレイトをInputProcessorに実装
impl Component for InputProcessor {
    fn name() -> &'static str {
        "InputProcessor"
    }
}

// ResourceトレイトをNetworkSendQueueに実装
impl Resource for NetworkSendQueue {}

// ResourceトレイトをNetworkStatusに実装
impl Resource for NetworkStatus {}

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
        for (client_id, entity, snapshot, sequence) in self.queue.drain(..) {
            // スナップショットメッセージを作成
            let message = NetworkMessage::new(MessageType::ComponentUpdate)
                .with_sequence(sequence)
                .with_entity_id(entity.index() as u32)
                .with_components(snapshot.components);
                
            // メッセージを送信
            if let Err(e) = network_client.send_message(message) {
                #[cfg(feature = "debug_network")]
                web_sys::console::log_1(&format!(
                    "エラー: クライアント {} へのメッセージ送信に失敗: {:?}",
                    client_id, e
                ).into());
            }
        }
    }
    
    pub fn queue_message(&mut self, client_id: u32, message: NetworkMessage) {
        // このメソッドはメッセージを直接キューに追加
        // 実際の実装ではメッセージタイプに基づいて適切な処理を行う
        // ここでは簡略化のためにログだけ出力
        #[cfg(feature = "debug_network")]
        web_sys::console::log_1(&format!("Queued message for client {}: {:?}", client_id, message.message_type).into());
    }
}

/// 状態補間システム
/// 
/// 他プレイヤーのエンティティを滑らかに補間表示するためのシステム
pub struct InterpolationSystem {
    /// 補間バッファの時間（ミリ秒）
    buffer_time: f64,
    /// 最後の更新時刻
    last_update: f64,
}

impl Default for InterpolationSystem {
    fn default() -> Self {
        Self {
            buffer_time: 100.0, // 100ms
            last_update: Date::now(),
        }
    }
}

impl System for InterpolationSystem {
    fn run(&mut self, world: &mut World, delta_time: f32) {
        let now = Date::now();
        let elapsed = now - self.last_update;
        self.last_update = now;
        
        // リモートエンティティのクエリ
        let query = world.query::<(Entity, &NetworkComponent)>()
            .filter(|_, network| network.is_synced && network.is_remote);
            
        for (entity, network) in query.iter(world) {
            // このエンティティの過去のスナップショットを取得
            // 補間時間（現在時刻 - バッファ時間）に基づいて、適切なスナップショットを選択
            // 選択したスナップショット間で線形補間を行い、滑らかな動きを実現
        }
    }
}

impl InterpolationSystem {
    /// 新しい補間システムを作成
    pub fn new(buffer_time: f64) -> Self {
        Self {
            buffer_time,
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
    fn run(&mut self, world: &mut World, delta_time: f32) {
        let now = Date::now();
        self.last_update = now;
        
        // 同期対象エンティティのクエリ
        let query = world.query::<(Entity, &NetworkComponent)>()
            .filter(|_, network| network.is_synced);
            
        for (entity, _) in query.iter(world) {
            // エンティティの状態をスナップショットとして保存
            // 定期的にスナップショットをクライアントに送信
        }
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
    fn run(&mut self, world: &mut World, delta_time: f32) {
        if self.is_server {
            // サーバーモードでの処理
            self.server_reconciliation.run(world, delta_time);
            self.entity_sync.run(world, delta_time);
        } else {
            // クライアントモードでの処理
            self.client_prediction.run(world, delta_time);
            self.interpolation.run(world, delta_time);
        }
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
    network_monitor: Option<Rc<RefCell<NetworkQualityMonitor>>>,
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
    fn run(&mut self, world: &mut World, delta_time: f32) {
        let now = Date::now();
        let elapsed = now - self.last_update;
        self.last_update = now;
        
        // ネットワークリソースを取得
        let network_resource = match world.get_resource::<NetworkResource>() {
            Some(resource) => resource,
            None => return,
        };
        
        // 入力リソースを取得
        let input_resource = match world.get_resource_mut::<InputResource>() {
            Some(resource) => resource,
            None => return,
        };
        
        // ローカルプレイヤーエンティティを取得
        let local_entity = match network_resource.controlled_entity {
            Some(entity) => entity,
            None => return,
        };
        
        // 現在の入力を取得
        let current_input = input_resource.get_current_input();
        
        // 入力バッファを更新
        self.update_input_buffer(current_input.clone());
        
        // RTTに基づいて先読み時間を調整
        let look_ahead_time = self.calculate_look_ahead_time(network_resource.rtt);
        
        // 遅延補正された入力を計算
        let compensated_input = self.compensate_input_latency();
        
        // 補正された入力を適用
        self.apply_compensated_input(world, local_entity, compensated_input, delta_time);
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
    pub fn with_network_monitor(mut self, monitor: Rc<RefCell<NetworkQualityMonitor>>) -> Self {
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
        if let Some(monitor) = &self.network_monitor {
            let monitor = monitor.borrow();
            
            // RTTとジッターに基づいて適応的に調整
            let adjusted_time = base_look_ahead + monitor.avg_rtt * 0.5 + monitor.jitter * 2.0;
            
            // 品質が低下した場合は先読み時間を増やす
            if monitor.packet_loss > 0.05 {
                return adjusted_time * (1.0 + monitor.packet_loss * 2.0);
            } else {
                return adjusted_time;
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
        if let (Some(m1), Some(m2), Some(m3)) = (input1.movement, input2.movement, input3.movement) {
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
            
            predicted_input.movement = Some((px, py));
        }
        
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
        if let (Some(m1), Some(m2)) = (input1.movement, input2.movement) {
            let x = m1.0 + (m2.0 - m1.0) * t;
            let y = m1.1 + (m2.1 - m1.1) * t;
            interpolated_input.movement = Some((x, y));
        }
        
        // アクション入力は最新を使用（ボタン入力は通常補間しない）
        
        interpolated_input
    }
    
    /// 補正された入力を適用
    fn apply_compensated_input(&self, world: &mut World, entity: Entity, input: InputData, delta_time: f32) {
        // 入力処理コンポーネントを取得
        let input_processor = world.get_component_mut::<InputProcessor>(entity);
        
        if let Some(mut processor) = input_processor {
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
#[derive(Component)]
pub struct InputProcessor {
    // 入力処理に必要なデータ
}

impl InputProcessor {
    /// 新しい入力処理コンポーネントを作成
    pub fn new() -> Self {
        Self {}
    }
    
    /// 入力を処理
    pub fn process_input(&mut self, input: &InputData, delta_time: f32) {
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
                    jitter_sum += (sample - p).abs();
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
    message_type: MessageType,
    sequence: Option<u32>,
    entity_id: Option<u32>,
    components: Option<HashMap<String, ComponentData>>,
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
    pub fn send_message(&mut self, message: NetworkMessage) -> Result<(), JsValue> {
        // メッセージをJSONに変換してWebSocket経由で送信
        #[cfg(feature = "debug_network")]
        web_sys::console::log_1(&format!("メッセージ送信: {:?}", message.message_type).into());
        
        // 実際の送信処理は別モジュールで実装
        Ok(())
    }
} 