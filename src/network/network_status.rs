//! ネットワーク状態監視モジュール
//!
//! このモジュールは、ネットワークの状態（RTT、パケットロス、帯域幅など）を
//! 監視し、適切な品質評価を行う機能を提供します。

use crate::ecs::{Resource, System, World, SystemPhase, SystemPriority, ResourceManager};
use crate::network::NetworkResource;
use std::collections::VecDeque;
use js_sys::Date;
use wasm_bindgen::JsValue;

/// 帯域の状態を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandwidthStatus {
    /// 良好 (1000 Kbps以上)
    Good,
    /// 十分 (500-1000 Kbps)
    Adequate,
    /// 制限あり (200-500 Kbps)
    Limited,
    /// 不足 (50-200 Kbps)
    Poor,
    /// 深刻な制限 (50 Kbps未満)
    Critical,
}

/// ネットワーク品質を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkQuality {
    /// 最高品質 (RTT < 50ms, パケットロス < 0.01)
    Excellent,
    /// 良好 (RTT < 100ms, パケットロス < 0.03)
    Good,
    /// 普通 (RTT < 150ms, パケットロス < 0.05)
    Fair,
    /// 不安定 (RTT < 250ms, パケットロス < 0.08)
    Poor,
    /// 非常に悪い (RTT >= 250ms, パケットロス >= 0.08)
    Bad,
}

/// ネットワーク状態を表すリソース
#[derive(Debug, Clone)]
pub struct NetworkStatus {
    /// 往復時間 (ミリ秒)
    pub rtt: f64,
    /// パケット損失率 (0.0 - 1.0)
    pub packet_loss: f32,
    /// 推定帯域幅 (Kbps)
    pub bandwidth_kbps: f32,
    /// 推定帯域の状態
    pub bandwidth_status: BandwidthStatus,
    /// RTTの変動 (ジッター, ミリ秒)
    pub latency_variation: f32,
    /// ネットワーク品質の総合評価
    pub quality: NetworkQuality,
    /// 最後の更新時刻 (ミリ秒)
    pub last_update: f64,
}

impl Default for NetworkStatus {
    fn default() -> Self {
        Self {
            rtt: 100.0,
            packet_loss: 0.0,
            bandwidth_kbps: 1000.0,
            bandwidth_status: BandwidthStatus::Good,
            latency_variation: 10.0,
            quality: NetworkQuality::Good,
            last_update: Date::now(),
        }
    }
}

/// ネットワーク状態監視システムの設定
#[derive(Debug, Clone)]
pub struct NetworkStatusMonitorConfig {
    /// RTT測定のサンプル数
    pub rtt_sample_size: usize,
    /// 帯域幅測定のサンプル数
    pub bandwidth_sample_size: usize,
    /// パケットロス測定のウィンドウサイズ
    pub packet_loss_window_size: usize,
    /// 品質評価の更新間隔 (ミリ秒)
    pub quality_update_interval: f64,
    /// パケット送信の追跡期間 (ミリ秒)
    pub packet_tracking_period: f64,
}

impl Default for NetworkStatusMonitorConfig {
    fn default() -> Self {
        Self {
            rtt_sample_size: 20,
            bandwidth_sample_size: 10,
            packet_loss_window_size: 100,
            quality_update_interval: 1000.0, // 1秒ごとに更新
            packet_tracking_period: 10000.0, // 10秒間のパケットを追跡
        }
    }
}

/// パケット情報を格納する構造体
#[derive(Debug, Clone)]
struct PacketInfo {
    /// パケットのシーケンス番号
    sequence: u32,
    /// 送信時刻 (ミリ秒)
    send_time: f64,
    /// 受信時刻 (ミリ秒, Noneは未受信)
    receive_time: Option<f64>,
    /// パケットサイズ (バイト)
    size: usize,
}

