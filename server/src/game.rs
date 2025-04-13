use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use std::time::{Duration, Instant};
use serde_json::Value;

use crate::config::{GameMode, GameSettings, GameType};
use crate::message::{GameAction, GameState, GamePhase, ActionResult};
use crate::player::PlayerId;

/// ゲームタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameType {
    /// マインスイーパー
    Minesweeper,
    /// テトリス
    Tetris,
    /// チェス
    Chess,
    /// オセロ
    Reversi,
    /// 汎用ゲームタイプ（カスタムゲーム用）
    Custom(String),
}

impl std::fmt::Display for GameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameType::Minesweeper => write!(f, "Minesweeper"),
            GameType::Tetris => write!(f, "Tetris"),
            GameType::Chess => write!(f, "Chess"),
            GameType::Reversi => write!(f, "Reversi"),
            GameType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// ゲーム設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// 最大プレイヤー数
    pub max_players: usize,
    /// ゲーム固有の設定
    pub game_specific: HashMap<String, serde_json::Value>,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            max_players: 4,
            game_specific: HashMap::new(),
        }
    }
}

/// プレイヤー情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// プレイヤーID
    pub id: String,
    /// プレイヤー名
    pub name: String,
    /// ホストかどうか
    pub is_host: bool,
    /// 準備完了状態
    pub ready: bool,
    /// 接続状態
    pub connected: bool,
    /// 最終アクティブ時間（Unix時間ミリ秒）
    pub last_active: u64,
}

/// ルーム概要（ルーム一覧用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSummary {
    /// ルームID
    pub id: String,
    /// ルームコード
    pub code: String,
    /// ゲームタイプ
    pub game_type: GameType,
    /// 現在のプレイヤー数
    pub player_count: usize,
    /// 最大プレイヤー数
    pub max_players: usize,
    /// ゲーム状態
    pub state: GameState,
    /// 作成時間（Unix時間ミリ秒）
    pub created_at: u64,
}

/// ゲーム状態
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameState {
    /// ロビー（ゲーム開始前）
    Lobby,
    /// ゲーム進行中
    InProgress,
    /// ゲーム終了
    Ended,
}

/// アクション結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// 成功したかどうか
    pub success: bool,
    /// アクション種別
    pub action_type: String,
    /// 結果メッセージ
    pub message: Option<String>,
    /// 追加データ
    pub data: Option<HashMap<String, serde_json::Value>>,
}

/// ゲームの状態を表すトレイト
pub trait GameState: Send + Sync {
    /// ゲームの状態をJSON形式でシリアライズ
    fn to_json(&self) -> Result<Value, String>;
    
    /// JSONからゲームの状態を復元
    fn from_json(json: &Value) -> Result<Self, String> where Self: Sized;
}

/// ゲームのイベントを表す列挙型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    // 基本イベント
    PlayerJoined { player_id: String, player_name: String },
    PlayerLeft { player_id: String },
    GameStarted,
    GameEnded { winner_id: Option<String> },
    
    // プレイヤー操作
    PlayerAction { player_id: String, action_type: String, data: Value },
    
    // タイマー関連
    TimerTick { remaining_seconds: u32 },
    TurnChanged { player_id: String, turn_number: u32 },
    
    // カスタムイベント
    Custom { event_type: String, data: Value },
}

/// ゲームの結果を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    pub winner_id: Option<String>,
    pub player_scores: std::collections::HashMap<String, i32>,
    pub game_duration: u64,
    pub stats: Value,
}

/// ゲームを表すトレイト
pub trait Game: Send + Sync {
    /// プレイヤーをゲームに追加
    fn add_player(&mut self, player_id: String, player_name: String) -> Result<(), String>;
    
    /// プレイヤーをゲームから削除
    fn remove_player(&mut self, player_id: &str) -> Result<(), String>;
    
    /// プレイヤーアクションを処理
    fn process_action(&mut self, player_id: &str, action_type: &str, data: Value) -> Result<Value, String>;
    
