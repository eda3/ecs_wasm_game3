//! ネットワーク帯域制御モジュール
//!
//! このモジュールは、ネットワーク使用量を最適化し、
//! ネットワーク状態に基づいて更新頻度を動的に調整する機能を提供します。

use crate::ecs::{World, System, Resource};
use crate::network::{NetworkStatus, NetworkResource, ConnectionState};
use crate::network::network_status::{BandwidthStatus, NetworkQuality};
use std::collections::VecDeque;
use js_sys::Date;

/// エンティティの更新優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpdatePriority {
    /// 非常に高い（常に更新）
    Critical,
    /// 高い（高頻度で更新）
    High,
    /// 中程度（通常の頻度で更新）
    Medium,
    /// 低い（低頻度でのみ更新）
    Low,
    /// 非常に低い（帯域に余裕がある場合のみ更新）
    VeryLow,
}

impl Default for UpdatePriority {
    fn default() -> Self {
        UpdatePriority::Medium
    }
}

/// 帯域制御の設定
#[derive(Debug, Clone)]
pub struct BandwidthControlConfig {
    /// 最大更新レート（1秒あたりの更新回数）
    pub max_update_rate: f32,
    /// 最小更新レート（1秒あたりの更新回数）
    pub min_update_rate: f32,
    /// 帯域幅不足時の削減率
    pub bandwidth_reduction_factor: f32,
    /// 各優先度の基本更新間隔（ミリ秒）
    pub priority_intervals: Vec<f64>,
    /// ネットワーク品質に応じた更新レート係数
    pub quality_rate_factors: Vec<f32>,
    /// エンティティあたりの最大データサイズ（バイト）
    pub max_entity_data_size: usize,
    /// 1パケットあたりの最大エンティティ数
    pub max_entities_per_packet: usize,
    /// 帯域制御の適用間隔（ミリ秒）
    pub control_interval: f64,
}

impl Default for BandwidthControlConfig {
    fn default() -> Self {
        let mut priority_intervals = Vec::new();
        priority_intervals.push(50.0);  // 20Hz
        priority_intervals.push(100.0); // 10Hz
        priority_intervals.push(200.0); // 5Hz
        priority_intervals.push(500.0); // 2Hz
        priority_intervals.push(1000.0); // 1Hz

        let mut quality_rate_factors = Vec::new();
        quality_rate_factors.push(1.2);
        quality_rate_factors.push(1.0);
        quality_rate_factors.push(0.8);
        quality_rate_factors.push(0.5);
        quality_rate_factors.push(0.3);

        Self {
            max_update_rate: 20.0,
            min_update_rate: 1.0,
            bandwidth_reduction_factor: 0.8,
            priority_intervals,
            quality_rate_factors,
            max_entity_data_size: 1024, // 1KB
            max_entities_per_packet: 20,
            control_interval: 2000.0, // 2秒ごとに調整
        }
    }
}

/// エンティティの更新スケジュール情報
#[derive(Debug, Clone)]
struct EntityUpdateInfo {
    /// エンティティID
    entity_id: u32,
    /// 更新の優先度
    priority: UpdatePriority,
    /// 基本更新間隔（ミリ秒）
    base_interval: f64,
    /// 適用される実際の更新間隔（ミリ秒）
    actual_interval: f64,
    /// 前回の更新時刻
    last_update: f64,
    /// 推定データサイズ（バイト）
    estimated_size: usize,
    /// 距離ベースの重要度係数（0.0-1.0）
    distance_factor: f32,
}

/// 帯域使用量の履歴エントリ
#[derive(Debug, Clone, Copy)]
struct BandwidthUsageEntry {
    /// タイムスタンプ
    timestamp: f64,
    /// 送信したバイト数
    bytes_sent: usize,
}

/// 帯域制御リソース
#[derive(Debug, Clone, Resource)]
pub struct BandwidthControlResource {
    /// 現在の1秒あたりの更新回数
    pub current_update_rate: f32,
    /// 目標帯域幅（Kbps）
    pub target_bandwidth: f32,
    /// 最後の制御適用時刻
    pub last_control_time: f64,
    /// 各優先度のエンティティ数
    pub entity_counts: Vec<usize>,
    /// 現在の帯域使用率（0.0-1.0）
    pub utilization_ratio: f32,
    /// 次回の自動調整までの時間（ミリ秒）
    pub next_adjustment_in: f64,
}

impl Default for BandwidthControlResource {
    fn default() -> Self {
        Self {
            current_update_rate: 10.0,
            target_bandwidth: 500.0,
            last_control_time: Date::now(),
            entity_counts: Vec::new(),
            utilization_ratio: 0.0,
            next_adjustment_in: 2000.0,
        }
    }
}