/// ネットワーク状態監視システム
pub struct NetworkStatusMonitor {
    /// 設定
    config: NetworkStatusMonitorConfig,
    /// 送信したパケットの履歴
    sent_packets: VecDeque<PacketInfo>,
    /// 受信したパケットの履歴 (シーケンス番号)
    received_sequences: VecDeque<u32>,
    /// RTTの履歴
    rtt_samples: VecDeque<f64>,
    /// 帯域幅の履歴 (Kbps)
    bandwidth_samples: VecDeque<f32>,
    /// 最後に品質を評価した時刻
    last_quality_update: f64,
    /// 最新のネットワーク状態
    status: NetworkStatus,
    /// 前回計測した合計送信バイト数
    last_total_bytes: usize,
    /// 前回の測定時刻
    last_measurement_time: f64,
}

impl Default for NetworkStatusMonitor {
    fn default() -> Self {
        let config = NetworkStatusMonitorConfig::default();
        let now = Date::now();
        
        // 必要な値を先に取得しておく
        let packet_loss_window_size = config.packet_loss_window_size;
        let rtt_sample_size = config.rtt_sample_size;
        let bandwidth_sample_size = config.bandwidth_sample_size;
        
        Self {
            config,
            sent_packets: VecDeque::with_capacity(packet_loss_window_size),
            received_sequences: VecDeque::with_capacity(packet_loss_window_size),
            rtt_samples: VecDeque::with_capacity(rtt_sample_size),
            bandwidth_samples: VecDeque::with_capacity(bandwidth_sample_size),
            last_quality_update: now,
            status: NetworkStatus::default(),
            last_total_bytes: 0,
            last_measurement_time: now,
        }
    }
}

impl NetworkStatusMonitor {
    /// 新しいネットワーク状態監視システムを作成
    pub fn new(config: NetworkStatusMonitorConfig) -> Self {
        let now = Date::now();
        
        Self {
            config,
            sent_packets: VecDeque::new(),
            received_sequences: VecDeque::new(),
            rtt_samples: VecDeque::new(),
            bandwidth_samples: VecDeque::new(),
            last_quality_update: now,
            status: NetworkStatus::default(),
            last_total_bytes: 0,
            last_measurement_time: now,
        }
    }
    
    /// パケット送信を記録
    pub fn record_packet_sent(&mut self, sequence: u32, size: usize) {
        let now = Date::now();
        
        // 古いパケット情報を削除
        self.clean_old_packets(now);
        
        // 新しいパケット情報を追加
        self.sent_packets.push_back(PacketInfo {
            sequence,
            send_time: now,
            receive_time: None,
            size,
        });
    }
    
    /// パケット受信を記録
    pub fn record_packet_received(&mut self, sequence: u32) {
        let now = Date::now();
        
        // 受信シーケンスを記録
        self.received_sequences.push_back(sequence);
        
        // 対応する送信パケットを探して受信時刻を記録
        for packet in &mut self.sent_packets {
            if packet.sequence == sequence && packet.receive_time.is_none() {
                packet.receive_time = Some(now);
                
                // RTTを計算
                let rtt = now - packet.send_time;
                self.rtt_samples.push_back(rtt);
                
                // RTTサンプル数を制限
                while self.rtt_samples.len() > self.config.rtt_sample_size {
                    self.rtt_samples.pop_front();
                }
                
                break;
            }
        }
        
        // 受信シーケンス数を制限
        while self.received_sequences.len() > self.config.packet_loss_window_size {
            self.received_sequences.pop_front();
        }
        
        // 状態を更新
        self.update_status(now);
    }
    
    /// 古いパケット情報を削除
    fn clean_old_packets(&mut self, now: f64) {
        let cutoff_time = now - self.config.packet_tracking_period;
        
        // 追跡期間外のパケットを削除
        while let Some(packet) = self.sent_packets.front() {
            if packet.send_time < cutoff_time {
                self.sent_packets.pop_front();
            } else {
                break;
            }
        }
    }
    
