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

// 必要なモジュールをリエクスポート
pub use client::NetworkClient;
pub use protocol::{NetworkMessage, MessageType};
pub use messages::{InputData, PlayerData, ComponentData};
pub use sync::SyncSystem;
pub use prediction::{PredictionSystem, ClientPrediction, ServerReconciliation};

// 外部クレートのインポート
use wasm_bindgen::prelude::*;
use web_sys::{console, WebSocket, MessageEvent};
use js_sys::Date;
use std::collections::HashMap;
use std::collections::VecDeque;

// 内部モジュールのインポート
use crate::ecs::{World, Entity, Component, System, Resource};

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
#[derive(Debug, Resource)]
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