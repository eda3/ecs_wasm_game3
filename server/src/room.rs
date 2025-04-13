use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use rand::distributions::{Alphanumeric, DistString};
use rand::Rng;
use serde_json::Value;

use crate::player::{Player, PlayerId};
use crate::config::{GameMode, GameSettings, GameType};
use crate::message::{GameState, GamePhase, ServerMessage, ClientMessage, ActionResult};
use crate::game::{Game, BaseGame};

/// ルームID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub String);

impl RoomId {
    /// 新しいルームIDを生成
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// 文字列からルームIDを生成
    pub fn from_string(s: &str) -> Self {
        Self(s.to_string())
    }
    
    /// 文字列表現を取得
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ルームの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomState {
    Waiting,   // プレイヤー待機中
    Playing,   // ゲームプレイ中
    Finished,  // ゲーム終了
}

/// ルーム情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub max_players: usize,
    pub player_count: usize,
    pub is_public: bool,
    pub state: RoomState,
    pub game_type: String,
    pub created_at: u64,
}

/// プレイヤー情報を表す構造体
#[derive(Debug, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub sender: mpsc::UnboundedSender<String>,
    pub last_ping: Instant,
}

/// ゲームルームを表す構造体
#[derive(Debug)]
pub struct GameRoom {
    pub info: RoomInfo,
    pub players: HashMap<String, Player>,
    pub spectators: HashSet<String>,
    pub last_activity: Instant,
}

impl GameRoom {
    pub fn new(name: String, owner_id: String, owner_name: String, sender: mpsc::UnboundedSender<String>, max_players: usize, is_public: bool, game_type: String) -> Self {
        let room_id = Uuid::new_v4().to_string();
        let now = Instant::now();
        
        let mut players = HashMap::new();
        players.insert(owner_id.clone(), Player {
            id: owner_id.clone(),
            name: owner_name,
            sender,
            last_ping: now,
        });
        
        Self {
            info: RoomInfo {
                id: room_id,
                name,
                owner_id,
                max_players,
                player_count: 1,
                is_public,
                state: RoomState::Waiting,
                game_type,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs(),
            },
            players,
            spectators: HashSet::new(),
            last_activity: now,
        }
    }
    
    // プレイヤーをルームに追加
    pub fn add_player(&mut self, player_id: String, player_name: String, sender: mpsc::UnboundedSender<String>) -> bool {
        if self.players.len() >= self.info.max_players {
            return false;
        }
        
        self.players.insert(player_id, Player {
            id: player_id,
            name: player_name,
            sender,
            last_ping: Instant::now(),
        });
        
        self.info.player_count = self.players.len();
        self.last_activity = Instant::now();
        true
    }
    
    // プレイヤーをルームから削除
    pub fn remove_player(&mut self, player_id: &str) -> bool {
        let was_removed = self.players.remove(player_id).is_some();
        
        if was_removed {
            self.info.player_count = self.players.len();
            
            // オーナーが退出した場合、新しいオーナーを設定
            if player_id == self.info.owner_id && !self.players.is_empty() {
                if let Some(new_owner_id) = self.players.keys().next().cloned() {
                    self.info.owner_id = new_owner_id;
                }
            }
        }
        
        self.last_activity = Instant::now();
        was_removed
    }
    
    // 観戦者を追加
    pub fn add_spectator(&mut self, spectator_id: String) -> bool {
        let result = self.spectators.insert(spectator_id);
        self.last_activity = Instant::now();
        result
    }
    
    // 観戦者を削除
    pub fn remove_spectator(&mut self, spectator_id: &str) -> bool {
        let result = self.spectators.remove(spectator_id);
        self.last_activity = Instant::now();
        result
    }
    
    // ルームの状態を更新
    pub fn set_state(&mut self, state: RoomState) {
        self.info.state = state;
        self.last_activity = Instant::now();
    }
    
    // 全プレイヤーにメッセージを送信
    pub fn broadcast_message(&self, message: &str) {
        for player in self.players.values() {
            let _ = player.sender.send(message.to_string());
        }
    }
    