    /// パケットロスを計算
    fn calculate_packet_loss(&self) -> f32 {
        if self.sent_packets.is_empty() {
            return 0.0;
        }
        
        // 十分なサンプルがない場合は0を返す
        if self.sent_packets.len() < 10 {
            return 0.0;
        }
        
        // 受信確認されていないパケットをカウント
        let mut lost_packets = 0;
        let mut total_packets = 0;
        
        for packet in &self.sent_packets {
            // 送信から一定時間経過したパケットのみカウント
            let now = Date::now();
            if now - packet.send_time > 2000.0 { // 2秒以上経過
                total_packets += 1;
                if packet.receive_time.is_none() {
                    lost_packets += 1;
                }
            }
        }
        
        if total_packets == 0 {
            return 0.0;
        }
        
        (lost_packets as f32) / (total_packets as f32)
    }
    
    /// 平均RTTを計算
    fn calculate_average_rtt(&self) -> f64 {
        if self.rtt_samples.is_empty() {
            return 100.0; // デフォルト値
        }
        
        let sum: f64 = self.rtt_samples.iter().sum();
        sum / (self.rtt_samples.len() as f64)
    }
    
    /// RTTの変動（ジッター）を計算
    fn calculate_latency_variation(&self) -> f32 {
        if self.rtt_samples.len() < 2 {
            return 10.0; // デフォルト値
        }
        
        let mut variations = Vec::with_capacity(self.rtt_samples.len() - 1);
        let mut prev_rtt = self.rtt_samples[0];
        
        for &rtt in self.rtt_samples.iter().skip(1) {
            variations.push((rtt - prev_rtt).abs());
            prev_rtt = rtt;
        }
        
        if variations.is_empty() {
            return 10.0;
        }
        
        let sum: f64 = variations.iter().sum();
        (sum / (variations.len() as f64)) as f32
    }
    
    /// 帯域幅を計算
    fn calculate_bandwidth(&mut self, now: f64) {
        // 前回の計測から十分な時間が経過していない場合はスキップ
        if now - self.last_measurement_time < 1000.0 { // 1秒未満
            return;
        }
        
        // 送信したバイト数の合計を計算
        let total_bytes: usize = self.sent_packets.iter()
            .filter(|p| p.send_time > self.last_measurement_time)
            .map(|p| p.size)
            .sum();
        
        // 経過時間（秒）
        let elapsed_seconds = (now - self.last_measurement_time) / 1000.0;
        
        if elapsed_seconds > 0.0 {
            // Kbpsに変換
            let bandwidth = ((total_bytes * 8) as f32) / (elapsed_seconds as f32) / 1000.0;
            
            // サンプルを追加
            self.bandwidth_samples.push_back(bandwidth);
            
            // サンプル数を制限
            while self.bandwidth_samples.len() > self.config.bandwidth_sample_size {
                self.bandwidth_samples.pop_front();
            }
            
            // 状態を更新
            self.last_total_bytes = total_bytes;
            self.last_measurement_time = now;
        }
    }
    
    /// 平均帯域幅を計算
    fn calculate_average_bandwidth(&self) -> f32 {
        if self.bandwidth_samples.is_empty() {
            return 1000.0; // デフォルト値
        }
        
        let sum: f32 = self.bandwidth_samples.iter().sum();
        sum / (self.bandwidth_samples.len() as f32)
    }
    
    /// 帯域状態を評価
    fn evaluate_bandwidth_status(&self, bandwidth_kbps: f32) -> BandwidthStatus {
        match bandwidth_kbps {
            b if b >= 1000.0 => BandwidthStatus::Good,
            b if b >= 500.0 => BandwidthStatus::Adequate,
            b if b >= 200.0 => BandwidthStatus::Limited,
            b if b >= 50.0 => BandwidthStatus::Poor,
            _ => BandwidthStatus::Critical,
        }
    }
    
    /// ネットワーク品質を評価
    fn evaluate_network_quality(&self, rtt: f64, packet_loss: f32) -> NetworkQuality {
        // RTTとパケットロスに基づいて品質を判定
        match (rtt, packet_loss) {
            (r, p) if r < 50.0 && p < 0.01 => NetworkQuality::Excellent,
            (r, p) if r < 100.0 && p < 0.03 => NetworkQuality::Good,
            (r, p) if r < 150.0 && p < 0.05 => NetworkQuality::Fair,
            (r, p) if r < 250.0 && p < 0.08 => NetworkQuality::Poor,
            _ => NetworkQuality::Bad,
        }
    }
    
