//! ネットワークメッセージ圧縮システム
//! 
//! このモジュールは、エンティティスナップショットに対して圧縮アルゴリズムを適用し、
//! ネットワーク帯域幅を節約するためのシステムを実装します。

use crate::ecs::{System, World, Resource, ResourceManager};
use crate::ecs::SystemPriority;
use super::sync::MessageCompressor;
use super::messages::EntitySnapshot;
use super::sync::DefaultMessageCompressor;
use wasm_bindgen::JsValue;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// ネットワークメッセージの圧縮を処理するシステム
pub struct NetworkCompressionSystem {
    /// 圧縮アルゴリズム
    compressor: Box<dyn MessageCompressor>,
    /// 帯域幅使用状況
    bandwidth_usage: BandwidthUsage,
    /// 適応モード
    adaptive_mode: AdaptiveMode,
    /// エンティティ優先度マップ
    entity_priorities: HashMap<u64, EntityPriority>,
}

/// 帯域幅使用状況の追跡
#[derive(Debug, Clone)]
pub struct BandwidthUsage {
    /// 最近10秒間の送信データ量（バイト）
    recent_bytes_sent: Vec<(Instant, usize)>,
    /// 最近10秒間の受信データ量（バイト）
    recent_bytes_received: Vec<(Instant, usize)>,
    /// 直近のピーク帯域幅（バイト/秒）
    peak_bandwidth: f32,
    /// 利用可能な帯域幅の見積もり（バイト/秒）
    estimated_available_bandwidth: f32,
    /// 帯域幅の利用目標（0.0〜1.0）
    target_usage_ratio: f32,
}

/// エンティティ更新の優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityPriority {
    /// 最高優先度（プレイヤー自身など）
    Critical,
    /// 高優先度（近くのプレイヤーなど）
    High,
    /// 標準優先度
    Normal,
    /// 低優先度（遠くの物体など）
    Low,
    /// 極低優先度（遠景など）
    VeryLow,
}

/// 適応モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveMode {
    /// 固定モード（帯域幅に関係なく常に同じ圧縮率）
    Fixed,
    /// 自動調整モード（帯域幅に応じて圧縮率を動的に調整）
    Auto,
    /// 帯域幅優先モード（帯域幅を優先し、必要に応じて大幅に圧縮）
    BandwidthPriority,
    /// 品質優先モード（品質を優先し、最小限の圧縮のみ適用）
    QualityPriority,
}

/// デバッグモードリソース
#[derive(Debug, Clone)]
pub struct DebugMode {
    /// デバッグモードが有効か
    pub enabled: bool,
    /// デバッグ情報を表示するか
    pub show_debug_info: bool,
    /// 帯域幅使用状況をログ出力するか
    pub log_bandwidth: bool,
}

impl Resource for DebugMode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl DebugMode {
    /// 新しいデバッグモードを作成
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            show_debug_info: enabled,
            log_bandwidth: enabled,
        }
    }
}

impl Default for NetworkCompressionSystem {
    fn default() -> Self {
        // デフォルトの圧縮設定：位置=2桁、回転=2桁、速度=1桁
        let default_compressor = DefaultMessageCompressor::new();
        
        Self {
            compressor: Box::new(default_compressor),
            bandwidth_usage: BandwidthUsage {
                recent_bytes_sent: Vec::new(),
                recent_bytes_received: Vec::new(),
                peak_bandwidth: 5000.0, // 初期値: 5KB/秒
                estimated_available_bandwidth: 10000.0, // 初期値: 10KB/秒
                target_usage_ratio: 0.8, // 初期値: 帯域幅の80%まで使用
            },
            adaptive_mode: AdaptiveMode::Auto,
            entity_priorities: HashMap::new(),
        }
    }
}