    // 特定プレイヤー以外の全員にメッセージを送信
    pub fn broadcast_message_except(&self, excluded_player_id: &str, message: &str) {
        for (id, player) in self.players.iter() {
            if id != excluded_player_id {
                let _ = player.sender.send(message.to_string());
            }
        }
    }
}

/// ルームマネージャー
pub struct RoomManager {
    rooms: HashMap<String, GameRoom>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }
    
    // 新しいルームを作成
    pub fn create_room(&mut self, name: String, owner_id: String, owner_name: String, sender: mpsc::UnboundedSender<String>, max_players: usize, is_public: bool, game_type: String) -> RoomInfo {
        let room = GameRoom::new(name, owner_id, owner_name, sender, max_players, is_public, game_type);
        let room_info = room.info.clone();
        self.rooms.insert(room_info.id.clone(), room);
        room_info
    }
    
    // ルームを削除
    pub fn delete_room(&mut self, room_id: &str) -> bool {
        self.rooms.remove(room_id).is_some()
    }
    
    // ルームを取得
    pub fn get_room(&self, room_id: &str) -> Option<&GameRoom> {
        self.rooms.get(room_id)
    }
    
    // ルームを可変で取得
    pub fn get_room_mut(&mut self, room_id: &str) -> Option<&mut GameRoom> {
        self.rooms.get_mut(room_id)
    }
    
    // 公開ルーム一覧を取得
    pub fn get_public_rooms(&self) -> Vec<RoomInfo> {
        self.rooms.values()
            .filter(|room| room.info.is_public)
            .map(|room| room.info.clone())
            .collect()
    }
    
    // プレイヤーが参加しているルームを取得
    pub fn get_player_room(&self, player_id: &str) -> Option<&GameRoom> {
        self.rooms.values().find(|room| room.players.contains_key(player_id))
    }
    
    // プレイヤーが参加しているルームを可変で取得
    pub fn get_player_room_mut(&mut self, player_id: &str) -> Option<&mut GameRoom> {
        self.rooms.values_mut().find(|room| room.players.contains_key(player_id))
    }
    
    // タイムアウトしたルームとプレイヤーのクリーンアップ
    pub fn cleanup(&mut self, room_timeout: Duration, player_timeout: Duration) {
        let now = Instant::now();
        
        // タイムアウトしたプレイヤーを各ルームから削除
        for room in self.rooms.values_mut() {
            let timed_out_players: Vec<String> = room.players.iter()
                .filter(|(_, player)| now.duration_since(player.last_ping) > player_timeout)
                .map(|(id, _)| id.clone())
                .collect();
            
            for player_id in timed_out_players {
                log::info!("🕒 タイムアウトによりプレイヤーを削除: {}", player_id);
                room.remove_player(&player_id);
            }
        }
        
        // タイムアウトしたルームか空のルームを削除
        let to_remove: Vec<String> = self.rooms.iter()
            .filter(|(_, room)| {
                now.duration_since(room.last_activity) > room_timeout || room.players.is_empty()
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for room_id in to_remove {
            log::info!("🕒 タイムアウトまたは空のためルームを削除: {}", room_id);
            self.rooms.remove(&room_id);
        }
    }
}

/// ルームの要約情報（ルーム一覧表示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSummary {
    /// ルームID
    pub id: String,
    
    /// ルーム参加コード
    pub code: String,
    
    /// ルーム名
    pub name: String,
    
    /// 現在のプレイヤー数
    pub player_count: usize,
    
    /// 最大プレイヤー数
    pub max_players: usize,
    
    /// ゲームタイプ
    pub game_type: GameType,
    
    /// ルームの状態
    pub state: RoomState,
    
    /// 作成日時（Unix時間）
    pub created_at: u64,
}

/// スレッドセーフなルームマネージャー
pub type SharedRoomManager = Arc<RwLock<RoomManager>>;

/// 新しい共有ルームマネージャーを作成
pub fn create_room_manager() -> SharedRoomManager {
    Arc::new(RwLock::new(RoomManager::new()))
}

