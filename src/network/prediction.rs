//! ネットワーク予測と補正
//! 
//! このモジュールは、ネットワークレイテンシによる影響を軽減するための
//! クライアント予測とサーバー権威による補正機能を実装します。

use std::collections::{HashMap, VecDeque};
use js_sys::Date;

use super::messages::{InputData, EntitySnapshot, ComponentData};
use super::client::NetworkComponent;
use crate::ecs::{World, Entity, Component, System, Query, With, Changed, Resource};

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
    fn run(&mut self, world: &mut World, delta_time: f32) {
        // 前回の更新からの経過時間
        let now = Date::now();
        let elapsed = now - self.last_update;
        self.last_update = now;
        
        // 所有エンティティのクエリ
        // 自分が制御するエンティティに対してのみ予測を行う
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
}

impl Default for ServerReconciliation {
    fn default() -> Self {
        Self {
            client_inputs: HashMap::new(),
            last_update: Date::now(),
        }
    }
}

impl System for ServerReconciliation {
    fn run(&mut self, world: &mut World, delta_time: f32) {
        // サーバー側の処理
        // ここはサーバー上で実行されることを想定
        
        let now = Date::now();
        self.last_update = now;
        
        // クライアント所有のエンティティを検出
        // 各クライアントの入力を適用し、結果を保存
        // クライアントの予測と大きく異なる場合、修正データを送信
    }
}

impl ServerReconciliation {
    /// 新しいサーバー再調整システムを作成
    pub fn new() -> Self {
        Self {
            client_inputs: HashMap::new(),
            last_update: Date::now(),
        }
    }
    
    /// クライアントからの入力を処理
    pub fn process_client_input(&mut self, client_id: u32, input: InputData, sequence: u32) {
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