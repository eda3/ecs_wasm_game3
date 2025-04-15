//! ネットワークモジュール
//! 
//! このモジュールはWebSocketを使用したクライアント・サーバー間の通信を実装します。
//! マルチプレイヤーゲームのための状態同期、予測と補正、ネットワーク最適化を提供します。

// サブモジュールをpubで公開
pub mod client;
pub mod server;
pub mod protocol;
pub mod sync;
pub mod prediction;
pub mod messages;
pub mod compression_system;
pub mod network_status;

// 必要なモジュールをリエクスポート
pub use client::NetworkClient;
pub use protocol::{NetworkMessage, MessageType};
pub use messages::{InputData, PlayerData, ComponentData};
pub use sync::SyncSystem;
pub use prediction::{PredictionSystem, ClientPrediction, ServerReconciliation};
pub use sync::MessageCompressor;
pub use messages::EntitySnapshot;
pub use compression_system::NetworkCompressionSystem;
pub use network_status::*;

// 外部クレートのインポート
use std::collections::HashMap;

// 内部モジュールのインポート
use crate::ecs::{Entity, Component, Resource};

// モジュール全体で共有する定数
/// ネットワーク更新の最大頻度（FPS）
pub const NETWORK_UPDATE_RATE: u32 = 20;

/// 接続状態を表す列挙型
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// 切断状態
    Disconnected,
    /// 接続試行中
    Connecting,
    /// 接続済み
    Connected,
    /// 切断処理中
    Disconnecting,
    /// エラー発生
    Error(String),
}

/// ネットワークエラーを表す列挙型
#[derive(Debug, Clone)]
pub enum NetworkError {
    /// 接続エラー
    ConnectionError(String),
    /// メッセージ処理エラー
    MessageProcessingError(String),
    /// タイムアウトエラー
    TimeoutError,
    /// 認証エラー
    AuthenticationError(String),
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::ConnectionError(msg) => write!(f, "接続エラー: {}", msg),
            NetworkError::MessageProcessingError(msg) => write!(f, "メッセージ処理エラー: {}", msg),
            NetworkError::TimeoutError => write!(f, "タイムアウトエラー"),
            NetworkError::AuthenticationError(msg) => write!(f, "認証エラー: {}", msg),
        }
    }
}

impl std::error::Error for NetworkError {}

/// ネットワーク設定リソース
#[derive(Debug, Resource, Clone)]
pub struct NetworkConfig {
    /// サーバーURL
    pub server_url: String,
    /// 同期頻度（更新/秒）
    pub sync_rate: u32,
    /// 接続タイムアウト（ミリ秒）
    pub connection_timeout_ms: u32,
    /// 再接続を試みる回数
    pub reconnect_attempts: u32,
    /// メッセージ圧縮を有効化するか
    pub enable_compression: bool,
    /// デバッグモードを有効化するか
    pub debug_mode: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:8080".to_string(),
            sync_rate: NETWORK_UPDATE_RATE,
            connection_timeout_ms: 5000,
            reconnect_attempts: 3,
            enable_compression: false,
            debug_mode: cfg!(debug_assertions),
        }
    }
}

/// 時間同期データ
#[derive(Debug, Clone, Default)]
pub struct TimeSyncData {
    /// サーバー時間とクライアント時間の差（ミリ秒）
    pub time_offset: f64,
    /// 往復遅延時間（ミリ秒）
    pub rtt: f64,
    /// 同期精度
    pub accuracy: f64,
    /// 最後の同期時刻
    pub last_sync: f64,
}

/// エンティティ所有権情報
#[derive(Debug, Clone)]
pub struct EntityOwnership {
    /// エンティティID
    pub entity_id: Entity,
    /// 所有者のプレイヤーID
    pub owner_id: Option<u32>,
    /// サーバーが権限を持つか
    pub server_authoritative: bool,
}