impl NetworkCompressionSystem {
    /// 新しいネットワーク圧縮システムを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// カスタム圧縮アルゴリズムを設定
    pub fn with_compressor<T: MessageCompressor + 'static>(mut self, compressor: T) -> Self {
        self.compressor = Box::new(compressor);
        self
    }
    
    /// 適応モードを設定
    pub fn set_adaptive_mode(&mut self, mode: AdaptiveMode) {
        self.adaptive_mode = mode;
    }
    
    /// エンティティの優先度を設定
    pub fn set_entity_priority(&mut self, entity_id: u64, priority: EntityPriority) {
        self.entity_priorities.insert(entity_id, priority);
    }
    
    /// EntitySnapshotをLocalEntitySnapshotに変換
    fn convert_to_local_snapshot(&self, snapshot: &EntitySnapshot) -> super::sync::LocalEntitySnapshot {
        let mut local = super::sync::LocalEntitySnapshot::new(
            snapshot.entity_id as u64,
            snapshot.timestamp
        );
        
        // 位置コンポーネントを変換
        if let Some(pos) = snapshot.components.get("Position") {
            if let super::messages::ComponentData::Position { x, y, z } = pos {
                let position = [*x, *y, z.unwrap_or(0.0)];
                local.position = Some(position);
            }
        }
        
        // 速度コンポーネントを変換
        if let Some(vel) = snapshot.components.get("Velocity") {
            if let super::messages::ComponentData::Velocity { x, y, z } = vel {
                let velocity = [*x, *y, z.unwrap_or(0.0)];
                local.velocity = Some(velocity);
            }
        }
        
        // 回転コンポーネントを変換
        if let Some(rot) = snapshot.components.get("Rotation") {
            if let super::messages::ComponentData::Rotation { angle } = rot {
                // 単一の角度から4次元クォータニオンに変換
                // 簡略化のため、単純に角度をw成分に設定
                let rotation = [0.0, 0.0, 0.0, *angle];
                local.rotation = Some(rotation);
            }
        }
        
        // 所有者IDを設定
        if let Some(owner_id) = snapshot.owner_id {
            let mut extra_data = HashMap::new();
            extra_data.insert("owner_id".to_string(), serde_json::to_value(owner_id).unwrap());
            local.extra_data = Some(extra_data);
        }
        
        local
    }
    
    /// LocalEntitySnapshotをEntitySnapshotに戻す変換
    fn convert_from_local_snapshot(&self, local: &super::sync::LocalEntitySnapshot) -> EntitySnapshot {
        let mut snapshot = EntitySnapshot::new(
            local.id as u32,
            local.timestamp
        );
        
        // 位置データを変換
        if let Some(position) = &local.position {
            let pos_component = super::messages::ComponentData::Position {
                x: position[0],
                y: position[1],
                z: if position[2] != 0.0 { Some(position[2]) } else { None }
            };
            snapshot.add_component("Position", pos_component);
        }
        
        // 速度データを変換
        if let Some(velocity) = &local.velocity {
            let vel_component = super::messages::ComponentData::Velocity {
                x: velocity[0],
                y: velocity[1],
                z: if velocity[2] != 0.0 { Some(velocity[2]) } else { None }
            };
            snapshot.add_component("Velocity", vel_component);
        }
        
        // 回転データを変換
        if let Some(rotation) = &local.rotation {
            let rot_component = super::messages::ComponentData::Rotation {
                angle: rotation[3]
            };
            snapshot.add_component("Rotation", rot_component);
        }
        
        // 所有者IDを設定
        if let Some(extra) = &local.extra_data {
            if let Some(owner_id) = extra.get("owner_id") {
                if let Ok(id) = serde_json::from_value::<u32>(owner_id.clone()) {
                    snapshot.set_owner(id);
                }
            }
        }
        
        snapshot
    }
    
    /// スナップショットを圧縮
    pub fn compress_snapshot(&self, snapshot: &EntitySnapshot) -> EntitySnapshot {
        // エンティティの優先度に基づいて圧縮レベルを調整
        let priority = self.entity_priorities.get(&(snapshot.entity_id as u64))
            .unwrap_or(&EntityPriority::Normal);
            
        // 優先度に基づいた圧縮処理（優先度が高いほど圧縮を軽くする）
        match priority {
            EntityPriority::Critical => {
                // クリティカルなエンティティは圧縮しない
                snapshot.clone()
            },
            EntityPriority::High => {
                // 高優先度は軽い圧縮
                if self.adaptive_mode == AdaptiveMode::QualityPriority {
                    // 品質優先モードでは圧縮しない
                    snapshot.clone()
                } else {
                    // EntitySnapshot → LocalEntitySnapshot → 圧縮 → EntitySnapshot
                    let local_snapshot = self.convert_to_local_snapshot(snapshot);
                    let compressed_local = self.compressor.compress(&local_snapshot);
                    self.convert_from_local_snapshot(&compressed_local)
                }
            },
            EntityPriority::Normal => {
                // 通常優先度は現在のモードに従う
                if self.adaptive_mode == AdaptiveMode::Fixed {
                    snapshot.clone()
                } else {
                    // EntitySnapshot → LocalEntitySnapshot → 圧縮 → EntitySnapshot
                    let local_snapshot = self.convert_to_local_snapshot(snapshot);
                    let compressed_local = self.compressor.compress(&local_snapshot);
                    self.convert_from_local_snapshot(&compressed_local)
                }
            },
            EntityPriority::Low => {
                // 低優先度は常に圧縮
                let local_snapshot = self.convert_to_local_snapshot(snapshot);
                let compressed_local = self.compressor.compress(&local_snapshot);
                
                // 適応モードが重いほど、さらに情報を間引く
                if self.adaptive_mode == AdaptiveMode::QualityPriority {
                    // 最大圧縮モードでは速度情報を完全に削除
                    let mut extra_compressed = compressed_local.clone();
                    extra_compressed.velocity = None;
                    // 遠くのオブジェクトは追加データも削除
                    extra_compressed.extra_data = None;
                    self.convert_from_local_snapshot(&extra_compressed)
                } else {
                    self.convert_from_local_snapshot(&compressed_local)
                }
            },
            EntityPriority::VeryLow => {
                // 極低優先度は圧縮しない
                snapshot.clone()
            }
        }
    }
    
    /// 帯域幅使用状況を更新
    pub fn update_bandwidth_usage(&mut self, bytes_sent: usize, bytes_received: usize) {
        self.bandwidth_usage.recent_bytes_sent.push((Instant::now(), bytes_sent));
        self.bandwidth_usage.recent_bytes_received.push((Instant::now(), bytes_received));
        self.bandwidth_usage.cleanup_old_data();
    }
    
    /// 現在の帯域使用状況を取得
    pub fn get_bandwidth_usage(&self) -> &BandwidthUsage {
        &self.bandwidth_usage
    }
    
    /// 帯域幅使用状況に基づいて圧縮レベルを自動調整
    fn adapt_compression_level(&mut self) {
        // 帯域幅の閾値 (バイト/秒)
        const LOW_BANDWIDTH_THRESHOLD: f32 = 5_000.0;    // 5 KB/s
        const MEDIUM_BANDWIDTH_THRESHOLD: f32 = 20_000.0; // 20 KB/s
        const HIGH_BANDWIDTH_THRESHOLD: f32 = 50_000.0;   // 50 KB/s
        
        let bytes_per_second = self.bandwidth_usage.calculate_current_bandwidth();
        
        // 現在の帯域使用量に基づいて適応モードを更新
        let new_mode = if bytes_per_second < LOW_BANDWIDTH_THRESHOLD {
            // 帯域が非常に制限されている場合、最大圧縮
            AdaptiveMode::QualityPriority
        } else if bytes_per_second < MEDIUM_BANDWIDTH_THRESHOLD {
            // 帯域が制限されている場合、重い圧縮
            AdaptiveMode::QualityPriority
        } else if bytes_per_second < HIGH_BANDWIDTH_THRESHOLD {
            // 通常の帯域幅、中程度の圧縮
            AdaptiveMode::Auto
        } else {
            // 帯域幅が十分ある場合、軽い圧縮
            AdaptiveMode::Fixed
        };
        
        // モードが変わった場合のみ更新
        if new_mode != self.adaptive_mode {
            self.set_adaptive_mode(new_mode);
            println!("帯域幅使用状況に基づいて圧縮モードを調整: {:?}, 帯域={:.1}KB/s", 
                new_mode, bytes_per_second / 1000.0);
        }
    }

    /// 10秒以上経過したデータを削除
    fn cleanup_old_data(&mut self) {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(10);
        
        self.bandwidth_usage.recent_bytes_sent.retain(|(time, _)| *time >= cutoff);
        self.bandwidth_usage.recent_bytes_received.retain(|(time, _)| *time >= cutoff);
    }

    /// 現在の帯域幅使用量を計算（バイト/秒）
    pub fn calculate_current_bandwidth(&self) -> f32 {
        let now = Instant::now();
        let window_start = now - Duration::from_secs(1);
        
        // 直近1秒間のデータ量を集計
        let bytes_in_last_second: usize = self.bandwidth_usage.recent_bytes_sent
            .iter()
            .filter(|(time, _)| *time >= window_start)
            .map(|(_, size)| size)
            .sum();
            
        bytes_in_last_second as f32
    }

    /// 利用可能な帯域幅を設定
    pub fn set_available_bandwidth(&mut self, bandwidth_bytes_per_sec: f32) {
        self.bandwidth_usage.estimated_available_bandwidth = bandwidth_bytes_per_sec;
    }

    /// 帯域幅の利用目標を設定
    pub fn set_target_usage_ratio(&mut self, ratio: f32) {
        self.bandwidth_usage.target_usage_ratio = ratio.max(0.1).min(0.95);
    }
}

