use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use tokio::time;
use uuid::Uuid;

use crate::config::{GameSettings, GameType, ServerConfig};
use crate::game::{Game, GameFactory};
use crate::message::{ActionResult, GameAction, GameState};
use crate::player::{ConnectionState, Player, PlayerId, PlayerSession};
use crate::room::{Room, RoomId, RoomState, RoomSummary};
use crate::utils;

/// ゲームサーバー
pub struct GameServer {
    /// サーバー設定
    config: ServerConfig,
    
    /// ルーム管理
    rooms: HashMap<String, Room>,
    
    /// ルームコードとIDのマッピング
    room_codes: HashMap<String, String>,
    
    /// プレイヤーセッション
    player_sessions: HashMap<String, PlayerSession>,
    
    /// メッセージ送信チャネル
    /// クライアントアドレスをキーとして、送信チャネルを管理
    #[allow(dead_code)]
    senders: Arc<DashMap<String, mpsc::Sender<String>>>,
    
    /// 起動時刻
    start_time: Instant,
}

impl GameServer {
    /// 新しいゲームサーバーを作成
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            rooms: HashMap::new(),
            room_codes: HashMap::new(),
            player_sessions: HashMap::new(),
            senders: Arc::new(DashMap::new()),
            start_time: Instant::now(),
        }
    }
    
    /// ルームを作成
    pub fn create_room(
        &mut self,
        player_name: String,
        game_type: GameType,
        settings: GameSettings,
    ) -> Result<Room, String> {
        // ルーム数の上限をチェック
        if self.rooms.len() >= self.config.max_rooms {
            return Err("Maximum number of rooms reached".to_string());
        }
        
        // プレイヤー作成
        let player = Player::new(player_name);
        
        // ルーム作成
        let room = Room::new(player, game_type, settings);
        
        // ルームコード重複チェック
        while self.room_codes.contains_key(&room.code) {
            // 重複する場合は新しいコードを生成
            let new_code = crate::room::generate_room_code();
            let mut new_room = room.clone();
            new_room.code = new_code;
            
            // 更新したルームを使用
            if !self.room_codes.contains_key(&new_room.code) {
                let room_id = new_room.id.to_string();
                let room_code = new_room.code.clone();
                
                // ルームを保存
                self.rooms.insert(room_id.clone(), new_room.clone());
                self.room_codes.insert(room_code, room_id);
                
                return Ok(new_room);
            }
        }
        
        // 通常パスでのルーム保存
        let room_id = room.id.to_string();
        let room_code = room.code.clone();
        
        self.rooms.insert(room_id.clone(), room.clone());
        self.room_codes.insert(room_code, room_id);
        
        Ok(room)
    }
    
    /// ルームに参加
    pub fn join_room(&mut self, room_code: &str, player_name: String) -> Result<(String, PlayerId), String> {
        // ルームコードからルームIDを取得
        let room_id = self.room_codes.get(room_code)
            .ok_or_else(|| "Room not found".to_string())?
            .clone();
            
        // ルームを取得
        let room = self.rooms.get_mut(&room_id)
            .ok_or_else(|| "Room not found".to_string())?;
            
        // プレイヤー作成
        let player = Player::new(player_name);
        let player_id = player.id.clone();
        
        // ルームに参加
        room.add_player(player)?;
        
        Ok((room_id, player_id))
    }
    
    /// ルームから退出
    pub fn leave_room(&mut self, room_id: &str, player_id: &str) -> Result<(), String> {
        // プレイヤーIDの変換
        let player_id = PlayerId::from_string(player_id);
        
        // ルームを取得
        let room = self.rooms.get_mut(room_id)
            .ok_or_else(|| "Room not found".to_string())?;
            
        // ルームから退出
        room.remove_player(&player_id)?;
        
        // ルームが空になった場合は削除
        if room.players.is_empty() {
            let room_code = room.code.clone();
            self.rooms.remove(room_id);
            self.room_codes.remove(&room_code);
        }
        
        Ok(())
    }
    
    /// ルーム一覧を取得
    pub fn get_room_list(&self) -> Vec<RoomSummary> {
        self.rooms.values()
            .map(|room| room.summary())
            .collect()
    }
    
    /// ルームを取得
    pub fn get_room(&self, room_id: &str) -> Option<Room> {
        self.rooms.get(room_id).cloned()
    }
    
    /// ルームをコードで取得
    pub fn get_room_by_code(&self, room_code: &str) -> Option<Room> {
        if let Some(room_id) = self.room_codes.get(room_code) {
            self.rooms.get(room_id).cloned()
        } else {
            None
        }
    }
    
    /// ゲームアクションを処理
    pub fn process_action(
        &mut self,
        room_id: &str,
        player_id: &str,
        action: GameAction,
    ) -> Result<ActionResult, String> {
        // プレイヤーIDの変換
        let player_id = PlayerId::from_string(player_id);
        
        // ルームを取得
        let room = self.rooms.get_mut(room_id)
            .ok_or_else(|| "Room not found".to_string())?;
            
        match action {
            GameAction::ToggleReady => {
                // 準備完了状態を切り替え
                let is_ready = room.toggle_player_ready(&player_id)?;
                
                Ok(ActionResult {
                    success: true,
                    action_type: "toggle_ready".to_string(),
                    message: Some(format!("Player ready status set to {}", is_ready)),
                    data: Some(serde_json::json!({ "is_ready": is_ready })),
                })
            },
            GameAction::StartGame => {
                // ゲーム開始
                let game_state = room.start_game(&player_id)?;
                
                Ok(ActionResult {
                    success: true,
                    action_type: "start_game".to_string(),
                    message: Some("Game started".to_string()),
                    data: Some(serde_json::json!({ "game_state": game_state })),
                })
            },
            GameAction::SendChat { message } => {
                // チャットメッセージ（この実装では単に成功を返すだけ）
                Ok(ActionResult {
                    success: true,
                    action_type: "send_chat".to_string(),
                    message: Some("Chat message sent".to_string()),
                    data: Some(serde_json::json!({ "message": message })),
                })
            },
            GameAction::GameSpecific(data) => {
                // ゲーム固有のアクション
                if room.state != RoomState::Playing {
                    return Err("Game is not in progress".to_string());
                }
                
                // 現在のゲーム状態を取得
                let game_state = room.game_state.as_ref()
                    .ok_or_else(|| "Game state not found".to_string())?;
                
                // ゲーム実装を生成
                let player_ids: Vec<PlayerId> = room.players.iter()
                    .map(|p| p.id.clone())
                    .collect();
                
                let mut game = GameFactory::create_game(&room.settings, &player_ids);
                
                // ゲームアクションを処理
                let result = game.process_action(&player_id, &GameAction::GameSpecific(data))?;
                
                // ゲーム状態を更新
                if room.state == RoomState::Playing {
                    let new_state = game.get_state(None);
                    room.game_state = Some(new_state);
                    
                    // ゲームが終了していれば、ルーム状態も更新
                    if game.is_game_over() {
                        let winners = game.get_winners();
                        room.end_game(winners)?;
                    }
                }
                
                Ok(result)
            },
        }
    }
    
    /// 非アクティブなルームとプレイヤーをクリーンアップ
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        
        // 非アクティブなルームをクリーンアップ
        let room_timeout = Duration::from_secs(self.config.room_timeout_secs);
        let mut rooms_to_remove = Vec::new();
        
        for (room_id, room) in &self.rooms {
            if now.duration_since(Instant::now() - room.inactivity_duration()) > room_timeout {
                rooms_to_remove.push((room_id.clone(), room.code.clone()));
            }
        }
        
        for (room_id, room_code) in rooms_to_remove {
            self.rooms.remove(&room_id);
            self.room_codes.remove(&room_code);
            info!("Removed inactive room: {}", room_id);
        }
        
        // 非アクティブなプレイヤーセッションをクリーンアップ
        let player_timeout = Duration::from_secs(self.config.player_timeout_secs);
        let mut players_to_remove = Vec::new();
        
        for (player_id, session) in &self.player_sessions {
            if session.inactivity_duration() > player_timeout {
                players_to_remove.push(player_id.clone());
            }
        }
        
        for player_id in players_to_remove {
            if let Some(session) = self.player_sessions.remove(&player_id) {
                // プレイヤーが参加中のルームがあれば、そこからも削除
                if let Some(room_id) = &session.current_room {
                    if let Some(room) = self.rooms.get_mut(room_id) {
                        if let Err(e) = room.remove_player(&session.player_id) {
                            warn!("Failed to remove player {} from room {}: {}", player_id, room_id, e);
                        }
                    }
                }
                info!("Removed inactive player session: {}", player_id);
            }
        }
    }
    
    /// サーバーの起動時間を取得
    pub fn uptime(&self) -> Duration {
        Instant::now().duration_since(self.start_time)
    }
    
    /// サーバー統計情報を取得
    pub fn get_stats(&self) -> ServerStats {
        ServerStats {
            uptime: self.uptime(),
            active_rooms: self.rooms.len(),
            active_players: self.player_sessions.len(),
            total_rooms_created: 0, // TODO: 実装
            total_players_connected: 0, // TODO: 実装
        }
    }
}

/// サーバー統計情報
#[derive(Debug, Clone)]
pub struct ServerStats {
    /// サーバー起動時間
    pub uptime: Duration,
    
    /// アクティブなルーム数
    pub active_rooms: usize,
    
    /// アクティブなプレイヤー数
    pub active_players: usize,
    
    /// 作成されたルームの総数
    pub total_rooms_created: usize,
    
    /// 接続したプレイヤーの総数
    pub total_players_connected: usize,
} 