    /// ゲームを開始
    fn start_game(&mut self) -> Result<(), String>;
    
    /// ゲームを終了
    fn end_game(&mut self, winner_id: Option<String>) -> Result<GameResult, String>;
    
    /// ゲームの更新処理（毎フレーム呼び出される）
    fn update(&mut self, delta_time: f32) -> Vec<GameEvent>;
    
    /// ゲームの状態を取得
    fn get_state(&self) -> Result<Value, String>;
    
    /// ゲームの結果を取得
    fn get_result(&self) -> Option<GameResult>;
    
    /// ゲームが終了したかどうか
    fn is_game_ended(&self) -> bool;
}

/// 基本的なゲーム実装
pub struct BaseGame {
    // 基本情報
    pub name: String,
    pub max_players: usize,
    pub min_players: usize,
    
    // ゲーム状態
    pub players: std::collections::HashMap<String, String>, // player_id -> player_name
    pub started: bool,
    pub ended: bool,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub winner_id: Option<String>,
    pub turn_player_id: Option<String>,
    pub turn_number: u32,
    
    // ゲーム設定
    pub turn_timeout: Option<Duration>,
    pub game_timeout: Option<Duration>,
    pub custom_config: Value,
    
    // 結果
    pub result: Option<GameResult>,
}

impl BaseGame {
    /// 新しいベースゲームを作成
    pub fn new(name: String, min_players: usize, max_players: usize) -> Self {
        Self {
            name,
            max_players,
            min_players,
            players: std::collections::HashMap::new(),
            started: false,
            ended: false,
            start_time: None,
            end_time: None,
            winner_id: None,
            turn_player_id: None,
            turn_number: 0,
            turn_timeout: None,
            game_timeout: None,
            custom_config: serde_json::json!({}),
            result: None,
        }
    }
    
    /// ターンタイムアウトを設定
    pub fn set_turn_timeout(&mut self, seconds: u64) {
        self.turn_timeout = Some(Duration::from_secs(seconds));
    }
    
    /// ゲームタイムアウトを設定
    pub fn set_game_timeout(&mut self, seconds: u64) {
        self.game_timeout = Some(Duration::from_secs(seconds));
    }
    
    /// カスタム設定を設定
    pub fn set_custom_config(&mut self, config: Value) {
        self.custom_config = config;
    }
    
    /// 次のプレイヤーのターンに進める
    pub fn next_turn(&mut self) -> Option<GameEvent> {
        if !self.started || self.ended || self.players.is_empty() {
            return None;
        }
        
        // プレイヤーのリストを取得
        let player_ids: Vec<String> = self.players.keys().cloned().collect();
        
        // 現在のターンプレイヤーのインデックスを取得
        let current_index = if let Some(current_id) = &self.turn_player_id {
            player_ids.iter().position(|id| id == current_id).unwrap_or(0)
        } else {
            0
        };
        
        // 次のプレイヤーを選択
        let next_index = (current_index + 1) % player_ids.len();
        let next_player_id = player_ids[next_index].clone();
        
        // ターン情報を更新
        self.turn_player_id = Some(next_player_id.clone());
        self.turn_number += 1;
        
        // ターン変更イベントを返す
        Some(GameEvent::TurnChanged {
            player_id: next_player_id,
            turn_number: self.turn_number,
        })
    }
    
    /// ゲームの経過時間を取得（秒）
    pub fn elapsed_seconds(&self) -> u64 {
        if let Some(start) = self.start_time {
            let end = if self.ended {
                self.end_time.unwrap_or_else(Instant::now)
            } else {
                Instant::now()
            };
            
            end.duration_since(start).as_secs()
        } else {
            0
        }
    }
}

