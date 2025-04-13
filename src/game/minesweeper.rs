use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// プレイヤーID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub Uuid);

impl PlayerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// ルームID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub Uuid);

impl RoomId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// 5文字のルーム参加コードを生成（ABCDE形式）
    pub fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
        (0..5).map(|_| *chars.choose(&mut rng).unwrap()).collect()
    }
}

/// ゲームの難易度設定
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,     // 9x9, 10 mines
    Intermediate, // 16x16, 40 mines
    Advanced,     // 30x16, 99 mines
    Custom(u8, u8, u32), // width, height, mines
}

impl Difficulty {
    /// 難易度に基づいたゲームボードの設定を取得
    pub fn board_config(&self) -> (u8, u8, u32) {
        match self {
            Difficulty::Beginner => (9, 9, 10),
            Difficulty::Intermediate => (16, 16, 40),
            Difficulty::Advanced => (30, 16, 99),
            Difficulty::Custom(width, height, mines) => (*width, *height, *mines),
        }
    }
}

/// ゲームモード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    /// 協力モード：プレイヤーは協力してボードを解く
    Cooperative,
    /// 競争モード：プレイヤーは高得点を競う
    Competitive,
}

/// セルの座標
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
    
    /// 周囲8マスの座標を取得
    pub fn neighbors(&self, width: u8, height: u8) -> Vec<Position> {
        let mut neighbors = Vec::with_capacity(8);
        
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue; // 自分自身はスキップ
                }
                
                let nx = self.x as i16 + dx;
                let ny = self.y as i16 + dy;
                
                if nx >= 0 && nx < width as i16 && ny >= 0 && ny < height as i16 {
                    neighbors.push(Position::new(nx as u8, ny as u8));
                }
            }
        }
        
        neighbors
    }
}

/// セルの状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub position: Position,
    pub is_mine: bool,
    pub is_revealed: bool,
    pub is_flagged: bool,
    pub adjacent_mines: u8,
    pub revealed_by: Option<PlayerId>,
    pub flagged_by: Option<PlayerId>,
    pub reveal_time: Option<Instant>,
}

impl Cell {
    pub fn new(x: u8, y: u8) -> Self {
        Self {
            position: Position::new(x, y),
            is_mine: false,
            is_revealed: false,
            is_flagged: false,
            adjacent_mines: 0,
            revealed_by: None,
            flagged_by: None,
            reveal_time: None,
        }
    }
}

/// プレイヤーの接続状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Connected,
    Disconnected(Instant), // 切断時刻
    Reconnecting,
}

/// プレイヤー情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub username: String,
    pub color: [u8; 4], // RGBA
    pub score: u32,
    pub is_host: bool,
    pub is_ready: bool,
    pub connection_state: ConnectionState,
    pub last_activity: Instant,
}

impl Player {
    pub fn new(id: PlayerId, username: String) -> Self {
        let mut rng = rand::thread_rng();
        
        Self {
            id,
            username,
            // ランダムな色を生成（彩度と明度を高めに）
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
            last_activity: Instant::now(),
        }
    }
}

/// ゲーム結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    pub winner: Option<PlayerId>, // 競争モードの場合のみ
    pub scores: HashMap<PlayerId, u32>,
    pub duration: Duration,
    pub is_victory: bool,
}

/// ゲームの状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Lobby,
    InProgress(Instant), // 開始時刻
    Completed(GameResult),
    Abandoned,
}

/// ゲームボード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBoard {
    pub width: u8,
    pub height: u8,
    pub cells: Vec<Cell>,
    pub mine_count: u32,
    pub revealed_count: u32,
    pub flagged_count: u32,
    pub first_move_made: bool,
}

