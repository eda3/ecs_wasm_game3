//! ネットワーク遅延補正システム
//! 
//! このモジュールは、ネットワーク遅延によって発生する問題を軽減するための
//! 機能を提供します。入力遅延の補正や状態の補間などの機能を実装しています。

use crate::ecs::{World, System, Resource, Entity, Resources};
use crate::network::{NetworkStatus, InputData, NetworkResource, ConnectionState};
use crate::network::messages::ComponentData;
use crate::network::network_status::BandwidthStatus;
use std::collections::{VecDeque, HashMap};
use js_sys::Date;

/// 遅延補正設定
#[derive(Debug, Clone, Resource)]
pub struct LatencyCompensationConfig {
    /// 予測係数（0.0-1.0、値が大きいほど予測を強く適用）
    pub prediction_factor: f32,
    /// 補間バッファ時間（ミリ秒）
    pub interpolation_buffer_ms: f32,
    /// 適応型補正を有効にするか
    pub enable_adaptive_compensation: bool,
    /// 最大補正係数
    pub max_correction_factor: f32,
    /// 最小RTT閾値（この値以下ではほとんど補正しない）
    pub min_rtt_threshold_ms: f32,
}

impl Default for LatencyCompensationConfig {
    fn default() -> Self {
        Self {
            prediction_factor: 0.7,
            interpolation_buffer_ms: 100.0,
            enable_adaptive_compensation: true,
            max_correction_factor: 0.95,
            min_rtt_threshold_ms: 30.0,
        }
    }
}

/// 遅延補正システム
pub struct LatencyCompensationSystem {
    /// 最後の更新時刻
    last_update: f64,
    /// 入力履歴
    input_history: VecDeque<(InputData, f64)>, // (入力, タイムスタンプ)
    /// 入力の予測モデル
    input_prediction_model: InputPredictionModel,
    /// エンティティ移動の予測履歴
    entity_prediction_history: HashMap<Entity, VecDeque<(f64, [f32; 3], [f32; 3])>>, // (時間, 位置, 速度)
    /// 適応型補正係数
    adaptive_correction_factor: f32,
    /// 設定
    config: LatencyCompensationConfig,
}

/// 入力予測モデル
struct InputPredictionModel {
    /// 入力の重み付け係数
    weight_factors: [f32; 5],
    /// 最近の入力
    recent_inputs: VecDeque<InputData>,
    /// 最後の予測結果
    last_prediction: Option<InputData>,
}

impl Default for InputPredictionModel {
    fn default() -> Self {
        Self {
            // 最新の入力ほど高い重みを持つ
            weight_factors: [0.5, 0.25, 0.15, 0.07, 0.03],
            recent_inputs: VecDeque::with_capacity(5),
            last_prediction: None,
        }
    }
}

impl InputPredictionModel {
    /// 新しい入力を追加し、予測を更新
    pub fn add_input(&mut self, input: InputData) {
        self.recent_inputs.push_front(input);
        if self.recent_inputs.len() > 5 {
            self.recent_inputs.pop_back();
        }
    }
    
    /// 入力を予測
    pub fn predict_next_input(&mut self) -> InputData {
        // 入力履歴が空の場合
        if self.recent_inputs.is_empty() {
            return InputData::default();
        }
        
        // 1つしかない場合はそれを返す
        if self.recent_inputs.len() == 1 {
            return self.recent_inputs[0].clone();
        }
        
        // 入力の加重平均を計算
        let mut weighted_x = 0.0;
        let mut weighted_y = 0.0;
        let mut total_weight = 0.0;
        
        for (i, input) in self.recent_inputs.iter().enumerate() {
            let weight = if i < self.weight_factors.len() {
                self.weight_factors[i]
            } else {
                0.01
            };
            
            weighted_x += input.x * weight;
            weighted_y += input.y * weight;
            total_weight += weight;
        }
        
        // 正規化
        let normalized_x = if total_weight > 0.0 { weighted_x / total_weight } else { 0.0 };
        let normalized_y = if total_weight > 0.0 { weighted_y / total_weight } else { 0.0 };
        
        // 直前の入力からの急激な変化を避けるためのスムージング
        let prediction = if let Some(last) = &self.last_prediction {
            let smooth_factor = 0.3; // スムージング係数（0.0-1.0）
            let smoothed_x = last.x * (1.0 - smooth_factor) + normalized_x * smooth_factor;
            let smoothed_y = last.y * (1.0 - smooth_factor) + normalized_y * smooth_factor;
            
            InputData {
                x: smoothed_x,
                y: smoothed_y,
                buttons: last.buttons.clone(), // ボタン状態は直前の入力を維持
            }
        } else {
            InputData {
                x: normalized_x,
                y: normalized_y,
                buttons: self.recent_inputs[0].buttons.clone(),
            }
        };
        
        // 予測結果を保存
        self.last_prediction = Some(prediction.clone());
        
        prediction
    }
}

