use serde::{Deserialize, Serialize};

/// サーバー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// HTTPサーバーのバインドアドレス
    pub http_bind_addr: String,
    
    /// WebSocketサーバーのバインドアドレス
    pub ws_bind_addr: String,
    
    /// 静的ファイルディレクトリ
    pub static_dir: String,
    
    /// 最大ルーム数
    pub max_rooms: usize,
    
    /// ルームタイムアウト（秒）
    pub room_timeout_secs: u64,
    
    /// プレイヤータイムアウト（秒）
    pub player_timeout_secs: u64,
    
    /// ルームあたりの最大プレイヤー数
    pub max_players_per_room: u8,
    
    /// Pingの間隔（秒）
    pub ping_interval_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_bind_addr: "127.0.0.1:8001".to_string(),
            ws_bind_addr: "127.0.0.1:8101".to_string(),
            static_dir: "./www".to_string(),
            max_rooms: 1000,
            room_timeout_secs: 3600, // 1時間
            player_timeout_secs: 300, // 5分
            max_players_per_room: 8,
            ping_interval_secs: 30,
        }
    }
}

/// ゲーム設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// ゲームタイプ
    pub game_type: GameType,
    
    /// 最大プレイヤー数
    pub max_players: u8,
    
    /// 追加設定（ゲームタイプ固有）
    pub options: serde_json::Value,
}

/// ゲームタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameType {
    /// 汎用ゲーム
    Generic,
    
    /// カスタムゲームタイプ
    Custom(String),
}

impl Default for GameType {
    fn default() -> Self {
        Self::Generic
    }
}

/// ゲームモード
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameMode {
    /// 協力モード
    Cooperative,
    
    /// 競争モード
    Competitive,
    
    /// チームモード
    Team,
}

impl Default for GameMode {
    fn default() -> Self {
        Self::Cooperative
    }
} 