impl GameBoard {
    /// 新しいゲームボードを作成
    pub fn new(width: u8, height: u8, mine_count: u32) -> Self {
        let mut cells = Vec::with_capacity((width as usize) * (height as usize));
        
        for y in 0..height {
            for x in 0..width {
                cells.push(Cell::new(x, y));
            }
        }
        
        Self {
            width,
            height,
            cells,
            mine_count,
            revealed_count: 0,
            flagged_count: 0,
            first_move_made: false,
        }
    }
    
    /// 難易度からボードを生成
    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        let (width, height, mines) = difficulty.board_config();
        Self::new(width, height, mines)
    }
    
    /// インデックスから座標への変換
    pub fn index_to_position(&self, index: usize) -> Position {
        let x = (index % self.width as usize) as u8;
        let y = (index / self.width as usize) as u8;
        Position::new(x, y)
    }
    
    /// 座標からインデックスへの変換
    pub fn position_to_index(&self, pos: &Position) -> usize {
        (pos.y as usize) * (self.width as usize) + (pos.x as usize)
    }
    
    /// 座標からセルを取得
    pub fn get_cell(&self, pos: &Position) -> Option<&Cell> {
        if pos.x >= self.width || pos.y >= self.height {
            return None;
        }
        
        let index = self.position_to_index(pos);
        self.cells.get(index)
    }
    
    /// 座標からセルを可変参照で取得
    pub fn get_cell_mut(&mut self, pos: &Position) -> Option<&mut Cell> {
        if pos.x >= self.width || pos.y >= self.height {
            return None;
        }
        
        let index = self.position_to_index(pos);
        self.cells.get_mut(index)
    }
    
    /// 地雷を配置（初手で地雷を踏まないように）
    pub fn place_mines(&mut self, first_move: &Position) {
        let mut rng = rand::thread_rng();
        let total_cells = (self.width as u32) * (self.height as u32);
        
        // 地雷数が多すぎる場合は調整
        let mine_count = std::cmp::min(self.mine_count, total_cells - 9);
        self.mine_count = mine_count;
        
        // 初手とその周囲は地雷を置かない
        let safe_positions: HashSet<Position> = std::iter::once(*first_move)
            .chain(first_move.neighbors(self.width, self.height))
            .collect();
        
        // 地雷を配置できる位置のリスト
        let mut available_positions: Vec<Position> = self.cells.iter()
            .map(|cell| cell.position)
            .filter(|pos| !safe_positions.contains(pos))
            .collect();
        
        // ランダムに地雷を配置
        available_positions.shuffle(&mut rng);
        
        for pos in available_positions.iter().take(mine_count as usize) {
            if let Some(cell) = self.get_cell_mut(pos) {
                cell.is_mine = true;
            }
        }
        
        // 周囲の地雷数を計算
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Position::new(x, y);
                let adjacent_mines = self.count_adjacent_mines(&pos);
                
                if let Some(cell) = self.get_cell_mut(&pos) {
                    cell.adjacent_mines = adjacent_mines;
                }
            }
        }
        
        self.first_move_made = true;
    }
    
    /// 周囲の地雷数を数える
    fn count_adjacent_mines(&self, pos: &Position) -> u8 {
        let neighbors = pos.neighbors(self.width, self.height);
        
        neighbors.iter()
            .filter_map(|neighbor_pos| self.get_cell(neighbor_pos))
            .filter(|cell| cell.is_mine)
            .count() as u8
    }
    
    /// セルを開く
    pub fn reveal_cell(&mut self, pos: &Position, player_id: PlayerId) -> Vec<Position> {
        let mut revealed_positions = Vec::new();
        
        // 最初のセルが無効または既に開かれているか旗が立っている場合
        if let Some(cell) = self.get_cell(pos) {
            if cell.is_revealed || cell.is_flagged {
                return revealed_positions;
            }
        } else {
            return revealed_positions;
        }
        
        // 初手の場合、地雷を配置
        if !self.first_move_made {
            self.place_mines(pos);
        }
        
        // セルを再取得（place_mines後）
        let is_mine = if let Some(cell) = self.get_cell(pos) {
            cell.is_mine
        } else {
            return revealed_positions;
        };
        
        // 地雷を踏んだ場合
        if is_mine {
            if let Some(cell) = self.get_cell_mut(pos) {
                cell.is_revealed = true;
                cell.revealed_by = Some(player_id);
                cell.reveal_time = Some(Instant::now());
                self.revealed_count += 1;
            }
            revealed_positions.push(*pos);
            return revealed_positions;
        }
        
        // 再帰的にセルを開く処理
        let mut to_reveal = vec![*pos];
        while let Some(current_pos) = to_reveal.pop() {
            // 既に処理済みの場合はスキップ
            if revealed_positions.contains(&current_pos) {
                continue;
            }
            
            // セルを開く
            if let Some(cell) = self.get_cell_mut(&current_pos) {
                if cell.is_revealed || cell.is_flagged {
                    continue;
                }
                
                cell.is_revealed = true;
                cell.revealed_by = Some(player_id);
                cell.reveal_time = Some(Instant::now());
                self.revealed_count += 1;
                revealed_positions.push(current_pos);
                
                // 周囲に地雷がない場合、周囲のセルも開く
                if cell.adjacent_mines == 0 {
                    for neighbor_pos in current_pos.neighbors(self.width, self.height) {
                        if let Some(neighbor) = self.get_cell(&neighbor_pos) {
                            if !neighbor.is_revealed && !neighbor.is_flagged {
                                to_reveal.push(neighbor_pos);
                            }
                        }
                    }
                }
            }
        }
        
        revealed_positions
    }
    
    /// セルに旗を立てる/外す
    pub fn toggle_flag(&mut self, pos: &Position, player_id: PlayerId) -> bool {
        if let Some(cell) = self.get_cell_mut(pos) {
            if cell.is_revealed {
                return false;
            }
            
            if cell.is_flagged {
                cell.is_flagged = false;
                cell.flagged_by = None;
                self.flagged_count -= 1;
            } else {
                cell.is_flagged = true;
                cell.flagged_by = Some(player_id);
                self.flagged_count += 1;
            }
            
            return true;
        }
        
        false
    }
    
    /// ゲームクリア判定
    pub fn is_completed(&self) -> bool {
        // 全ての非地雷セルが開かれているか
        let non_mine_cells = (self.width as u32) * (self.height as u32) - self.mine_count;
        self.revealed_count >= non_mine_cells
    }
    
    /// 全ての地雷を開示
    pub fn reveal_all_mines(&mut self) {
        for cell in &mut self.cells {
            if cell.is_mine && !cell.is_revealed {
                cell.is_revealed = true;
                cell.reveal_time = Some(Instant::now());
            }
        }
    }
}

