use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::game::{ActionResult, GameType, GameSettings, RoomSummary};

/// クライアントからサーバーへのメッセージ
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ClientMessage {
    /// ルーム作成リクエスト
    CreateRoom {
        /// ゲームタイプ (例: "minesweeper")
        game_type: String,
        /// ゲーム設定 (JSONオブジェクト)
        settings: serde_json::Value,
        /// プレイヤー名
        player_name: String,
    },
    
    /// ルーム参加リクエスト
    JoinRoom {
        /// ルームコード
        room_code: String,
        /// プレイヤー名
        player_name: String,
    },
    
    /// ルーム退出リクエスト
    LeaveRoom,
    
    /// ゲーム開始リクエスト (ルームホストのみ)
    StartGame,
    
    /// ゲームアクション (ゲーム特有のアクション)
    GameAction {
        /// アクションデータ (JSONオブジェクト)
        action: serde_json::Value,
    },
    
    /// チャットメッセージ
    Chat {
        /// メッセージ内容
        message: String,
    },
    
    /// ハートビート応答
    Pong,
}

/// サーバーからクライアントへのメッセージ
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerMessage {
    /// 接続時のウェルカムメッセージ
    Welcome {
        /// プレイヤーID
        player_id: String,
    },
    
    /// ルーム作成成功
    RoomCreated {
        /// ルームコード
        room_code: String,
        /// ゲームタイプ
        game_type: String,
        /// ゲーム設定
        settings: serde_json::Value,
    },
    
    /// ルーム参加成功
    RoomJoined {
        /// ルームコード
        room_code: String,
        /// ゲームタイプ
        game_type: String,
        /// ゲーム設定
        settings: serde_json::Value,
        /// 既存プレイヤーリスト
        players: Vec<Player>,
        /// 自分がホストかどうか
        is_host: bool,
    },
    
    /// プレイヤーが入室
    PlayerJoined {
        /// プレイヤー情報
        player: Player,
    },
    
    /// プレイヤーが退室
    PlayerLeft {
        /// プレイヤーID
        player_id: String,
    },
    
    /// ホスト変更
    HostChanged {
        /// 新ホストID
        host_id: String,
    },
    
    /// ゲーム開始
    GameStarted {
        /// 初期ゲーム状態
        state: serde_json::Value,
    },
    
    /// ゲーム状態更新
    GameStateUpdate {
        /// 更新されたゲーム状態
        state: serde_json::Value,
    },
    
    /// ゲームアクション結果
    GameActionResult {
        /// アクション結果
        result: serde_json::Value,
        /// プレイヤーID (誰のアクションか)
        player_id: String,
    },
    
    /// ゲーム終了
    GameEnded {
        /// 勝者のプレイヤーID (協力ゲームの場合は全員かnull)
        winner_ids: Option<Vec<String>>,
        /// 最終ゲーム状態
        final_state: serde_json::Value,
    },
    
    /// チャットメッセージ
    Chat {
        /// 送信者ID
        player_id: String,
        /// 送信者名
        player_name: String,
        /// メッセージ内容
        message: String,
    },
    
    /// エラーメッセージ
    Error {
        /// エラーの詳細
        message: String,
    },
    
    /// ハートビート (接続維持用)
    Heartbeat,
}

/// プレイヤー情報
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    /// プレイヤーID
    pub id: String,
    /// プレイヤー名
    pub name: String,
    /// 追加のプレイヤーデータ
    #[serde(default)]
    pub data: serde_json::Value,
} 