impl Game for BaseGame {
    fn add_player(&mut self, player_id: String, player_name: String) -> Result<(), String> {
        // すでにゲームが開始されている場合はエラー
        if self.started {
            return Err("ゲームはすでに開始されています".to_string());
        }
        
        // プレイヤー数が最大値に達している場合はエラー
        if self.players.len() >= self.max_players {
            return Err("プレイヤー数が最大値に達しています".to_string());
        }
        
        // すでに同じIDのプレイヤーが存在する場合はエラー
        if self.players.contains_key(&player_id) {
            return Err("同じIDのプレイヤーがすでに存在します".to_string());
        }
        
        // プレイヤーを追加
        self.players.insert(player_id.clone(), player_name.clone());
        
        Ok(())
    }
    
    fn remove_player(&mut self, player_id: &str) -> Result<(), String> {
        // すでにゲームが開始されている場合はエラー
        if self.started && !self.ended {
            return Err("ゲーム進行中にプレイヤーを削除できません".to_string());
        }
        
        // プレイヤーが存在しない場合はエラー
        if !self.players.contains_key(player_id) {
            return Err("プレイヤーが存在しません".to_string());
        }
        
        // プレイヤーを削除
        self.players.remove(player_id);
        
        // 現在のターンプレイヤーが削除されたプレイヤーの場合、次のプレイヤーのターンにする
        if let Some(turn_player) = &self.turn_player_id {
            if turn_player == player_id {
                self.next_turn();
            }
        }
        
        Ok(())
    }
    
    fn process_action(&mut self, player_id: &str, action_type: &str, data: Value) -> Result<Value, String> {
        // ゲームが開始されていない場合はエラー
        if !self.started {
            return Err("ゲームがまだ開始されていません".to_string());
        }
        
        // ゲームが終了している場合はエラー
        if self.ended {
            return Err("ゲームはすでに終了しています".to_string());
        }
        
        // プレイヤーが存在しない場合はエラー
        if !self.players.contains_key(player_id) {
            return Err("プレイヤーが存在しません".to_string());
        }
        
        // 現在のターンプレイヤーでない場合はエラー（ターン制ゲームの場合）
        if let Some(turn_player) = &self.turn_player_id {
            if turn_player != player_id && action_type != "chat" {
                return Err("あなたのターンではありません".to_string());
            }
        }
        
        // ベース実装では何もせずにデータをそのまま返す
        // 具体的なゲーム実装ではここをオーバーライドする
        Ok(data)
    }
    
    fn start_game(&mut self) -> Result<(), String> {
        // すでにゲームが開始されている場合はエラー
        if self.started {
            return Err("ゲームはすでに開始されています".to_string());
        }
        
        // プレイヤー数が最小値に達していない場合はエラー
        if self.players.len() < self.min_players {
            return Err(format!("ゲームを開始するには{}人以上のプレイヤーが必要です", self.min_players));
        }
        
        // ゲームを開始
        self.started = true;
        self.start_time = Some(Instant::now());
        
        // 最初のプレイヤーをターンプレイヤーに設定
        if !self.players.is_empty() {
            let first_player = self.players.keys().next().cloned();
            self.turn_player_id = first_player;
            self.turn_number = 1;
        }
        
        Ok(())
    }
    
    fn end_game(&mut self, winner_id: Option<String>) -> Result<GameResult, String> {
        // ゲームが開始されていない場合はエラー
        if !self.started {
            return Err("ゲームがまだ開始されていません".to_string());
        }
        
        // すでにゲームが終了している場合はエラー
        if self.ended {
            return Err("ゲームはすでに終了しています".to_string());
        }
        
        // 勝者が指定されている場合、プレイヤーが存在するか確認
        if let Some(id) = &winner_id {
            if !self.players.contains_key(id) {
                return Err("指定された勝者が存在しません".to_string());
            }
        }
        
        // ゲームを終了
        self.ended = true;
        self.end_time = Some(Instant::now());
        self.winner_id = winner_id.clone();
        
        // ゲーム結果を作成
        let mut player_scores = std::collections::HashMap::new();
        for (player_id, _) in &self.players {
            let score = if Some(player_id) == self.winner_id.as_ref() { 100 } else { 0 };
            player_scores.insert(player_id.clone(), score);
        }
        
        let game_duration = self.elapsed_seconds();
        
        let result = GameResult {
            winner_id,
            player_scores,
            game_duration,
            stats: serde_json::json!({
                "turns": self.turn_number,
            }),
        };
        
        // 結果を保存
        self.result = Some(result.clone());
        
        Ok(result)
    }
    