/// ゲームルーム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRoom {
    pub id: RoomId,
    pub code: String, // 参加コード（ABCDE形式）
    pub players: Vec<Player>,
    pub board: GameBoard,
    pub game_mode: GameMode,
    pub difficulty: Difficulty,
    pub state: GameState,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub max_players: u8,
    pub chat_history: Vec<ChatMessage>,
}

impl GameRoom {
    /// 新しいゲームルームを作成
    pub fn new(host: Player, game_mode: GameMode, difficulty: Difficulty) -> Self {
        let id = RoomId::new();
        let code = RoomId::generate_code();
        let board = GameBoard::from_difficulty(difficulty);
        let now = Instant::now();
        
        let mut host = host;
        host.is_host = true;
        
        Self {
            id,
            code,
            players: vec![host],
            board,
            game_mode,
            difficulty,
            state: GameState::Lobby,
            created_at: now,
            last_activity: now,
            max_players: 8,
            chat_history: Vec::new(),
        }
    }
    
    /// プレイヤーをルームに追加
    pub fn add_player(&mut self, player: Player) -> Result<(), &'static str> {
        // 定員チェック
        if self.players.len() >= self.max_players as usize {
            return Err("Room is full");
        }
        
        // 既に参加しているか確認
        if self.players.iter().any(|p| p.id == player.id) {
            return Err("Player already in room");
        }
        