impl System for NetworkCompressionSystem {
    fn name(&self) -> &'static str {
        "NetworkCompressionSystem"
    }
    
    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        // 現在の時間を取得
        let current_time = js_sys::Date::now();
        
        // 処理すべきエンティティがあればここで圧縮処理を実行
        // 実際の実装では、このシステムは他のネットワークシステムと連携して動作します
        
        // 性能ログ出力（デバッグ用）
        if let Some(mode) = resources.get::<DebugMode>() {
            if mode.show_debug_info {
                println!("NetworkCompressionSystem: 現在のモード={:?}, 帯域={:.1}KB/s", 
                    self.adaptive_mode,
                    self.bandwidth_usage.calculate_current_bandwidth() / 1000.0);
            }
        }
        
        Ok(())
    }

    fn phase(&self) -> crate::ecs::SystemPhase {
        crate::ecs::SystemPhase::Update
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(0) // 標準優先度
    }
}

/// ユニットテスト
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compression_system() {
        let mut system = NetworkCompressionSystem::new();
        
        // スナップショットを作成
        let snapshot = EntitySnapshot::new(1)
            .with_position([123.45678, 456.78912, 789.12345])
            .with_rotation([0.12345, 0.23456, 0.34567, 0.98765])
            .with_velocity([10.5432, 20.6543, 30.7654]);
            
        // 圧縮実行
        let compressed = system.compress_snapshot(&snapshot);
        
        // 圧縮結果を検証
        if let Some(pos) = compressed.position {
            assert_eq!(pos[0], 123.46); // 小数点2桁に丸められる
            assert_eq!(pos[1], 456.79);
            assert_eq!(pos[2], 789.12);
        }
        
        if let Some(vel) = compressed.velocity {
            assert_eq!(vel[0], 10.5); // 小数点1桁に丸められる
            assert_eq!(vel[1], 20.7);
            assert_eq!(vel[2], 30.8);
        }
        
        // 適応モードを変更してテスト
        system.set_adaptive_mode(AdaptiveMode::QualityPriority);
        let max_compressed = system.compress_snapshot(&snapshot);
        
        // 最大圧縮では小数点以下がすべて0に丸められる
        if let Some(pos) = max_compressed.position {
            assert_eq!(pos[0], 123.0);
            assert_eq!(pos[1], 457.0);
            assert_eq!(pos[2], 789.0);
        }
    }
}

