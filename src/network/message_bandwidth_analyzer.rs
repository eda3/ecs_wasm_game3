//! ネットワークメッセージの帯域幅分析ツール
//!
//! このモジュールはメッセージサイズ、頻度、重要度を分析し、
//! 最適な帯域幅使用のための情報を提供します。

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

use super::messages::EntitySnapshot;
use super::compression_system::{BandwidthStatus, EntityPriority};

/// メッセージ種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageCategory {
    /// 接続関連
    Connection,
    /// エンティティ同期
    EntitySync,
    /// 入力
    Input,
    /// チャット
    Chat,
    /// システム
    System,
    /// カスタム
    Custom,
}

/// メッセージサイズの統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    /// メッセージ種別
    pub category: String,
    /// 合計サイズ（バイト）
    pub total_bytes: usize,
    /// メッセージ数
    pub message_count: usize,
    /// 平均サイズ（バイト）
    pub average_size: f32,
    /// 最大サイズ（バイト）
    pub max_size: usize,
    /// 最小サイズ（バイト）
    pub min_size: usize,
    /// 単位時間あたりの帯域使用量（バイト/秒）
    pub bytes_per_second: f32,
}

impl MessageStats {
    /// 新しい統計情報を作成
    pub fn new(category: &str) -> Self {
        Self {
            category: category.to_string(),
            total_bytes: 0,
            message_count: 0,
            average_size: 0.0,
            max_size: 0,
            min_size: usize::MAX,
            bytes_per_second: 0.0,
        }
    }
    
    /// メッセージを追加して統計を更新
    pub fn add_message(&mut self, size: usize) {
        self.total_bytes += size;
        self.message_count += 1;
        self.max_size = self.max_size.max(size);
        self.min_size = self.min_size.min(size);
        self.average_size = self.total_bytes as f32 / self.message_count as f32;
    }
    
    /// 統計情報をリセット
    pub fn reset(&mut self) {
        self.total_bytes = 0;
        self.message_count = 0;
        self.average_size = 0.0;
        self.max_size = 0;
        self.min_size = usize::MAX;
        self.bytes_per_second = 0.0;
    }
    
    /// 報告期間に基づいて帯域使用量を計算
    pub fn calculate_bandwidth(&mut self, period_seconds: f32) {
        if period_seconds > 0.0 {
            self.bytes_per_second = self.total_bytes as f32 / period_seconds;
        }
    }
}

/// エンティティの帯域幅使用に関する情報
#[derive(Debug, Clone)]
struct EntityBandwidthInfo {
    /// エンティティID
    entity_id: u64,
    /// 優先度
    priority: EntityPriority,
    /// 最近の更新履歴
    recent_updates: VecDeque<(Instant, usize)>,
    /// 累積帯域使用量（バイト）
    total_bytes: usize,
    /// 更新回数
    update_count: usize,
    /// 最後の更新時間
    last_update: Instant,
}

/// 帯域幅分析レポート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthReport {
    /// 開始時刻からの経過時間（秒）
    pub elapsed_seconds: f32,
    /// 各カテゴリの統計
    pub categories: HashMap<String, MessageStats>,
    /// 合計送信バイト数
    pub total_sent_bytes: usize,
    /// 合計受信バイト数
    pub total_received_bytes: usize,
    /// 平均送信レート（バイト/秒）
    pub average_send_rate: f32,
    /// 平均受信レート（バイト/秒）
    pub average_receive_rate: f32,
    /// 帯域状態
    pub bandwidth_status: String,
    /// 最も帯域を使用しているエンティティTop5
    pub top_bandwidth_entities: Vec<(u64, usize)>,
    /// レポート生成時刻
    pub timestamp: String,
}

/// 帯域幅分析器
pub struct MessageBandwidthAnalyzer {
    /// 開始時刻
    start_time: Instant,
    /// カテゴリ別の統計
    category_stats: HashMap<MessageCategory, MessageStats>,
    /// 送信バイト数履歴
    sent_bytes_history: VecDeque<(Instant, usize)>,
    /// 受信バイト数履歴
    received_bytes_history: VecDeque<(Instant, usize)>,
    /// エンティティ別の帯域使用情報
    entity_bandwidth: HashMap<u64, EntityBandwidthInfo>,
    /// 履歴の保持期間（秒）
    history_duration: Duration,
    /// 最後のレポート生成時刻
    last_report_time: Instant,
    /// レポート生成間隔（秒）
    report_interval: Duration,
}