        // ゲーム進行中は参加不可
        if matches!(self.state, GameState::InProgress(_)) {
            return Err("Game already in progress");
        }
        
        self.players.push(player);
        self.last_activity = Instant::now();
        Ok(())
    }
    
    /// プレイヤーをルームから削除
    pub fn remove_player(&mut self, player_id: PlayerId) -> Result<(), &'static str> {
        let player_index = self.players.iter().position(|p| p.id == player_id)
            .ok_or("Player not found")?;
        
        let removed_player = self.players.remove(player_index);
        
        // ホストが退出した場合、新しいホストを設定
        if removed_player.is_host && !self.players.is_empty() {
            self.players[0].is_host = true;
        }
        
        // プレイヤーがいなくなった場合、ゲームを終了
        if self.players.is_empty() {
            self.state = GameState::Abandoned;
        }
        
        self.last_activity = Instant::now();
        Ok(())
    }
    
    /// ゲームを開始
    pub fn start_game(&mut self, starter_id: PlayerId) -> Result<(), &'static str> {
        // 開始者がホストであることを確認
        if !self.players.iter().any(|p| p.id == starter_id && p.is_host) {
            return Err("Only the host can start the game");
        }
        
        // 全員が準備完了しているか確認
        if !self.players.iter().all(|p| p.is_ready) {
            return Err("Not all players are ready");
        }
        
        // プレイヤーが2人以上いるか確認
        if self.players.len() < 2 {
            return Err("At least 2 players are required");
        }
        
        // 新しいボードを生成
        self.board = GameBoard::from_difficulty(self.difficulty);
        
        // スコアをリセット
        for player in &mut self.players {
            player.score = 0;
        }
        
        // ゲーム状態を更新
        self.state = GameState::InProgress(Instant::now());
        self.last_activity = Instant::now();
        
        Ok(())
    }
    
    /// セルを開く
    pub fn reveal_cell(&mut self, player_id: PlayerId, pos: Position) -> Result<Vec<Position>, &'static str> {
        // ゲームが進行中か確認
        if !matches!(self.state, GameState::InProgress(_)) {
            return Err("Game is not in progress");
        }
        
        // プレイヤーがルームにいるか確認
        if !self.players.iter().any(|p| p.id == player_id) {
            return Err("Player not found");
        }
        
        // セルを開く
        let revealed = self.board.reveal_cell(&pos, player_id);
        
        // 得点計算
        if !revealed.is_empty() {
            let player_index = self.players.iter().position(|p| p.id == player_id).unwrap();
            
            // 通常のセル開封（1ポイント/セル）
            let score_increase = revealed.len() as u32;
            self.players[player_index].score += score_increase;
            
            // 地雷を踏んだ場合
            if let Some(cell) = self.board.get_cell(&pos) {
                if cell.is_mine && cell.is_revealed {
                    // 競争モードでは地雷を踏むとスコアが減少（最低0）
                    if matches!(self.game_mode, GameMode::Competitive) {
                        let penalty = std::cmp::min(self.players[player_index].score, 10);
                        self.players[player_index].score -= penalty;
                    }
                    
                    // 協力モードでは地雷を踏むとゲームオーバー
                    if matches!(self.game_mode, GameMode::Cooperative) {
                        self.end_game(false);
                    }
                }
            }
        }
        
        // クリア判定
        if self.board.is_completed() {
            self.end_game(true);
        }
        
        self.last_activity = Instant::now();
        Ok(revealed)
    }
    
    /// セルに旗を立てる/外す
    pub fn toggle_flag(&mut self, player_id: PlayerId, pos: Position) -> Result<bool, &'static str> {
        // ゲームが進行中か確認
        if !matches!(self.state, GameState::InProgress(_)) {
            return Err("Game is not in progress");
        }
        
        // プレイヤーがルームにいるか確認
        if !self.players.iter().any(|p| p.id == player_id) {
            return Err("Player not found");
        }
        
        let result = self.board.toggle_flag(&pos, player_id);
        self.last_activity = Instant::now();
        
        Ok(result)
    }
    
    /// ゲームを終了
    fn end_game(&mut self, is_victory: bool) {
        if let GameState::InProgress(start_time) = self.state {
            let duration = start_time.elapsed();
            
            let mut scores = HashMap::new();
            let mut winner = None;
            let mut max_score = 0;
            
            for player in &self.players {
                scores.insert(player.id, player.score);
                
                // 競争モードの場合、最高得点のプレイヤーを勝者とする
                if matches!(self.game_mode, GameMode::Competitive) {
                    if player.score > max_score {
                        max_score = player.score;
                        winner = Some(player.id);
                    }
                }
            }
            
            let result = GameResult {
                winner,
                scores,
                duration,
                is_victory,
            };
            
            // 全ての地雷を開示
            self.board.reveal_all_mines();
            
            // ゲーム状態を更新
            self.state = GameState::Completed(result);
            self.last_activity = Instant::now();
        }
    }
    
    /// プレイヤーの準備状態をトグル
    pub fn toggle_ready(&mut self, player_id: PlayerId) -> Result<bool, &'static str> {
        // プレイヤーがルームにいるか確認
        let player_index = self.players.iter().position(|p| p.id == player_id)
            .ok_or("Player not found")?;
        
        // 準備状態をトグル
        self.players[player_index].is_ready = !self.players[player_index].is_ready;
        self.last_activity = Instant::now();
        
        Ok(self.players[player_index].is_ready)
    }
}

/// チャットメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender_id: Option<PlayerId>,
    pub sender_name: String,
    pub content: String,
    pub timestamp: Instant,
    pub is_system: bool,
}

impl ChatMessage {
    pub fn new(sender_id: Option<PlayerId>, sender_name: String, content: String) -> Self {
        Self {
            sender_id,
            sender_name,
            content,
            timestamp: Instant::now(),
            is_system: false,
        }
    }
    
    pub fn system(content: String) -> Self {
        Self {
            sender_id: None,
            sender_name: "System".to_string(),
            content,
            timestamp: Instant::now(),
            is_system: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_board_creation() {
        let board = GameBoard::new(9, 9, 10);
        assert_eq!(board.width, 9);
        assert_eq!(board.height, 9);
        assert_eq!(board.mine_count, 10);
        assert_eq!(board.cells.len(), 81);
    }
    
    #[test]
    fn test_position_neighbors() {
        let pos = Position::new(1, 1);
        let neighbors = pos.neighbors(3, 3);
        assert_eq!(neighbors.len(), 8);
        
        let corner = Position::new(0, 0);
        let corner_neighbors = corner.neighbors(3, 3);
        assert_eq!(corner_neighbors.len(), 3);
    }
    
    #[test]
    fn test_room_creation() {
        let player_id = PlayerId::new();
        let player = Player::new(player_id, "TestPlayer".to_string());
        let room = GameRoom::new(player, GameMode::Cooperative, Difficulty::Beginner);
        
        assert_eq!(room.players.len(), 1);
        assert_eq!(room.players[0].is_host, true);
        assert_eq!(room.game_mode, GameMode::Cooperative);
        assert_eq!(room.board.width, 9);
        assert_eq!(room.board.height, 9);
        assert_eq!(room.board.mine_count, 10);
    }
    
    #[test]
    fn test_room_code_generation() {
        let code = RoomId::generate_code();
        assert_eq!(code.len(), 5);
        
        // 同じコードが生成されない確率が高い
        let code2 = RoomId::generate_code();
        assert_ne!(code, code2);
    }
} 