// 6文字の英数字ルームコード生成（わかりやすい文字のみ）
const ROOM_CODE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// ゲームルーム情報
#[derive(Debug, Clone)]
pub struct Room {
    /// ルームID (内部用UUID)
    pub id: String,
    /// 公開ルームコード (参加用)
    pub code: String,
    /// ゲームタイプ
    pub game_type: String,
    /// ゲーム設定
    pub settings: Value,
    /// ルームホストID
    pub host_id: String,
    /// 参加プレイヤー
    pub players: HashMap<String, Player>,
    /// プレイヤーごとのメッセージ送信チャンネル
    #[serde(skip)]
    pub player_channels: HashMap<String, mpsc::UnboundedSender<ServerMessage>>,
    /// ゲーム状態
    pub game_state: Option<Value>,
    /// ゲーム進行中かどうか
    pub game_in_progress: bool,
    /// 最終更新時刻
    pub last_updated: Instant,
}

impl Room {
    /// 新しいルームを作成
    pub fn new(host_id: String, host_name: String, game_type: String, settings: Value) -> Self {
        let room_id = Uuid::new_v4().to_string();
        let room_code = Self::generate_room_code();
        
        let mut players = HashMap::new();
        players.insert(
            host_id.clone(),
            Player {
                id: host_id.clone(),
                name: host_name,
                data: serde_json::json!({}),
            },
        );
        
        Room {
            id: room_id,
            code: room_code,
            game_type,
            settings,
            host_id,
            players,
            player_channels: HashMap::new(),
            game_state: None,
            game_in_progress: false,
            last_updated: Instant::now(),
        }
    }
    
    /// ランダムなルームコードを生成
    fn generate_room_code() -> String {
        let mut rng = rand::thread_rng();
        (0..6)
            .map(|_| {
                let idx = rng.gen_range(0..ROOM_CODE_CHARS.len());
                ROOM_CODE_CHARS[idx] as char
            })
            .collect()
    }
    
    /// プレイヤーをルームに追加
    pub fn add_player(&mut self, player_id: String, player_name: String, channel: mpsc::UnboundedSender<ServerMessage>) -> bool {
        // ゲーム進行中は参加不可
        if self.game_in_progress {
            return false;
        }
        
        // プレイヤー情報を追加
        self.players.insert(
            player_id.clone(),
            Player {
                id: player_id.clone(),
                name: player_name,
                data: serde_json::json!({}),
            },
        );
        
        // チャンネルを保存
        self.player_channels.insert(player_id, channel);
        self.last_updated = Instant::now();
        
        true
    }
    
    /// プレイヤーをルームから削除
    pub fn remove_player(&mut self, player_id: &str) -> bool {
        // プレイヤーとチャンネルを削除
        self.players.remove(player_id);
        self.player_channels.remove(player_id);
        
        // ルームが空になった場合
        if self.players.is_empty() {
            return false; // ルーム削除を意味する
        }
        
        // ホストが退出した場合は新しいホストを設定
        if player_id == self.host_id {
            if let Some(new_host_id) = self.players.keys().next().cloned() {
                self.host_id = new_host_id.clone();
                
                // 新しいホスト通知を全員に送信
                self.broadcast_message(ServerMessage::HostChanged {
                    host_id: new_host_id,
                });
            }
        }
        
        self.last_updated = Instant::now();
        true // ルームは存続
    }
    
    /// ゲームを開始
    pub fn start_game(&mut self, initial_state: Value) -> bool {
        if self.game_in_progress || self.players.len() < 1 {
            return false;
        }
        
        self.game_state = Some(initial_state.clone());
        self.game_in_progress = true;
        
        // ゲーム開始通知を全員に送信
        self.broadcast_message(ServerMessage::GameStarted {
            state: initial_state,
        });
        
        self.last_updated = Instant::now();
        true
    }
    
    /// ゲーム状態を更新
    pub fn update_game_state(&mut self, new_state: Value) {
        if !self.game_in_progress {
            return;
        }
        
        self.game_state = Some(new_state.clone());
        
        // ゲーム状態更新を全員に送信
        self.broadcast_message(ServerMessage::GameStateUpdate {
            state: new_state,
        });
        
        self.last_updated = Instant::now();
    }
    