impl MessageBandwidthAnalyzer {
    /// 新しい帯域幅分析器を作成
    pub fn new() -> Self {
        let now = Instant::now();
        
        let mut category_stats = HashMap::new();
        category_stats.insert(MessageCategory::Connection, MessageStats::new("Connection"));
        category_stats.insert(MessageCategory::EntitySync, MessageStats::new("EntitySync"));
        category_stats.insert(MessageCategory::Input, MessageStats::new("Input"));
        category_stats.insert(MessageCategory::Chat, MessageStats::new("Chat"));
        category_stats.insert(MessageCategory::System, MessageStats::new("System"));
        category_stats.insert(MessageCategory::Custom, MessageStats::new("Custom"));
        
        Self {
            start_time: now,
            category_stats,
            sent_bytes_history: VecDeque::new(),
            received_bytes_history: VecDeque::new(),
            entity_bandwidth: HashMap::new(),
            history_duration: Duration::from_secs(60), // 1分間の履歴を保持
            last_report_time: now,
            report_interval: Duration::from_secs(5),   // 5秒ごとにレポート生成
        }
    }
    
    /// 送信メッセージを追跡
    pub fn track_sent_message(&mut self, category: MessageCategory, size: usize) {
        if let Some(stats) = self.category_stats.get_mut(&category) {
            stats.add_message(size);
        }
        
        let now = Instant::now();
        self.sent_bytes_history.push_back((now, size));
        self.cleanup_old_history();
    }
    
    /// 受信メッセージを追跡
    pub fn track_received_message(&mut self, category: MessageCategory, size: usize) {
        let now = Instant::now();
        self.received_bytes_history.push_back((now, size));
        self.cleanup_old_history();
    }
    
    /// エンティティスナップショットの送信を追跡
    pub fn track_entity_snapshot(&mut self, snapshot: &EntitySnapshot, size: usize, priority: EntityPriority) {
        let now = Instant::now();
        
        // エンティティの帯域情報を取得または作成
        let entity_info = self.entity_bandwidth
            .entry(snapshot.id)
            .or_insert_with(|| EntityBandwidthInfo {
                entity_id: snapshot.id,
                priority,
                recent_updates: VecDeque::new(),
                total_bytes: 0,
                update_count: 0,
                last_update: now,
            });
            
        // 更新情報を追加
        entity_info.recent_updates.push_back((now, size));
        entity_info.total_bytes += size;
        entity_info.update_count += 1;
        entity_info.last_update = now;
        entity_info.priority = priority; // 優先度を更新
        
        // 古い更新履歴を削除
        while let Some(front) = entity_info.recent_updates.front() {
            if now.duration_since(front.0) > self.history_duration {
                entity_info.recent_updates.pop_front();
            } else {
                break;
            }
        }
        
        // エンティティ同期カテゴリにも追加
        self.track_sent_message(MessageCategory::EntitySync, size);
    }
    
    /// 古い履歴データをクリーンアップ
    fn cleanup_old_history(&mut self) {
        let now = Instant::now();
        let cutoff = now - self.history_duration;
        
        // 古い送信履歴を削除
        while let Some(front) = self.sent_bytes_history.front() {
            if front.0 < cutoff {
                self.sent_bytes_history.pop_front();
            } else {
                break;
            }
        }
        
        // 古い受信履歴を削除
        while let Some(front) = self.received_bytes_history.front() {
            if front.0 < cutoff {
                self.received_bytes_history.pop_front();
            } else {
                break;
            }
        }
    }
    
    /// 最近の送信レート（バイト/秒）を計算
    pub fn calculate_recent_send_rate(&self, duration_secs: u64) -> f32 {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(duration_secs);
        
        let recent_bytes: usize = self.sent_bytes_history
            .iter()
            .filter(|(time, _)| *time >= cutoff)
            .map(|(_, size)| *size)
            .sum();
            
        recent_bytes as f32 / duration_secs as f32
    }
    
    /// 最近の受信レート（バイト/秒）を計算
    pub fn calculate_recent_receive_rate(&self, duration_secs: u64) -> f32 {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(duration_secs);
        
        let recent_bytes: usize = self.received_bytes_history
            .iter()
            .filter(|(time, _)| *time >= cutoff)
            .map(|(_, size)| *size)
            .sum();
            
        recent_bytes as f32 / duration_secs as f32
    }
    
    /// 帯域幅状態を評価
    pub fn evaluate_bandwidth_status(&self) -> BandwidthStatus {
        let recent_send_rate = self.calculate_recent_send_rate(5); // 直近5秒間
        
        if recent_send_rate > 50_000.0 {     // 50KB/s
            BandwidthStatus::Critical
        } else if recent_send_rate > 30_000.0 { // 30KB/s
            BandwidthStatus::High
        } else if recent_send_rate > 15_000.0 { // 15KB/s
            BandwidthStatus::Moderate
        } else {
            BandwidthStatus::Good
        }
    }
    
    /// 最も帯域を使用しているエンティティTOP Nを取得
    pub fn get_top_bandwidth_entities(&self, limit: usize) -> Vec<(u64, usize)> {
        let mut entities: Vec<(u64, usize)> = self.entity_bandwidth
            .values()
            .map(|info| (info.entity_id, info.total_bytes))
            .collect();
            
        // 帯域使用量でソート（降順）
        entities.sort_by(|a, b| b.1.cmp(&a.1));
        
        // 上位N件を返す
        entities.iter().take(limit).map(|&(id, bytes)| (id, bytes)).collect()
    }
    