    fn update(&mut self, delta_time: f32) -> Vec<GameEvent> {
        let mut events = Vec::new();
        
        // ゲームが開始されていない場合や終了している場合は何もしない
        if !self.started || self.ended {
            return events;
        }
        
        // ゲームタイムアウトをチェック
        if let Some(timeout) = self.game_timeout {
            if let Some(start_time) = self.start_time {
                if start_time.elapsed() > timeout {
                    // ゲームを終了
                    if let Ok(result) = self.end_game(None) {
                        events.push(GameEvent::GameEnded { winner_id: None });
                    }
                    return events;
                }
            }
        }
        
        // ターンタイムアウトをチェック
        // ここでは実装しません（具体的なゲーム実装で必要に応じて実装）
        
        events
    }
    
    fn get_state(&self) -> Result<Value, String> {
        // ゲームの状態をJSON形式で返す
        let state = serde_json::json!({
            "name": self.name,
            "players": self.players,
            "started": self.started,
            "ended": self.ended,
            "turn_player_id": self.turn_player_id,
            "turn_number": self.turn_number,
            "elapsed_seconds": self.elapsed_seconds(),
            "winner_id": self.winner_id,
        });
        
        Ok(state)
    }
    
    fn get_result(&self) -> Option<GameResult> {
        self.result.clone()
    }
    
    fn is_game_ended(&self) -> bool {
        self.ended
    }
}

/// マインスイーパー用のゲーム構造体
pub struct MinesweeperGame {
    base: BaseGame,
    pub width: usize,
    pub height: usize,
    pub mines: usize,
    pub board: Vec<Vec<Cell>>,
    pub revealed_count: usize,
    pub flagged_count: usize,
    pub game_over: bool,
}

/// マインスイーパーのセル
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cell {
    pub has_mine: bool,
    pub is_revealed: bool,
    pub is_flagged: bool,
    pub adjacent_mines: u8,
}

impl MinesweeperGame {
    /// 新しいマインスイーパーゲームを作成
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        let mut base = BaseGame::new("Minesweeper".to_string(), 1, 4);
        base.set_custom_config(serde_json::json!({
            "width": width,
            "height": height,
            "mines": mines,
        }));
        
        // 空のボードを作成
        let board = vec![vec![Cell {
            has_mine: false,
            is_revealed: false,
            is_flagged: false,
            adjacent_mines: 0,
        }; width]; height];
        