impl Default for LatencyCompensationSystem {
    fn default() -> Self {
        Self {
            last_update: Date::now(),
            input_history: VecDeque::with_capacity(30),
            input_prediction_model: InputPredictionModel::default(),
            entity_prediction_history: HashMap::new(),
            adaptive_correction_factor: 0.5,
            config: LatencyCompensationConfig::default(),
        }
    }
}

impl LatencyCompensationSystem {
    /// 新しい遅延補正システムを作成
    pub fn new(config: LatencyCompensationConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }
    
    /// 入力を記録
    pub fn record_input(&mut self, input: InputData) {
        let now = Date::now();
        self.input_history.push_front((input.clone(), now));
        
        // 入力履歴が大きくなりすぎないように制限
        while self.input_history.len() > 30 {
            self.input_history.pop_back();
        }
        
        // 入力予測モデルを更新
        self.input_prediction_model.add_input(input);
    }
    
    /// 指定された時間の入力を補間
    pub fn interpolate_input_at_time(&self, time: f64) -> InputData {
        // 指定された時間の前後の入力を検索
        let mut before: Option<(usize, &(InputData, f64))> = None;
        let mut after: Option<(usize, &(InputData, f64))> = None;
        
        for (i, &(ref input, timestamp)) in self.input_history.iter().enumerate() {
            if timestamp <= time {
                before = Some((i, &(input.clone(), timestamp)));
                break;
            }
        }
        
        if let Some((i, _)) = before {
            if i > 0 {
                after = Some((i - 1, &self.input_history[i - 1]));
            }
        }
        
        match (before, after) {
            // 両方ある場合は補間
            (Some((_, &(ref before_input, before_time))), Some((_, &(ref after_input, after_time)))) => {
                let time_range = after_time - before_time;
                if time_range <= 0.0 {
                    return before_input.clone();
                }
                
                let t = (time - before_time) / time_range;
                let t = t.max(0.0).min(1.0);
                
                InputData {
                    x: before_input.x * (1.0 - t) + after_input.x * t,
                    y: before_input.y * (1.0 - t) + after_input.y * t,
                    buttons: before_input.buttons.clone(), // ボタン状態は補間しない
                }
            },
            // 前のみある場合はそれを使用
            (Some((_, &(ref input, _))), None) => {
                input.clone()
            },
            // 後のみある場合はそれを使用
            (None, Some((_, &(ref input, _)))) => {
                input.clone()
            },
            // どちらもない場合はデフォルト
            _ => InputData::default(),
        }
    }
    
    /// エンティティの位置と速度を予測履歴に記録
    pub fn record_entity_state(&mut self, entity: Entity, position: [f32; 3], velocity: [f32; 3]) {
        let now = Date::now();
        let history = self.entity_prediction_history
            .entry(entity)
            .or_insert_with(|| VecDeque::with_capacity(10));
            
        history.push_front((now, position, velocity));
        
        // 履歴が大きくなりすぎないように制限
        while history.len() > 10 {
            history.pop_back();
        }
    }
    
    /// エンティティの将来位置を予測
    pub fn predict_entity_position(&self, entity: Entity, time_ahead_ms: f64) -> Option<[f32; 3]> {
        if let Some(history) = self.entity_prediction_history.get(&entity) {
            if let Some(&(timestamp, position, velocity)) = history.front() {
                // 簡単な運動方程式に基づく予測
                let seconds_ahead = time_ahead_ms / 1000.0;
                let predicted_position = [
                    position[0] + velocity[0] * seconds_ahead as f32,
                    position[1] + velocity[1] * seconds_ahead as f32,
                    position[2] + velocity[2] * seconds_ahead as f32,
                ];
                
                return Some(predicted_position);
            }
        }
        
        None
    }
    
    /// ネットワーク状態に基づいて適応型補正係数を更新
    fn update_adaptive_correction_factor(&mut self, network_status: &NetworkStatus) {
        if !self.config.enable_adaptive_compensation {
            return;
        }
        
        // RTTに基づいて補正係数を計算
        let rtt = network_status.rtt as f32;
        
        if rtt < self.config.min_rtt_threshold_ms {
            // RTTが十分に小さい場合は補正を最小限に
            self.adaptive_correction_factor = 0.1;
        } else {
            // RTTが大きいほど補正を強くする
            let base_factor = (rtt - self.config.min_rtt_threshold_ms) / 200.0;
            let factor = base_factor.min(self.config.max_correction_factor);
            
            // 現在の値と新しい値の間をスムージング
            let smooth_factor = 0.3;
            self.adaptive_correction_factor = self.adaptive_correction_factor * (1.0 - smooth_factor) 
                + factor * smooth_factor;
        }
        
        // 帯域幅の状態に応じてさらに調整
        match network_status.bandwidth_status {
            BandwidthStatus::Poor => {
                // 悪い帯域幅では予測を強化
                self.adaptive_correction_factor *= 1.5;
            },
            BandwidthStatus::Limited => {
                // 制限された帯域幅では少し予測を強化
                self.adaptive_correction_factor *= 1.2;
            },
            _ => {}
        }
        
        // 範囲内に収める
        self.adaptive_correction_factor = self.adaptive_correction_factor.max(0.0).min(self.config.max_correction_factor);
    }
    