    /// ゲームを終了
    pub fn end_game(&mut self, winner_ids: Option<Vec<String>>, final_state: Value) {
        if !self.game_in_progress {
            return;
        }
        
        self.game_state = Some(final_state.clone());
        self.game_in_progress = false;
        
        // ゲーム終了通知を全員に送信
        self.broadcast_message(ServerMessage::GameEnded {
            winner_ids,
            final_state,
        });
        
        self.last_updated = Instant::now();
    }
    
    /// ゲームアクション結果を送信
    pub fn send_action_result(&mut self, player_id: String, result: Value) {
        // アクション結果を全員に送信
        self.broadcast_message(ServerMessage::GameActionResult {
            player_id,
            result,
        });
        
        self.last_updated = Instant::now();
    }
    
    /// メッセージを全プレイヤーに送信
    pub fn broadcast_message(&self, message: ServerMessage) {
        for (player_id, channel) in &self.player_channels {
            if let Err(err) = channel.send(message.clone()) {
                // 送信エラーはログに記録するだけ（チャンネルクリーンアップは別処理で行う）
                eprintln!("メッセージ送信エラー（プレイヤー {}）: {:?}", player_id, err);
            }
        }
    }
    
    /// チャットメッセージを送信
    pub fn send_chat(&mut self, player_id: String, player_name: String, message: String) {
        // チャットメッセージを全員に送信
        self.broadcast_message(ServerMessage::Chat {
            player_id,
            player_name,
            message,
        });
        
        self.last_updated = Instant::now();
    }
    
    /// プレイヤーに個別メッセージを送信
    pub fn send_message_to_player(&self, player_id: &str, message: ServerMessage) -> bool {
        if let Some(channel) = self.player_channels.get(player_id) {
            channel.send(message).is_ok()
        } else {
            false
        }
    }
    
    /// 特定のプレイヤーがホストかどうか確認
    pub fn is_host(&self, player_id: &str) -> bool {
        self.host_id == player_id
    }
    
    /// ルームの公開情報を取得
    pub fn get_summary(&self) -> RoomSummary {
        RoomSummary {
            code: self.code.clone(),
            game_type: self.game_type.clone(),
            player_count: self.players.len(),
            in_progress: self.game_in_progress,
        }
    }
}

/// ルーム公開情報（一覧表示用）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomSummary {
    /// ルームコード
    pub code: String,
    /// ゲームタイプ
    pub game_type: String,
    /// プレイヤー数
    pub player_count: usize,
    /// ゲーム進行中かどうか
    pub in_progress: bool,
}

/// ルーム管理システム
pub struct RoomManager {
    /// ルーム一覧（コード→ルーム）
    rooms: HashMap<String, Room>,
    /// プレイヤーID→ルームコードのマッピング
    player_rooms: HashMap<String, String>,
}

impl RoomManager {
    /// 新しいルーム管理システムを作成
    pub fn new() -> Self {
        RoomManager {
            rooms: HashMap::new(),
            player_rooms: HashMap::new(),
        }
    }
    
    /// ルームを作成
    pub fn create_room(
        &mut self,
        host_id: String,
        host_name: String,
        game_type: String,
        settings: Value,
        channel: mpsc::UnboundedSender<ServerMessage>,
    ) -> (String, String) {
        // 既存のルームに参加していた場合は退出
        if let Some(room_code) = self.player_rooms.get(&host_id) {
            if let Some(room) = self.rooms.get_mut(room_code) {
                room.remove_player(&host_id);
                // ルームが空になった場合は削除
                if room.players.is_empty() {
                    self.rooms.remove(room_code);
                }
            }
            self.player_rooms.remove(&host_id);
        }
        
        // 新しいルームを作成
        let mut room = Room::new(host_id.clone(), host_name, game_type.clone(), settings.clone());
        
        // チャンネルを追加
        room.player_channels.insert(host_id.clone(), channel);
        
        let room_code = room.code.clone();
        let room_id = room.id.clone();
        
        // ルームを保存
        self.rooms.insert(room_code.clone(), room);
        self.player_rooms.insert(host_id, room_code.clone());
        
        (room_id, room_code)
    }
    