        Self {
            base,
            width,
            height,
            mines,
            board,
            revealed_count: 0,
            flagged_count: 0,
            game_over: false,
        }
    }
    
    /// ボードを初期化（地雷の配置）
    fn initialize_board(&mut self, first_x: usize, first_y: usize) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // 地雷をランダムに配置
        let mut mines_placed = 0;
        while mines_placed < self.mines {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);
            
            // 最初にクリックしたセルとその周囲には地雷を配置しない
            let is_safe_zone = (x as i32 - first_x as i32).abs() <= 1 && (y as i32 - first_y as i32).abs() <= 1;
            
            if !self.board[y][x].has_mine && !is_safe_zone {
                self.board[y][x].has_mine = true;
                mines_placed += 1;
            }
        }
        
        // 隣接する地雷の数を計算
        for y in 0..self.height {
            for x in 0..self.width {
                if self.board[y][x].has_mine {
                    continue;
                }
                
                let mut count = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        
                        if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                            if self.board[ny as usize][nx as usize].has_mine {
                                count += 1;
                            }
                        }
                    }
                }
                
                self.board[y][x].adjacent_mines = count;
            }
        }
    }
    
    /// セルを開く
    fn reveal(&mut self, x: usize, y: usize) -> Result<Vec<(usize, usize)>, String> {
        // 範囲チェック
        if x >= self.width || y >= self.height {
            return Err("座標が範囲外です".to_string());
        }
        
        // すでに開かれているか、フラグが立っている場合は何もしない
        if self.board[y][x].is_revealed || self.board[y][x].is_flagged {
            return Ok(vec![]);
        }
        
        // 最初のクリックの場合はボードを初期化
        if self.revealed_count == 0 {
            self.initialize_board(x, y);
        }
        
        let mut revealed = vec![(x, y)];
        
        // セルを開く
        self.board[y][x].is_revealed = true;
        self.revealed_count += 1;
        
        // 地雷があればゲームオーバー
        if self.board[y][x].has_mine {
            self.game_over = true;
            return Ok(revealed);
        }
        
        // 隣接する地雷がない場合は周囲のセルも開く
        if self.board[y][x].adjacent_mines == 0 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        
                        if !self.board[ny][nx].is_revealed && !self.board[ny][nx].is_flagged {
                            if let Ok(mut sub_revealed) = self.reveal(nx, ny) {
                                revealed.append(&mut sub_revealed);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(revealed)
    }
    
    /// フラグを切り替える
    fn toggle_flag(&mut self, x: usize, y: usize) -> Result<bool, String> {
        // 範囲チェック
        if x >= self.width || y >= self.height {
            return Err("座標が範囲外です".to_string());
        }
        
        // すでに開かれている場合は何もしない
        if self.board[y][x].is_revealed {
            return Ok(false);
        }
        
        // フラグを切り替える
        self.board[y][x].is_flagged = !self.board[y][x].is_flagged;
        
        // フラグの数を更新
        if self.board[y][x].is_flagged {
            self.flagged_count += 1;
        } else {
            self.flagged_count -= 1;
        }
        
        Ok(self.board[y][x].is_flagged)
    }
    
    /// ゲームがクリアされたかどうか
    fn is_cleared(&self) -> bool {
        self.revealed_count + self.mines == self.width * self.height
    }
    
    /// プレイヤー視点のボードを取得
    fn get_player_board(&self) -> Vec<Vec<PlayerCell>> {
        let mut player_board = Vec::with_capacity(self.height);
        
        for y in 0..self.height {
            let mut row = Vec::with_capacity(self.width);
            
            for x in 0..self.width {
                let cell = &self.board[y][x];
                let player_cell = if cell.is_revealed {
                    if cell.has_mine {
                        PlayerCell::Mine
                    } else {
                        PlayerCell::Revealed(cell.adjacent_mines)
                    }
                } else if cell.is_flagged {
                    PlayerCell::Flagged
                } else {
                    PlayerCell::Hidden
                };
                
                row.push(player_cell);
            }
            
            player_board.push(row);
        }
        
        player_board
    }
}

/// プレイヤー視点のセル
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PlayerCell {
    Hidden,
    Revealed(u8),
    Flagged,
    Mine,
}

impl Game for MinesweeperGame {
    fn add_player(&mut self, player_id: String, player_name: String) -> Result<(), String> {
        self.base.add_player(player_id, player_name)
    }
    
    fn remove_player(&mut self, player_id: &str) -> Result<(), String> {
        self.base.remove_player(player_id)
    }
    
    fn process_action(&mut self, player_id: &str, action_type: &str, data: Value) -> Result<Value, String> {
        // ゲームが開始されていない場合はエラー
        if !self.base.started {
            return Err("ゲームがまだ開始されていません".to_string());
        }
        
        // ゲームが終了している場合はエラー
        if self.base.ended || self.game_over {
            return Err("ゲームはすでに終了しています".to_string());
        }
        
        // プレイヤーが存在しない場合はエラー
        if !self.base.players.contains_key(player_id) {
            return Err("プレイヤーが存在しません".to_string());
        }
        
        match action_type {
            "reveal" => {
                // 座標を取得
                let x = data["x"].as_u64().ok_or("x座標が不正です")? as usize;
                let y = data["y"].as_u64().ok_or("y座標が不正です")? as usize;
                
                // セルを開く
                let revealed = self.reveal(x, y)?;
                
                // ゲームオーバーかクリアかチェック
                if self.game_over {
                    self.base.end_game(None)?;
                } else if self.is_cleared() {
                    self.base.end_game(Some(player_id.to_string()))?;
                }
                
                // 結果を返す
                Ok(serde_json::json!({
                    "revealed": revealed,
                    "game_over": self.game_over,
                    "is_cleared": self.is_cleared(),
                }))
            },
            "flag" => {
                // 座標を取得
                let x = data["x"].as_u64().ok_or("x座標が不正です")? as usize;
                let y = data["y"].as_u64().ok_or("y座標が不正です")? as usize;
                
                // フラグを切り替える
                let is_flagged = self.toggle_flag(x, y)?;
                
                // 結果を返す
                Ok(serde_json::json!({
                    "x": x,
                    "y": y,
                    "is_flagged": is_flagged,
                    "flagged_count": self.flagged_count,
                }))
            },
            _ => Err(format!("不明なアクションタイプです: {}", action_type)),
        }
    }
    
    fn start_game(&mut self) -> Result<(), String> {
        self.base.start_game()
    }
    
    fn end_game(&mut self, winner_id: Option<String>) -> Result<GameResult, String> {
        self.base.end_game(winner_id)
    }
    
    fn update(&mut self, delta_time: f32) -> Vec<GameEvent> {
        self.base.update(delta_time)
    }
    
    fn get_state(&self) -> Result<Value, String> {
        // ベースのゲーム状態を取得
        let base_state = self.base.get_state()?;
        
        // マインスイーパー固有の状態を追加
        let mut state = serde_json::json!({
            "base": base_state,
            "width": self.width,
            "height": self.height,
            "mines": self.mines,
            "revealed_count": self.revealed_count,
            "flagged_count": self.flagged_count,
            "game_over": self.game_over,
            "board": self.get_player_board(),
        });
        
        // ゲームオーバーの場合は全ての地雷の位置を公開
        if self.game_over || self.base.ended {
            let mines = self.board.iter().enumerate().flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, cell)| {
                    if cell.has_mine {
                        Some(serde_json::json!({ "x": x, "y": y }))
                    } else {
                        None
                    }
                })
            }).collect::<Vec<_>>();
            
            if let Some(obj) = state.as_object_mut() {
                obj.insert("mines_positions".to_string(), serde_json::Value::Array(mines));
            }
        }
        
        Ok(state)
    }
    
    fn get_result(&self) -> Option<GameResult> {
        self.base.get_result()
    }
    
    fn is_game_ended(&self) -> bool {
        self.base.ended || self.game_over
    }
}

/// ゲームファクトリー - ゲーム設定に基づいて適切なゲーム実装を作成
pub struct GameFactory;

impl GameFactory {
    /// 新しいゲームインスタンスを作成
    pub fn create_game(settings: &GameSettings, players: &[PlayerId]) -> Box<dyn Game> {
        match settings.game_type {
            GameType::Generic => Box::new(GenericGame::new(settings, players)),
            GameType::Custom(ref name) => {
                // カスタムゲームの場合、名前に基づいて実装を選択
                match name.as_str() {
                    // 将来的に実装するカスタムゲーム
                    // "minesweeper" => Box::new(MinesweeperGame::new(settings, players)),
                    // "tictactoe" => Box::new(TicTacToeGame::new(settings, players)),
                    _ => Box::new(GenericGame::new(settings, players)),
                }
            }
        }
    }
} 