    /// 遅延補正を適用した入力を取得
    pub fn get_compensated_input(&self, network_status: &NetworkStatus) -> InputData {
        // 現在時刻を取得
        let now = Date::now();
        
        // 予測時間を計算（RTTの半分 + 追加バッファ）
        let prediction_time_ms = network_status.rtt as f64 / 2.0 + self.config.interpolation_buffer_ms as f64;
        
        // 将来の時間を計算
        let future_time = now + prediction_time_ms;
        
        // 履歴から将来の入力を補間
        let interpolated = self.interpolate_input_at_time(future_time);
        
        // 予測モデルからの入力を取得
        let predicted = self.input_prediction_model.predict_next_input();
        
        // 両者を適応型係数で混合
        let blended_x = interpolated.x * (1.0 - self.adaptive_correction_factor) + 
                         predicted.x * self.adaptive_correction_factor;
        let blended_y = interpolated.y * (1.0 - self.adaptive_correction_factor) + 
                         predicted.y * self.adaptive_correction_factor;
        
        InputData {
            x: blended_x,
            y: blended_y,
            buttons: interpolated.buttons.clone(), // ボタン状態は最近の入力から
        }
    }
}

impl System for LatencyCompensationSystem {
    fn run(&mut self, world: &mut World, delta_time: f32) {
        // 現在の時刻を取得
        let now = Date::now();
        self.last_update = now;
        
        // ネットワークリソースを取得
        let network_resource = match world.get_resource::<NetworkResource>() {
            Some(resource) => resource,
            None => return, // リソースがなければ何もしない
        };
        
        // 切断状態であれば何もしない
        if network_resource.connection_state == ConnectionState::Disconnected {
            return;
        }
        
        // ネットワーク状態リソースを取得
        let network_status = match world.get_resource::<NetworkStatus>() {
            Some(status) => status,
            None => return, // リソースがなければ何もしない
        };
        
        // 適応型補正係数を更新
        self.update_adaptive_correction_factor(network_status);
        
        // 自己エンティティを取得
        if let Some(self_entity) = network_resource.controlled_entity {
            // 位置と速度のコンポーネントを取得
            if let (Some(position), Some(velocity)) = (
                world.get_component::<ComponentData>(self_entity, "Position"),
                world.get_component::<ComponentData>(self_entity, "Velocity")
            ) {
                // コンポーネントデータから位置と速度を抽出
                if let (
                    ComponentData::Position { x, y, z },
                    ComponentData::Velocity { x: vx, y: vy, z: vz }
                ) = (position, velocity) {
                    // 位置と速度を記録
                    self.record_entity_state(
                        self_entity,
                        [*x, *y, *z.unwrap_or(0.0)],
                        [*vx, *vy, *vz.unwrap_or(0.0)]
                    );
                }
            }
            
            // 入力リソースを取得して補正入力を適用
            if let Some(mut input_resource) = world.get_resource_mut::<InputResource>() {
                // 現在の入力を記録
                let current_input = input_resource.get_current_input().clone();
                self.record_input(current_input);
                
                // 遅延補正入力を計算
                let compensated_input = self.get_compensated_input(network_status);
                
                // 補正入力をリソースに設定
                input_resource.set_compensated_input(compensated_input);
            }
        }
    }
}

/// 入力リソース（遅延補正用に拡張）
#[derive(Debug, Resource)]
pub struct InputResource {
    /// 現在の入力
    current_input: InputData,
    /// 遅延補正された入力
    compensated_input: Option<InputData>,
}

impl Default for InputResource {
    fn default() -> Self {
        Self {
            current_input: InputData::default(),
            compensated_input: None,
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
    
    /// 補正入力を設定
    pub fn set_compensated_input(&mut self, input: InputData) {
        self.compensated_input = Some(input);
    }
    
    /// 補正入力を取得
    pub fn get_compensated_input(&self) -> Option<&InputData> {
        self.compensated_input.as_ref()
    }
    
    /// 遅延補正の有無に基づいて適切な入力を取得
    pub fn get_effective_input(&self, use_compensation: bool) -> &InputData {
        if use_compensation {
            self.compensated_input.as_ref().unwrap_or(&self.current_input)
        } else {
            &self.current_input
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_prediction_model() {
        let mut model = InputPredictionModel::default();
        
        // 入力を追加
        model.add_input(InputData { x: 1.0, y: 0.0, buttons: vec![] });
        model.add_input(InputData { x: 1.0, y: 0.5, buttons: vec![] });
        model.add_input(InputData { x: 1.0, y: 1.0, buttons: vec![] });
        
        // 予測を実行
        let prediction = model.predict_next_input();
        
        // 予測結果を検証
        assert!(prediction.x >= 0.9 && prediction.x <= 1.1);
        assert!(prediction.y >= 0.0 && prediction.y <= 1.5);
    }
} 