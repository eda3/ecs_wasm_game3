use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// プレイヤーID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub String);

impl PlayerId {
    /// 新しいプレイヤーIDを生成
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// 文字列からプレイヤーIDを生成
    pub fn from_string(s: &str) -> Self {
        Self(s.to_string())
    }
    
    /// 文字列表現を取得
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// プレイヤーの接続状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// 接続中
    Connected,
    
    /// 切断中（切断時刻を保持）
    Disconnected { since: u64 },
    
    /// 再接続中
    Reconnecting,
}

/// プレイヤー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// プレイヤーID
    pub id: PlayerId,
    
    /// プレイヤー名
    pub name: String,
    
    /// プレイヤーカラー（RGBA）
    pub color: [u8; 4],
    
    /// スコア
    pub score: i32,
    
    /// ホストかどうか
    pub is_host: bool,
    
    /// 準備完了状態
    pub is_ready: bool,
    
    /// 接続状態
    pub connection_state: ConnectionState,
    
    /// 最終アクティビティ時刻
    pub last_activity: u64,
    
    /// クライアントアドレス
    #[serde(skip)]
    pub client_addr: Option<String>,
}

impl Player {
    /// 新しいプレイヤーを作成
    pub fn new(name: String) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        Self {
            id: PlayerId::new(),
            name,
            // ランダムな色を生成（彩度と明度を高く）
            color: [
                rng.gen_range(20..220),
                rng.gen_range(20..220),
                rng.gen_range(20..220),
                255,
            ],
            score: 0,
            is_host: false,
            is_ready: false,
            connection_state: ConnectionState::Connected,
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            client_addr: None,
        }
    }
    
    /// プレイヤーの接続を更新
    pub fn update_connection(&mut self, connected: bool) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        self.last_activity = now;
        
        if connected {
            self.connection_state = ConnectionState::Connected;
        } else {
            self.connection_state = ConnectionState::Disconnected { since: now };
        }
    }
    
    /// プレイヤーが接続中かどうか
    pub fn is_connected(&self) -> bool {
        matches!(self.connection_state, ConnectionState::Connected)
    }
    
    /// プレイヤーをリセット（新しいゲームのため）
    pub fn reset_for_new_game(&mut self) {
        self.score = 0;
        self.is_ready = false;
    }
    
    /// プレイヤーの最終アクティビティからの経過時間
    pub fn inactivity_duration(&self) -> Duration {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Duration::from_secs(now.saturating_sub(self.last_activity))
    }
    
    /// プレイヤーを crate::message::Player に変換
    pub fn to_message_player(&self) -> crate::message::Player {
        crate::message::Player {
            id: self.id.to_string(),
            name: self.name.clone(),
            ready: self.is_ready,
            is_host: self.is_host,
            connected: self.is_connected(),
            color: self.color,
        }
    }
    
    /// プレイヤーの状態を crate::message::PlayerState に変換
    pub fn to_player_state(&self) -> crate::message::PlayerState {
        crate::message::PlayerState {
            id: self.id.to_string(),
            score: self.score,
            connected: self.is_connected(),
            data: serde_json::json!({}),
        }
    }
}

/// プレイヤーのセッション情報
#[derive(Debug)]
pub struct PlayerSession {
    /// プレイヤーID
    pub player_id: PlayerId,
    
    /// 現在参加中のルームID
    pub current_room: Option<String>,
    
    /// クライアントアドレス
    pub client_addr: String,
    
    /// セッション作成時刻
    pub created_at: Instant,
    
    /// 最終アクティビティ時刻
    pub last_activity: Instant,
}

impl PlayerSession {
    /// 新しいプレイヤーセッションを作成
    pub fn new(player_id: PlayerId, client_addr: String) -> Self {
        let now = Instant::now();
        Self {
            player_id,
            current_room: None,
            client_addr,
            created_at: now,
            last_activity: now,
        }
    }
    
    /// アクティビティを更新
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }
    
    /// 最終アクティビティからの経過時間
    pub fn inactivity_duration(&self) -> Duration {
        Instant::now().duration_since(self.last_activity)
    }
} 