    /// ルームに参加
    pub fn join_room(
        &mut self,
        player_id: String,
        player_name: String,
        room_code: &str,
        channel: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<(String, String, String, Value, Vec<Player>, bool), String> {
        // ルームが存在するか確認
        let room = match self.rooms.get_mut(room_code) {
            Some(room) => room,
            None => return Err("ルームが見つかりません".to_string()),
        };
        
        // ゲームが進行中かチェック
        if room.game_in_progress {
            return Err("ゲームが既に進行中です".to_string());
        }
        
        // 既に他のルームに参加していた場合は退出
        if let Some(old_room_code) = self.player_rooms.get(&player_id) {
            if old_room_code != room_code {
                if let Some(old_room) = self.rooms.get_mut(old_room_code) {
                    old_room.remove_player(&player_id);
                    // ルームが空になった場合は削除
                    if old_room.players.is_empty() {
                        self.rooms.remove(old_room_code);
                    }
                }
            } else {
                // 同じルームに再参加の場合はチャンネルだけ更新
                if let Some(room) = self.rooms.get_mut(room_code) {
                    room.player_channels.insert(player_id.clone(), channel);
                    
                    // プレイヤー情報を取得
                    let players: Vec<Player> = room.players.values().cloned().collect();
                    let is_host = room.is_host(&player_id);
                    
                    return Ok((
                        room.id.clone(),
                        room.code.clone(),
                        room.game_type.clone(),
                        room.settings.clone(),
                        players,
                        is_host,
                    ));
                }
            }
        }
        
        // プレイヤーをルームに追加
        if !room.add_player(player_id.clone(), player_name.clone(), channel.clone()) {
            return Err("ルームに参加できません".to_string());
        }
        
        // プレイヤーIDとルームコードのマッピングを保存
        self.player_rooms.insert(player_id.clone(), room_code.to_string());
        
        // 他のプレイヤーに通知
        let player_info = room.players.get(&player_id).unwrap().clone();
        room.broadcast_message(ServerMessage::PlayerJoined {
            player: player_info.clone(),
        });
        
        // プレイヤー情報を取得
        let players: Vec<Player> = room.players.values().cloned().collect();
        let is_host = room.is_host(&player_id);
        
        Ok((
            room.id.clone(),
            room.code.clone(),
            room.game_type.clone(),
            room.settings.clone(),
            players,
            is_host,
        ))
    }
    
    /// ルームから退出
    pub fn leave_room(&mut self, player_id: &str) -> bool {
        // プレイヤーが参加しているルームを確認
        let room_code = match self.player_rooms.get(player_id) {
            Some(code) => code.clone(),
            None => return false,
        };
        
        // プレイヤー→ルームのマッピングを削除
        self.player_rooms.remove(player_id);
        
        // ルームからプレイヤーを削除
        if let Some(room) = self.rooms.get_mut(&room_code) {
            // 退出通知を送信
            room.broadcast_message(ServerMessage::PlayerLeft {
                player_id: player_id.to_string(),
            });
            
            // プレイヤーを削除
            let room_exists = room.remove_player(player_id);
            
            // ルームが空になった場合は削除
            if !room_exists {
                self.rooms.remove(&room_code);
            }
            
            return true;
        }
        
        false
    }
    
    /// ルームを取得
    pub fn get_room(&self, room_code: &str) -> Option<&Room> {
        self.rooms.get(room_code)
    }
    
    /// ルームを変更可能として取得
    pub fn get_room_mut(&mut self, room_code: &str) -> Option<&mut Room> {
        self.rooms.get_mut(room_code)
    }
    
    /// プレイヤーが参加しているルームを取得
    pub fn get_player_room(&self, player_id: &str) -> Option<&Room> {
        match self.player_rooms.get(player_id) {
            Some(room_code) => self.rooms.get(room_code),
            None => None,
        }
    }
    
    /// プレイヤーが参加しているルームを変更可能として取得
    pub fn get_player_room_mut(&mut self, player_id: &str) -> Option<&mut Room> {
        match self.player_rooms.get(player_id) {
            Some(room_code) => self.rooms.get_mut(room_code),
            None => None,
        }
    }
    
    /// プレイヤーが参加しているルームコードを取得
    pub fn get_player_room_code(&self, player_id: &str) -> Option<String> {
        self.player_rooms.get(player_id).cloned()
    }
    
    /// プレイヤーにメッセージを送信
    pub fn send_message_to_player(&self, player_id: &str, message: ServerMessage) -> bool {
        if let Some(room_code) = self.player_rooms.get(player_id) {
            if let Some(room) = self.rooms.get(room_code) {
                return room.send_message_to_player(player_id, message);
            }
        }
        false
    }
    
    /// ルームの一覧を取得
    pub fn list_rooms(&self) -> Vec<RoomSummary> {
        self.rooms.values().map(|room| room.get_summary()).collect()
    }
    
    /// ゲームを開始
    pub fn start_game(&mut self, player_id: &str, initial_state: Value) -> Result<(), String> {
        // プレイヤーが参加しているルームを確認
        let room = match self.get_player_room_mut(player_id) {
            Some(room) => room,
            None => return Err("ルームに参加していません".to_string()),
        };
        
        // ホストかどうか確認
        if !room.is_host(player_id) {
            return Err("ゲームを開始する権限がありません".to_string());
        }
        
        // ゲームを開始
        if !room.start_game(initial_state) {
            return Err("ゲームを開始できません".to_string());
        }
        
        Ok(())
    }
    
    /// ゲーム状態を更新
    pub fn update_game_state(&mut self, room_code: &str, new_state: Value) -> Result<(), String> {
        // ルームを確認
        let room = match self.rooms.get_mut(room_code) {
            Some(room) => room,
            None => return Err("ルームが見つかりません".to_string()),
        };
        
        // ゲーム状態を更新
        room.update_game_state(new_state);
        
        Ok(())
    }
    
    /// チャットメッセージを送信
    pub fn send_chat(&mut self, player_id: &str, message: String) -> Result<(), String> {
        // プレイヤーが参加しているルームを確認
        let room = match self.get_player_room_mut(player_id) {
            Some(room) => room,
            None => return Err("ルームに参加していません".to_string()),
        };
        
        // プレイヤー名を取得
        let player_name = match room.players.get(player_id) {
            Some(player) => player.name.clone(),
            None => return Err("プレイヤー情報が見つかりません".to_string()),
        };
        
        // チャットメッセージを送信
        room.send_chat(player_id.to_string(), player_name, message);
        
        Ok(())
    }
    
    /// ゲームアクション結果を送信
    pub fn send_action_result(&mut self, player_id: &str, result: Value) -> Result<(), String> {
        // プレイヤーが参加しているルームを確認
        let room = match self.get_player_room_mut(player_id) {
            Some(room) => room,
            None => return Err("ルームに参加していません".to_string()),
        };
        
        // アクション結果を送信
        room.send_action_result(player_id.to_string(), result);
        
        Ok(())
    }
    
    /// ゲームを終了
    pub fn end_game(&mut self, room_code: &str, winner_ids: Option<Vec<String>>, final_state: Value) -> Result<(), String> {
        // ルームを確認
        let room = match self.rooms.get_mut(room_code) {
            Some(room) => room,
            None => return Err("ルームが見つかりません".to_string()),
        };
        
        // ゲームを終了
        room.end_game(winner_ids, final_state);
        
        Ok(())
    }
    
    /// 古いルームをクリーンアップ
    pub fn cleanup_inactive_rooms(&mut self, max_inactive_time: Duration) {
        let now = Instant::now();
        let mut rooms_to_remove = Vec::new();
        
        // 非アクティブなルームを特定
        for (code, room) in &self.rooms {
            if now.duration_since(room.last_updated) > max_inactive_time {
                rooms_to_remove.push(code.clone());
            }
        }
        
        // 非アクティブなルームを削除
        for code in rooms_to_remove {
            if let Some(room) = self.rooms.remove(&code) {
                // プレイヤー→ルームのマッピングも削除
                for player_id in room.players.keys() {
                    self.player_rooms.remove(player_id);
                }
            }
        }
    }
} 