/// ネットワークリソース
#[derive(Debug, Resource)]
pub struct NetworkResource {
    /// サーバーURL
    pub server_url: String,
    /// プレイヤーID
    pub player_id: Option<u32>,
    /// 自分の制御するエンティティ
    pub controlled_entity: Option<Entity>,
    /// 最後に送信したシーケンス番号
    pub last_sequence: u32,
    /// RTT (Round Trip Time) 測定値 (ミリ秒)
    pub rtt: f64,
    /// サーバーとの時間オフセット
    pub time_offset: f64,
    /// 最後に受信したサーバー時間
    pub last_server_time: f64,
}

impl NetworkResource {
    /// 新しいネットワークリソースを作成
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            player_id: None,
            controlled_entity: None,
            last_sequence: 0,
            rtt: 0.0,
            time_offset: 0.0,
            last_server_time: 0.0,
        }
    }

    /// 次のシーケンス番号を取得
    pub fn next_sequence(&mut self) -> u32 {
        self.last_sequence += 1;
        self.last_sequence
    }

    /// サーバー時間を取得
    pub fn get_server_time(&self) -> f64 {
        js_sys::Date::now() + self.time_offset
    }

    /// 時間オフセットを更新
    pub fn update_time_offset(&mut self, client_time: f64, server_time: f64) {
        // RTTの半分をネットワーク遅延として扱う
        let now = js_sys::Date::now();
        let rtt = now - client_time;
        self.rtt = rtt;
        
        // サーバー時間とクライアント時間の差を計算
        self.time_offset = server_time - (now - rtt / 2.0);
        self.last_server_time = server_time;
    }
}

/// ネットワークマネージャー
#[derive(Debug)]
pub struct NetworkManager {
    pub connection_state: ConnectionState,
    pub player_id: Option<u32>,
    pub players: HashMap<u32, NetworkPlayer>,
    pub latency: f32,
    pub packet_loss: f32,
}

/// ネットワークプレイヤー
#[derive(Debug)]
pub struct NetworkPlayer {
    pub id: u32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub last_update: f64,
}

impl NetworkManager {
    /// 新しいネットワークマネージャーを作成
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            player_id: None,
            players: HashMap::new(),
            latency: 0.0,
            packet_loss: 0.0,
        }
    }

    /// サーバーに接続
    pub fn connect(&mut self, _server_url: &str) -> Result<(), String> {
        self.connection_state = ConnectionState::Connecting;
        // TODO: WebSocket接続の実装
        Ok(())
    }

    /// サーバーから切断
    pub fn disconnect(&mut self) {
        self.connection_state = ConnectionState::Disconnecting;
        // TODO: 切断処理の実装
    }

    /// プレイヤーの状態を更新
    pub fn update_player(&mut self, player_id: u32, position: [f32; 2], velocity: [f32; 2]) {
        if let Some(player) = self.players.get_mut(&player_id) {
            player.position = position;
            player.velocity = velocity;
            player.last_update = js_sys::Date::now();
        } else {
            self.players.insert(player_id, NetworkPlayer {
                id: player_id,
                position,
                velocity,
                last_update: js_sys::Date::now(),
            });
        }
    }
}

/// ネットワークコンポーネント
#[derive(Debug, Component)]
pub struct NetworkComponent {
    pub is_synced: bool,
    pub last_sync_time: f64,
    pub interpolation_factor: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_manager_creation() {
        let manager = NetworkManager::new();
        assert_eq!(manager.connection_state, ConnectionState::Disconnected);
        assert!(manager.player_id.is_none());
        assert!(manager.players.is_empty());
    }

    #[test]
    fn test_player_update() {
        let mut manager = NetworkManager::new();
        let position = [100.0, 200.0];
        let velocity = [10.0, 20.0];
        
        manager.update_player(1, position, velocity);
        let player = manager.players.get(&1).unwrap();
        
        assert_eq!(player.position, position);
        assert_eq!(player.velocity, velocity);
    }
} 