    /// ネットワーク状態を更新
    fn update_status(&mut self, now: f64) {
        // 品質更新間隔を確認
        if now - self.last_quality_update < self.config.quality_update_interval {
            return;
        }
        
        // 帯域幅を計算
        self.calculate_bandwidth(now);
        
        // 各指標を計算
        let rtt = self.calculate_average_rtt();
        let packet_loss = self.calculate_packet_loss();
        let bandwidth_kbps = self.calculate_average_bandwidth();
        let latency_variation = self.calculate_latency_variation();
        
        // 帯域状態を評価
        let bandwidth_status = self.evaluate_bandwidth_status(bandwidth_kbps);
        
        // ネットワーク品質を評価
        let quality = self.evaluate_network_quality(rtt, packet_loss);
        
        // 状態を更新
        self.status = NetworkStatus {
            rtt,
            packet_loss,
            bandwidth_kbps,
            bandwidth_status,
            latency_variation,
            quality,
            last_update: now,
        };
        
        // 最終更新時刻を記録
        self.last_quality_update = now;
    }
    
    /// 現在のネットワーク状態を取得
    pub fn get_status(&self) -> NetworkStatus {
        self.status.clone()
    }
}

impl System for NetworkStatusMonitor {
    fn name(&self) -> &'static str {
        "NetworkStatusMonitor"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(10) // ネットワーク状態は早めに更新
    }

    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        let now = Date::now();
        
        // ネットワークリソースを取得
        let _network_resource = match world.get_resource::<NetworkResource>() {
            Some(_) => (), // 存在確認のみ
            None => return Ok(()), // リソースがなければ何もしない
        };
        
        // 古いパケット情報を削除
        self.clean_old_packets(now);
        
        // 状態を更新
        self.update_status(now);
        
        // WorldにNetworkStatusリソースを更新
        world.insert_resource(self.status.clone());
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_packet_loss_calculation() {
        let mut monitor = NetworkStatusMonitor::default();
        
        // 10個のパケットを送信
        for i in 0..10 {
            monitor.record_packet_sent(i, 100);
        }
        
        // 8個のパケットを受信（20%ロス）
        for i in 0..8 {
            monitor.record_packet_received(i);
        }
        
        // 現在時刻を3秒後に設定して計算
        let now = Date::now() + 3000.0;
        
        // 手動で古いパケットをクリーンアップせずに計算
        for packet in &mut monitor.sent_packets {
            packet.send_time -= 3000.0; // 3秒前に送信したことにする
        }
        
        let packet_loss = monitor.calculate_packet_loss();
        assert!((packet_loss - 0.2).abs() < 0.01); // 20%のパケットロスを期待
    }
    
    #[test]
    fn test_rtt_calculation() {
        let mut monitor = NetworkStatusMonitor::default();
        
        // RTTが100msのパケットを5つ記録
        for i in 0..5 {
            monitor.rtt_samples.push_back(100.0);
        }
        
        let avg_rtt = monitor.calculate_average_rtt();
        assert!((avg_rtt - 100.0).abs() < 0.01);
    }
    
    #[test]
    fn test_quality_evaluation() {
        let monitor = NetworkStatusMonitor::default();
        
        // 良好な条件
        let quality1 = monitor.evaluate_network_quality(40.0, 0.005);
        assert_eq!(quality1, NetworkQuality::Excellent);
        
        // 普通の条件
        let quality2 = monitor.evaluate_network_quality(120.0, 0.04);
        assert_eq!(quality2, NetworkQuality::Fair);
        
        // 悪い条件
        let quality3 = monitor.evaluate_network_quality(300.0, 0.1);
        assert_eq!(quality3, NetworkQuality::Bad);
    }
} 