impl BandwidthUsage {
    /// 新しい帯域幅監視オブジェクトを作成
    pub fn new() -> Self {
        Self {
            recent_bytes_sent: Vec::new(),
            recent_bytes_received: Vec::new(),
            peak_bandwidth: 5000.0, // 初期値: 5KB/秒
            estimated_available_bandwidth: 10000.0, // 初期値: 10KB/秒
            target_usage_ratio: 0.8, // 初期値: 帯域幅の80%まで使用
        }
    }

    /// 古いデータを削除
    pub fn cleanup_old_data(&mut self) {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(10);
        
        self.recent_bytes_sent.retain(|(time, _)| *time >= cutoff);
        self.recent_bytes_received.retain(|(time, _)| *time >= cutoff);
    }

    /// 現在の帯域幅使用量を計算 (bytes/秒)
    pub fn calculate_current_bandwidth(&self) -> f32 {
        let now = Instant::now();
        let window_start = now - Duration::from_secs(1);
        
        // 直近1秒間のデータ量を集計
        let bytes_in_last_second: usize = self.recent_bytes_sent
            .iter()
            .filter(|(time, _)| *time >= window_start)
            .map(|(_, size)| size)
            .sum();
            
        bytes_in_last_second as f32
    }

    /// 送信バイト数を記録
    pub fn record_sent(&mut self, bytes: usize) {
        let now = Instant::now();
        self.recent_bytes_sent.push((now, bytes));
        
        // 定期的にクリーンアップ
        if self.recent_bytes_sent.len() > 100 {
            self.cleanup_old_data();
        }
    }

    /// 受信バイト数を記録
    pub fn record_received(&mut self, bytes: usize) {
        let now = Instant::now();
        self.recent_bytes_received.push((now, bytes));
        
        // 定期的にクリーンアップ
        if self.recent_bytes_received.len() > 100 {
            self.cleanup_old_data();
        }
    }
} 