/// 帯域制御システム
pub struct BandwidthControlSystem {
    /// 設定
    config: BandwidthControlConfig,
    /// エンティティの更新情報
    entity_updates: Vec<EntityUpdateInfo>,
    /// 帯域使用量の履歴
    bandwidth_usage: VecDeque<BandwidthUsageEntry>,
    /// 前回の総送信バイト数
    last_total_bytes: usize,
    /// 前回の測定時刻
    last_measurement_time: f64,
    /// 直近の制御計算結果
    control_resource: BandwidthControlResource,
    /// 前回適用したネットワーク品質
    last_quality: NetworkQuality,
}

impl Default for BandwidthControlSystem {
    fn default() -> Self {
        let config = BandwidthControlConfig::default();
        let now = Date::now();
        
        Self {
            config,
            entity_updates: Vec::new(),
            bandwidth_usage: VecDeque::with_capacity(20),
            last_total_bytes: 0,
            last_measurement_time: now,
            control_resource: BandwidthControlResource::default(),
            last_quality: NetworkQuality::Good,
        }
    }
}

impl BandwidthControlSystem {
    /// 新しい帯域制御システムを作成
    pub fn new(config: BandwidthControlConfig) -> Self {
        let now = Date::now();
        
        Self {
            config,
            entity_updates: Vec::new(),
            bandwidth_usage: VecDeque::new(),
            last_total_bytes: 0,
            last_measurement_time: now,
            control_resource: BandwidthControlResource::default(),
            last_quality: NetworkQuality::Good,
        }
    }
    
    /// エンティティを登録または更新
    pub fn register_entity(&mut self, entity_id: u32, priority: UpdatePriority, estimated_size: usize) {
        let base_interval = self.config.priority_intervals[priority as usize];
        let now = Date::now();
        
        // 既存のエントリを更新または新規作成
        self.entity_updates.push(EntityUpdateInfo {
            entity_id,
            priority,
            base_interval,
            actual_interval: base_interval, // 初期値は基本間隔
            last_update: now,
            estimated_size,
            distance_factor: 1.0, // デフォルトは最大重要度
        });
        
        // 優先度ごとのエンティティ数を更新
        self.control_resource.entity_counts.push(0);
    }
    
    /// エンティティの登録を解除
    pub fn unregister_entity(&mut self, entity_id: u32) {
        self.entity_updates.retain(|info| info.entity_id != entity_id);
    }
    
    /// エンティティの優先度を更新
    pub fn update_entity_priority(&mut self, entity_id: u32, new_priority: UpdatePriority) {
        if let Some(info) = self.entity_updates.iter_mut().find(|info| info.entity_id == entity_id) {
            // 古い優先度のカウントを減らす
            if let Some(count) = self.control_resource.entity_counts.get_mut(info.priority as usize) {
                if *count > 0 {
                    *count -= 1;
                }
            }
            
            // 新しい優先度のカウントを増やす
            let count = self.control_resource.entity_counts.get_mut(new_priority as usize).unwrap();
            *count += 1;
            
            // 優先度と基本間隔を更新
            info.priority = new_priority;
            info.base_interval = self.config.priority_intervals[new_priority as usize];
            
            // 実際の間隔も更新（帯域制約を考慮）
            info.actual_interval = self.adjust_interval_for_bandwidth(info.base_interval);
        }
    }
    
    /// エンティティの距離係数を更新
    pub fn update_entity_distance_factor(&mut self, entity_id: u32, distance_factor: f32) {
        if let Some(info) = self.entity_updates.iter_mut().find(|info| info.entity_id == entity_id) {
            // 距離係数を0.0～1.0の範囲に制限
            info.distance_factor = distance_factor.max(0.0).min(1.0);
        }
    }
    
    /// 更新すべきエンティティを取得
    pub fn get_entities_to_update(&self) -> Vec<u32> {
        let now = Date::now();
        let mut to_update = Vec::new();
        
        for info in &self.entity_updates {
            // 重要度が高いほど、また距離が近いほど頻繁に更新
            let effective_interval = info.actual_interval / info.distance_factor.max(0.1);
            
            // 前回の更新から十分な時間が経過しているかチェック
            if now - info.last_update >= effective_interval {
                to_update.push(info.entity_id);
            }
        }
        
        to_update
    }
    
    /// エンティティの更新を記録
    pub fn record_entity_update(&mut self, entity_id: u32, bytes_sent: usize) {
        let now = Date::now();
        
        // エンティティの最終更新時刻を更新
        if let Some(info) = self.entity_updates.iter_mut().find(|info| info.entity_id == entity_id) {
            info.last_update = now;
            
            // 推定サイズを更新（移動平均）
            info.estimated_size = (info.estimated_size * 3 + bytes_sent) / 4;
        }
        
        // 帯域使用量の履歴に追加
        self.bandwidth_usage.push_back(BandwidthUsageEntry {
            timestamp: now,
            bytes_sent,
        });
        
        // 古い履歴を削除（10秒以上前）
        let cutoff_time = now - 10000.0;
        while self.bandwidth_usage.front().map_or(false, |entry| entry.timestamp < cutoff_time) {
            self.bandwidth_usage.pop_front();
        }
        
        // 帯域使用率を更新
        self.update_bandwidth_usage();
    }
    