    /// 帯域幅分析レポートを生成
    pub fn generate_report(&mut self) -> Option<BandwidthReport> {
        let now = Instant::now();
        
        // レポート間隔に達していない場合はNoneを返す
        if now.duration_since(self.last_report_time) < self.report_interval {
            return None;
        }
        
        // 経過時間を計算
        let elapsed = now.duration_since(self.start_time);
        let elapsed_seconds = elapsed.as_secs_f32();
        
        // カテゴリ別の統計情報を更新
        for stats in self.category_stats.values_mut() {
            stats.calculate_bandwidth(elapsed_seconds);
        }
        
        // 送受信の合計と平均レートを計算
        let total_sent_bytes: usize = self.sent_bytes_history.iter().map(|(_, size)| *size).sum();
        let total_received_bytes: usize = self.received_bytes_history.iter().map(|(_, size)| *size).sum();
        
        let average_send_rate = if elapsed_seconds > 0.0 {
            total_sent_bytes as f32 / elapsed_seconds
        } else {
            0.0
        };
        
        let average_receive_rate = if elapsed_seconds > 0.0 {
            total_received_bytes as f32 / elapsed_seconds
        } else {
            0.0
        };
        
        // 帯域状態を文字列に変換
        let bandwidth_status = match self.evaluate_bandwidth_status() {
            BandwidthStatus::Good => "Good",
            BandwidthStatus::Moderate => "Moderate",
            BandwidthStatus::High => "High",
            BandwidthStatus::Critical => "Critical",
        }.to_string();
        
        // カテゴリ別の統計をHashMapに変換
        let categories: HashMap<String, MessageStats> = self.category_stats
            .iter()
            .map(|(_, stats)| (stats.category.clone(), stats.clone()))
            .collect();
            
        // 最も帯域を使用しているエンティティTop5を取得
        let top_bandwidth_entities = self.get_top_bandwidth_entities(5);
        
        // タイムスタンプを生成
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // レポートを作成
        let report = BandwidthReport {
            elapsed_seconds,
            categories,
            total_sent_bytes,
            total_received_bytes,
            average_send_rate,
            average_receive_rate,
            bandwidth_status,
            top_bandwidth_entities,
            timestamp,
        };
        
        // 最後のレポート時間を更新
        self.last_report_time = now;
        
        Some(report)
    }
    
    /// レポート間隔を設定
    pub fn set_report_interval(&mut self, seconds: u64) {
        self.report_interval = Duration::from_secs(seconds);
    }
    
    /// 履歴の保持期間を設定
    pub fn set_history_duration(&mut self, seconds: u64) {
        self.history_duration = Duration::from_secs(seconds);
        self.cleanup_old_history(); // 新しい期間で古いデータをクリーンアップ
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_stats() {
        let mut stats = MessageStats::new("Test");
        
        // メッセージ追加のテスト
        stats.add_message(100);
        stats.add_message(200);
        stats.add_message(300);
        
        assert_eq!(stats.message_count, 3);
        assert_eq!(stats.total_bytes, 600);
        assert_eq!(stats.average_size, 200.0);
        assert_eq!(stats.max_size, 300);
        assert_eq!(stats.min_size, 100);
        
        // 帯域計算のテスト
        stats.calculate_bandwidth(2.0);
        assert_eq!(stats.bytes_per_second, 300.0);
        
        // リセットのテスト
        stats.reset();
        assert_eq!(stats.message_count, 0);
        assert_eq!(stats.total_bytes, 0);
    }
    
    #[test]
    fn test_bandwidth_analyzer() {
        let mut analyzer = MessageBandwidthAnalyzer::new();
        
        // 送信メッセージ追跡のテスト
        analyzer.track_sent_message(MessageCategory::EntitySync, 1000);
        analyzer.track_sent_message(MessageCategory::Input, 500);
        
        // 受信メッセージ追跡のテスト
        analyzer.track_received_message(MessageCategory::Connection, 200);
        
        // エンティティスナップショット追跡のテスト
        let snapshot = EntitySnapshot::new(1)
            .with_position([1.0, 2.0, 3.0])
            .with_rotation([0.0, 0.0, 0.0, 1.0]);
            
        analyzer.track_entity_snapshot(&snapshot, 2000, EntityPriority::High);
        
        // レポート生成を強制的にテスト（通常は時間経過でトリガー）
        analyzer.last_report_time = Instant::now() - Duration::from_secs(10);
        let report = analyzer.generate_report();
        
        assert!(report.is_some());
        let report = report.unwrap();
        
        // レポートの検証
        assert!(report.total_sent_bytes >= 3500); // 1000 + 500 + 2000
        assert!(report.total_received_bytes >= 200);
        
        // トップエンティティの検証
        assert!(!report.top_bandwidth_entities.is_empty());
        assert_eq!(report.top_bandwidth_entities[0].0, 1); // エンティティID
    }
} 