    /// 帯域使用率を更新
    fn update_bandwidth_usage(&mut self) {
        let now = Date::now();
        
        // 直近1秒間の帯域使用量を計算
        let recent_time = now - 1000.0;
        let recent_bytes: usize = self.bandwidth_usage.iter()
            .filter(|entry| entry.timestamp >= recent_time)
            .map(|entry| entry.bytes_sent)
            .sum();
        
        // Kbpsに変換
        let recent_kbps = ((recent_bytes * 8) as f32) / 1000.0;
        
        // 利用率を計算（目標帯域幅に対する割合）
        if self.control_resource.target_bandwidth > 0.0 {
            self.control_resource.utilization_ratio = 
                (recent_kbps / self.control_resource.target_bandwidth).min(1.0);
        } else {
            self.control_resource.utilization_ratio = 0.0;
        }
    }
    
    /// 帯域制約を考慮して更新間隔を調整
    fn adjust_interval_for_bandwidth(&self, base_interval: f64) -> f64 {
        // 帯域使用率に応じて間隔を調整
        let utilization = self.control_resource.utilization_ratio;
        
        if utilization > 0.9 {
            // 帯域がほぼ飽和状態 - 大幅に間隔を広げる
            base_interval * 2.0
        } else if utilization > 0.75 {
            // 帯域使用率が高い - 間隔を広げる
            base_interval * 1.5
        } else if utilization < 0.3 {
            // 帯域に余裕がある - 間隔を狭める
            base_interval * 0.8
        } else {
            // 適切な範囲内 - そのまま
            base_interval
        }
    }
    
    /// ネットワーク品質に基づいて更新頻度を調整
    fn adjust_update_rate_for_quality(&mut self, quality: NetworkQuality) {
        // 品質に応じた係数を取得
        let factor = self.config.quality_rate_factors[quality as usize];
        
        // 基本更新レートに係数を適用
        let ideal_rate = 10.0 * factor; // 基本レート10Hzとして
        
        // 上限と下限を適用
        self.control_resource.current_update_rate = ideal_rate
            .max(self.config.min_update_rate)
            .min(self.config.max_update_rate);
    }
    
    /// 帯域制約を考慮して全エンティティの更新間隔を再計算
    fn recalculate_entity_intervals(&mut self) {
        // 現在の帯域状況に基づいて各エンティティの実際の更新間隔を調整
        for info in self.entity_updates.iter_mut() {
            info.actual_interval = self.adjust_interval_for_bandwidth(info.base_interval);
        }
    }
    
    /// 目標帯域幅をネットワーク状態に基づいて調整
    fn adjust_target_bandwidth(&mut self, status: &NetworkStatus) {
        match status.bandwidth_status {
            BandwidthStatus::Good => {
                // 良好な帯域幅 - 推定の80%を使用
                self.control_resource.target_bandwidth = status.bandwidth_kbps * 0.8;
            },
            BandwidthStatus::Adequate => {
                // 十分な帯域幅 - 推定の70%を使用
                self.control_resource.target_bandwidth = status.bandwidth_kbps * 0.7;
            },
            BandwidthStatus::Limited => {
                // 制限された帯域幅 - 推定の60%を使用
                self.control_resource.target_bandwidth = status.bandwidth_kbps * 0.6;
            },
            BandwidthStatus::Poor => {
                // 不足した帯域幅 - 推定の50%を使用
                self.control_resource.target_bandwidth = status.bandwidth_kbps * 0.5;
            },
            BandwidthStatus::Critical => {
                // 非常に制限された帯域幅 - 推定の40%を使用
                self.control_resource.target_bandwidth = status.bandwidth_kbps * 0.4;
            },
        }
    }
    
    /// 帯域制御パラメータを適用
    fn apply_bandwidth_control(&mut self, status: &NetworkStatus) {
        let now = Date::now();
        
        // 前回の制御から十分な時間が経過していない場合はスキップ
        if now - self.control_resource.last_control_time < self.config.control_interval {
            // 次回調整までの残り時間を更新
            self.control_resource.next_adjustment_in = 
                self.config.control_interval - (now - self.control_resource.last_control_time);
            return;
        }
        
        // ネットワーク品質に変化があった場合に更新レートを調整
        if status.quality != self.last_quality {
            self.adjust_update_rate_for_quality(status.quality);
            self.last_quality = status.quality;
        }
        
        // 目標帯域幅を調整
        self.adjust_target_bandwidth(status);
        
        // 全エンティティの更新間隔を再計算
        self.recalculate_entity_intervals();
        
        // 最後の制御時刻を更新
        self.control_resource.last_control_time = now;
        self.control_resource.next_adjustment_in = self.config.control_interval;
    }
    
    /// パケットのサイズを最適化
    pub fn optimize_packet_size(&self, entities: &[u32], current_size: usize) -> Vec<u32> {
        let max_size = self.config.max_entity_data_size * self.config.max_entities_per_packet;
        
        // 既にサイズが十分小さい場合はそのまま返す
        if current_size <= max_size || entities.is_empty() {
            return entities.to_vec();
        }
        
        // エンティティの重要度に基づいてソート
        let mut entity_priorities: Vec<(u32, UpdatePriority, usize)> = entities.iter()
            .filter_map(|entity_id| {
                self.entity_updates.iter().find(|info| info.entity_id == *entity_id).map(|info| {
                    (*entity_id, info.priority, info.estimated_size)
                })
            })
            .collect();
        
        // 優先度の高い順にソート
        entity_priorities.sort_by(|a, b| {
            match a.1.cmp(&b.1) {
                std::cmp::Ordering::Equal => a.2.cmp(&b.2), // サイズが小さい順
                other => other.reverse(), // 優先度が高い順
            }
        });
        
        // 最大数またはサイズに達するまでエンティティを選択
        let mut selected = Vec::new();
        let mut total_size = 0;
        
        for (entity_id, _, estimated_size) in entity_priorities {
            // 最大エンティティ数または最大サイズに達したら終了
            if selected.len() >= self.config.max_entities_per_packet || 
               total_size + estimated_size > max_size {
                break;
            }
            
            selected.push(*entity_id);
            total_size += estimated_size;
        }
        
        selected
    }
}

impl System for BandwidthControlSystem {
    fn run(&mut self, world: &mut World, _delta_time: f32) {
        // ネットワーク状態を取得
        let network_status = match world.get_resource::<NetworkStatus>() {
            Some(status) => status.clone(),
            None => return, // ネットワーク状態がなければ何もしない
        };
        
        // 帯域制御パラメータを適用
        self.apply_bandwidth_control(&network_status);
        
        // 帯域制御リソースをWorldに更新
        world.insert_resource(self.control_resource.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_priority_management() {
        let mut system = BandwidthControlSystem::default();
        
        // エンティティを登録
        system.register_entity(1, UpdatePriority::High, 500);
        system.register_entity(2, UpdatePriority::Medium, 300);
        system.register_entity(3, UpdatePriority::Low, 200);
        
        // 優先度ごとのエンティティ数を確認
        assert_eq!(system.control_resource.entity_counts.get(0), Some(&1));
        assert_eq!(system.control_resource.entity_counts.get(1), Some(&1));
        assert_eq!(system.control_resource.entity_counts.get(2), Some(&1));
        
        // 優先度を変更
        system.update_entity_priority(2, UpdatePriority::High);
        
        // 変更後のカウントを確認
        assert_eq!(system.control_resource.entity_counts.get(0), Some(&1));
        assert_eq!(system.control_resource.entity_counts.get(1), Some(&2));
        
        // エンティティを削除
        system.unregister_entity(1);
        
        // 削除後のカウントを確認
        assert_eq!(system.control_resource.entity_counts.get(0), Some(&0));
    }
    
    #[test]
    fn test_bandwidth_adjustment() {
        let mut system = BandwidthControlSystem::default();
        
        // 低い帯域幅の状態を作成
        let mut status = NetworkStatus::default();
        status.bandwidth_kbps = 200.0;
        status.bandwidth_status = BandwidthStatus::Limited;
        
        // 帯域制御を適用
        system.apply_bandwidth_control(&status);
        
        // 目標帯域幅が調整されたことを確認
        assert!((system.control_resource.target_bandwidth - 120.0).abs() < 0.1); // 200 * 0.6
    }
    
    #[test]
    fn test_packet_optimization() {
        let mut system = BandwidthControlSystem::default();
        
        // エンティティを登録
        system.register_entity(1, UpdatePriority::Critical, 100);
        system.register_entity(2, UpdatePriority::High, 200);
        system.register_entity(3, UpdatePriority::Medium, 300);
        system.register_entity(4, UpdatePriority::Low, 150);
        system.register_entity(5, UpdatePriority::VeryLow, 50);
        
        // 全エンティティを含むリストを作成
        let all_entities = vec![1, 2, 3, 4, 5];
        
        // パケットサイズを最適化
        let optimized = system.optimize_packet_size(&all_entities, 2000);
        
        // 制限に達するまでの優先度の高いエンティティが選択されていることを確認
        assert!(optimized.contains(&1)); // Critical
        assert!(optimized.contains(&2)); // High
    }
} 