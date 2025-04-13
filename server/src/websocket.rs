use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use serde::{Deserialize, Serialize};

use crate::message::{ClientMessage, ServerMessage};
use crate::room::RoomManager;

/// 共有ルームマネージャー型
pub type SharedRoomManager = Arc<RwLock<RoomManager>>;

// ハートビートの間隔と期限切れ時間
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(120);
const ROOM_CLEANUP_INTERVAL: Duration = Duration::from_secs(3600); // 1時間
const ROOM_MAX_INACTIVE_TIME: Duration = Duration::from_secs(7200); // 2時間

/// WebSocket接続をハンドル
pub async fn handle_websocket(ws: WebSocket, room_manager: SharedRoomManager) {
    // プレイヤーIDを生成
    let player_id = Uuid::new_v4().to_string();
    
    println!("🔌 新しい接続: {}", player_id);
    
    // WebSocketを送信と受信に分割
    let (mut ws_tx, mut ws_rx) = ws.split();
    
    // メッセージ送信用チャンネルを作成
    let (tx, rx) = mpsc::unbounded_channel::<ServerMessage>();
    let mut rx = UnboundedReceiverStream::new(rx);
    
    // サーバーからのメッセージをWebSocketに送信するタスク
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            let json = serde_json::to_string(&message).unwrap_or_else(|e| {
                eprintln!("❌ JSONシリアル化エラー: {:?}", e);
                r#"{"type":"error","message":"内部エラー"}"#.to_string()
            });
            
            if let Err(e) = ws_tx.send(Message::text(json)).await {
                eprintln!("❌ WebSocket送信エラー: {:?}", e);
                break;
            }
        }
    });
    
    // ウェルカムメッセージを送信
    tx.send(ServerMessage::Welcome {
        player_id: player_id.clone(),
    }).unwrap_or_else(|e| {
        eprintln!("❌ ウェルカムメッセージ送信エラー: {:?}", e);
    });
    
    // 最後のアクティビティ時間を追跡
    let mut last_activity = Instant::now();
    
    // ハートビート送信タスク
    let heartbeat_tx = tx.clone();
    let heartbeat_player_id = player_id.clone();
    tokio::task::spawn(async move {
        let mut interval = time::interval(HEARTBEAT_INTERVAL);
        loop {
            interval.tick().await;
            
            // ハートビート送信
            if let Err(e) = heartbeat_tx.send(ServerMessage::Heartbeat) {
                eprintln!("❌ ハートビート送信エラー ({}): {:?}", heartbeat_player_id, e);
                break;
            }
        }
    });
    
    // クライアントからのメッセージを処理
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                // 最後のアクティビティ時間を更新
                last_activity = Instant::now();
                
                // テキストメッセージを処理
                if let Ok(text) = msg.to_str() {
                    process_message(&player_id, text, &tx, &room_manager).await;
                }
            }
            Err(e) => {
                eprintln!("❌ WebSocket受信エラー ({}): {:?}", player_id, e);
                break;
            }
        }
        
        // 長時間無応答のクライアントを切断
        if last_activity.elapsed() > CLIENT_TIMEOUT {
            println!("⏰ クライアントタイムアウト: {}", player_id);
            break;
        }
    }
    
    // 接続が閉じられた時にルームから退出
    player_disconnect(&player_id, &room_manager).await;
    println!("👋 接続終了: {}", player_id);
}

/// メッセージを処理
async fn process_message(
    player_id: &str,
    text: &str,
    tx: &mpsc::UnboundedSender<ServerMessage>,
    room_manager: &SharedRoomManager,
) {
    // JSONをパース
    let client_msg: ClientMessage = match serde_json::from_str(text) {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("❌ JSONパースエラー: {:?}", e);
            let _ = tx.send(ServerMessage::Error {
                message: "不正なメッセージ形式".to_string(),
            });
            return;
        }
    };
    
    // メッセージタイプに応じて処理
    match client_msg {
        ClientMessage::CreateRoom {
            player_name,
            game_type,
            settings,
        } => {
            let mut manager = room_manager.write().await;
            let (room_id, room_code) = manager.create_room(
                player_id.to_string(),
                player_name.clone(),
                game_type.clone(),
                settings.clone(),
                tx.clone(),
            );
            
            // ルーム作成成功メッセージを送信
            let _ = tx.send(ServerMessage::RoomCreated {
                room_id,
                room_code: room_code.clone(),
                game_type,
                settings,
                is_host: true,
            });
            
            println!("🏠 ルーム作成: {} (ホスト: {})", room_code, player_id);
        }
        
        ClientMessage::JoinRoom {
            player_name,
            room_code,
        } => {
            let mut manager = room_manager.write().await;
            match manager.join_room(
                player_id.to_string(),
                player_name.clone(),
                &room_code,
                tx.clone(),
            ) {
                Ok((room_id, room_code, game_type, settings, players, is_host)) => {
                    // ルーム参加成功メッセージを送信
                    let _ = tx.send(ServerMessage::RoomJoined {
                        room_id,
                        room_code,
                        game_type,
                        settings,
                        players,
                        is_host,
                    });
                    
                    println!("👋 ルーム参加: {} (プレイヤー: {})", room_code, player_id);
                }
                Err(e) => {
                    let _ = tx.send(ServerMessage::Error {
                        message: e,
                    });
                }
            }
        }
        
        ClientMessage::LeaveRoom => {
            let mut manager = room_manager.write().await;
            if manager.leave_room(player_id) {
                let _ = tx.send(ServerMessage::RoomLeft);
                println!("🚪 ルーム退出: {}", player_id);
            }
        }
        
        ClientMessage::StartGame { initial_state } => {
            let mut manager = room_manager.write().await;
            match manager.start_game(player_id, initial_state) {
                Ok(()) => {
                    println!("🎮 ゲーム開始: プレイヤー {}", player_id);
                }
                Err(e) => {
                    let _ = tx.send(ServerMessage::Error {
                        message: e,
                    });
                }
            }
        }
        
        ClientMessage::GameAction { action } => {
            // ゲームロジックはゲームタイプごとに実装
            handle_game_action(player_id, action, tx, room_manager).await;
        }
        
        ClientMessage::Chat { message } => {
            let mut manager = room_manager.write().await;
            if let Err(e) = manager.send_chat(player_id, message) {
                let _ = tx.send(ServerMessage::Error {
                    message: e,
                });
            }
        }
        
        ClientMessage::HeartbeatResponse => {
            // 何もしない（アクティビティ時間更新済み）
        }
    }
}

/// ゲームアクションを処理
async fn handle_game_action(
    player_id: &str,
    action: Value,
    tx: &mpsc::UnboundedSender<ServerMessage>,
    room_manager: &SharedRoomManager,
) {
    let room_opt = {
        let manager = room_manager.read().await;
        manager.get_player_room(player_id).map(|r| {
            (
                r.code.clone(),
                r.game_type.clone(),
                r.game_in_progress,
                r.game_state.clone(),
            )
        })
    };
    
    if let Some((room_code, game_type, in_progress, state)) = room_opt {
        // ゲームが進行中か確認
        if !in_progress {
            let _ = tx.send(ServerMessage::Error {
                message: "ゲームが開始されていません".to_string(),
            });
            return;
        }
        
        // ゲームタイプに応じたアクションハンドラーを呼び出す
        let result = match game_type.as_str() {
            "minesweeper" => {
                // マインスイーパーの場合はマインスイーパーロジックを実行
                #[cfg(feature = "minesweeper")]
                {
                    crate::games::minesweeper::handle_action(player_id, &action, state).await
                }
                #[cfg(not(feature = "minesweeper"))]
                {
                    Err("マインスイーパーゲームモジュールがロードされていません".to_string())
                }
            }
            // 他のゲームタイプを追加可能
            _ => Err(format!("未対応のゲームタイプ: {}", game_type)),
        };
        
        match result {
            Ok((action_result, new_state, game_ended, winner_ids)) => {
                let mut manager = room_manager.write().await;
                
                // アクション結果を送信
                if let Err(e) = manager.send_action_result(player_id, action_result) {
                    eprintln!("❌ アクション結果送信エラー: {}", e);
                }
                
                // ゲーム状態を更新
                if let Err(e) = manager.update_game_state(&room_code, new_state.clone()) {
                    eprintln!("❌ ゲーム状態更新エラー: {}", e);
                }
                
                // ゲームが終了した場合
                if game_ended {
                    if let Err(e) = manager.end_game(&room_code, winner_ids, new_state) {
                        eprintln!("❌ ゲーム終了処理エラー: {}", e);
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(ServerMessage::Error {
                    message: e,
                });
            }
        }
    } else {
        let _ = tx.send(ServerMessage::Error {
            message: "ルームに参加していません".to_string(),
        });
    }
}

/// プレイヤーの切断処理
async fn player_disconnect(player_id: &str, room_manager: &SharedRoomManager) {
    let mut manager = room_manager.write().await;
    if manager.leave_room(player_id) {
        println!("👋 ルーム退出（切断）: {}", player_id);
    }
}

/// 定期的なルームクリーンアップタスクを開始
pub async fn start_room_cleanup(room_manager: SharedRoomManager) {
    tokio::spawn(async move {
        let mut interval = time::interval(ROOM_CLEANUP_INTERVAL);
        loop {
            interval.tick().await;
            
            let mut manager = room_manager.write().await;
            manager.cleanup_inactive_rooms(ROOM_MAX_INACTIVE_TIME);
            println!("🧹 非アクティブルームのクリーンアップ完了");
        }
    });
}

/// WebSocketハンドラーを返す
pub fn create_websocket_handler(room_manager: SharedRoomManager) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::ws()
        .map(move |ws: warp::ws::Ws| {
            let room_manager = room_manager.clone();
            ws.on_upgrade(move |socket| handle_websocket(socket, room_manager))
        })
}

// WebSocketサーバーの開始
pub async fn start_websocket_server(room_manager: Arc<RwLock<RoomManager>>, port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => {
            log::info!("🔌 WebSocketサーバーを開始しました：{}", addr);
            listener
        },
        Err(e) => {
            log::error!("❌ WebSocketサーバーの起動に失敗しました：{}", e);
            return;
        }
    };
    
    // 定期的なクリーンアップタスクを開始
    let cleanup_manager = room_manager.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            
            // タイムアウトしたルームとプレイヤーを削除
            let mut manager = cleanup_manager.write().await;
            manager.cleanup(
                Duration::from_secs(3600), // ルームタイムアウト: 1時間
                Duration::from_secs(300)   // プレイヤータイムアウト: 5分
            );
        }
    });
    
    // 接続を受け付けるループ
    while let Ok((stream, addr)) = listener.accept().await {
        log::info!("🔗 新しい接続がありました：{}", addr);
        
        // 各クライアント接続を処理するタスクを生成
        let peer_manager = room_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer_manager).await {
                log::error!("❌ 接続処理中にエラーが発生しました：{}", e);
            }
        });
    }
}

// 接続の処理
async fn handle_connection(stream: TcpStream, room_manager: Arc<RwLock<RoomManager>>) -> Result<(), Box<dyn std::error::Error>> {
    // WebSocketに接続
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    log::debug!("🤝 WebSocket接続が確立されました");
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // プレイヤーID生成（認証前）
    let player_id = Uuid::new_v4().to_string();
    let mut player_name = String::new();
    let mut current_room_id: Option<String> = None;
    
    // メッセージ送信用チャネル
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    
    // メッセージ受信ループ
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = ws_sender.send(Message::Text(msg)).await {
                log::error!("❌ メッセージ送信エラー: {}", e);
                break;
            }
        }
    });
    
    // 定期的なpingを送信するタスク
    let ping_tx = tx.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let ping_msg = serde_json::to_string(&ServerMessage::Pong).unwrap();
            if ping_tx.send(ping_msg).is_err() {
                break;
            }
        }
    });
    
    // メッセージ処理ループ
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    // JSONメッセージをパース
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match process_message(
                            client_msg,
                            &player_id,
                            &mut player_name,
                            &mut current_room_id,
                            &tx,
                            room_manager.clone()
                        ).await {
                            Ok(_) => (),
                            Err(e) => {
                                // エラーメッセージを送信
                                let error_msg = serde_json::to_string(&ServerMessage::Error {
                                    code: 400,
                                    message: e.to_string(),
                                })?;
                                tx.send(error_msg)?;
                            }
                        }
                    } else {
                        // 不正なJSON形式
                        let error_msg = serde_json::to_string(&ServerMessage::Error {
                            code: 400,
                            message: "不正なメッセージ形式です".to_string(),
                        })?;
                        tx.send(error_msg)?;
                    }
                } else if let Message::Ping(_) = msg {
                    // Pingメッセージの応答
                    ws_sender.send(Message::Pong(vec![])).await?;
                }
            }
            Err(e) => {
                log::error!("❌ WebSocketエラー: {}", e);
                break;
            }
        }
    }
    
    // 切断処理
    log::info!("👋 クライアントが切断されました: {}", player_id);
    
    // プレイヤーがルームに参加していたら退出させる
    if let Some(room_id) = current_room_id {
        let mut manager = room_manager.write().await;
        if let Some(room) = manager.get_room_mut(&room_id) {
            room.remove_player(&player_id);
            
            // 他のプレイヤーに通知
            let leave_msg = serde_json::to_string(&ServerMessage::PlayerLeft {
                player_id: player_id.clone(),
            })?;
            room.broadcast_message_except(&player_id, &leave_msg);
        }
    }
    
    Ok(())
}

// メッセージの処理
async fn process_message(
    msg: ClientMessage,
    player_id: &str,
    player_name: &mut String,
    current_room_id: &mut Option<String>,
    tx: &mpsc::UnboundedSender<String>,
    room_manager: Arc<RwLock<RoomManager>>
) -> Result<(), String> {
    match msg {
        ClientMessage::Authenticate { name } => {
            // 名前のバリデーション
            if name.trim().is_empty() {
                return Err("名前は空にできません".to_string());
            }
            
            if name.len() > 20 {
                return Err("名前は20文字以内にしてください".to_string());
            }
            
            // 認証成功
            *player_name = name;
            
            let auth_msg = serde_json::to_string(&ServerMessage::Authenticated {
                player_id: player_id.to_string(),
            }).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            
            tx.send(auth_msg).map_err(|_| "メッセージ送信エラー".to_string())?;
        }
        
        ClientMessage::CreateRoom { name, max_players, is_public, game_type } => {
            // 認証済みかチェック
            if player_name.is_empty() {
                return Err("認証が必要です".to_string());
            }
            
            // ルーム名のバリデーション
            if name.trim().is_empty() {
                return Err("ルーム名は空にできません".to_string());
            }
            
            // すでにルームに参加していないか確認
            if current_room_id.is_some() {
                return Err("すでにルームに参加しています".to_string());
            }
            
            // ルームを作成
            let mut manager = room_manager.write().await;
            let room_info = manager.create_room(
                name,
                player_id.to_string(),
                player_name.clone(),
                tx.clone(),
                max_players,
                is_public,
                game_type
            );
            
            // ルームIDを保存
            *current_room_id = Some(room_info.id.clone());
            
            // 成功メッセージを送信
            let room_json = serde_json::to_value(&room_info).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            let created_msg = serde_json::to_string(&ServerMessage::RoomCreated {
                room_id: room_info.id,
            }).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            
            tx.send(created_msg).map_err(|_| "メッセージ送信エラー".to_string())?;
        }
        
        ClientMessage::JoinRoom { room_id } => {
            // 認証済みかチェック
            if player_name.is_empty() {
                return Err("認証が必要です".to_string());
            }
            
            // すでにルームに参加していないか確認
            if current_room_id.is_some() {
                return Err("すでにルームに参加しています".to_string());
            }
            
            // ルームが存在するか確認
            let mut manager = room_manager.write().await;
            let room = manager.get_room_mut(&room_id).ok_or("ルームが見つかりません")?;
            
            // ルームが満員でないか確認
            if room.players.len() >= room.info.max_players {
                return Err("ルームは満員です".to_string());
            }
            
            // ゲームがすでに開始されていないか確認
            if room.info.state != RoomState::Waiting {
                return Err("ゲームはすでに開始されています".to_string());
            }
            
            // プレイヤーをルームに追加
            if !room.add_player(player_id.to_string(), player_name.clone(), tx.clone()) {
                return Err("ルームに参加できませんでした".to_string());
            }
            
            // ルームIDを保存
            *current_room_id = Some(room_id.clone());
            
            // 他のプレイヤーに通知
            let join_msg = serde_json::to_string(&ServerMessage::PlayerJoined {
                player_id: player_id.to_string(),
                player_name: player_name.clone(),
            }).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            room.broadcast_message_except(player_id, &join_msg);
            
            // 成功メッセージを送信
            let room_json = serde_json::to_value(&room.info).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            let joined_msg = serde_json::to_string(&ServerMessage::RoomJoined {
                room_info: room_json,
            }).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            
            tx.send(joined_msg).map_err(|_| "メッセージ送信エラー".to_string())?;
        }
        
        ClientMessage::LeaveRoom => {
            // ルームに参加しているか確認
            if current_room_id.is_none() {
                return Err("ルームに参加していません".to_string());
            }
            
            let room_id = current_room_id.clone().unwrap();
            
            // ルームが存在するか確認
            let mut manager = room_manager.write().await;
            let room = manager.get_room_mut(&room_id).ok_or("ルームが見つかりません")?;
            
            // プレイヤーをルームから削除
            if !room.remove_player(player_id) {
                return Err("ルームから退出できませんでした".to_string());
            }
            
            // 他のプレイヤーに通知
            let leave_msg = serde_json::to_string(&ServerMessage::PlayerLeft {
                player_id: player_id.to_string(),
            }).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            room.broadcast_message(&leave_msg);
            
            // ルームが空になったら削除
            if room.players.is_empty() {
                manager.delete_room(&room_id);
            }
            
            // ルームIDをクリア
            *current_room_id = None;
            
            // 成功メッセージを送信
            let left_msg = serde_json::to_string(&ServerMessage::RoomLeft).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            tx.send(left_msg).map_err(|_| "メッセージ送信エラー".to_string())?;
        }
        
        ClientMessage::GameAction { action } => {
            // ルームに参加しているか確認
            if current_room_id.is_none() {
                return Err("ルームに参加していません".to_string());
            }
            
            let room_id = current_room_id.clone().unwrap();
            
            // ルームが存在するか確認
            let manager = room_manager.read().await;
            let room = manager.get_room(&room_id).ok_or("ルームが見つかりません")?;
            
            // ゲームアクションを処理（この実装ではダミー）
            // 実際のゲーム処理はgameモジュールに実装する
            
            // アクション結果を送信
            let action_msg = serde_json::to_string(&ServerMessage::GameAction {
                result: true,
                message: "アクションを受け付けました".to_string(),
                data: None,
            }).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            
            tx.send(action_msg).map_err(|_| "メッセージ送信エラー".to_string())?;
        }
        
        ClientMessage::StartGame => {
            // ルームに参加しているか確認
            if current_room_id.is_none() {
                return Err("ルームに参加していません".to_string());
            }
            
            let room_id = current_room_id.clone().unwrap();
            
            // ルームが存在するか確認
            let mut manager = room_manager.write().await;
            let room = manager.get_room_mut(&room_id).ok_or("ルームが見つかりません")?;
            
            // ルームのオーナーかどうか確認
            if room.info.owner_id != player_id {
                return Err("ゲームを開始する権限がありません".to_string());
            }
            
            // ゲームの状態を変更
            room.set_state(RoomState::Playing);
            
            // 全員にゲーム開始を通知
            let start_msg = serde_json::to_string(&ServerMessage::GameStarted).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            room.broadcast_message(&start_msg);
        }
        
        ClientMessage::ReadyState { ready } => {
            // 将来的にはここに準備状態の処理を実装
        }
        
        ClientMessage::Ping => {
            // Pongを返す
            let pong_msg = serde_json::to_string(&ServerMessage::Pong).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
            tx.send(pong_msg).map_err(|_| "メッセージ送信エラー".to_string())?;
        }
        
        ClientMessage::Chat { message } => {
            // 認証済みかチェック
            if player_name.is_empty() {
                return Err("認証が必要です".to_string());
            }
            
            // メッセージのバリデーション
            if message.trim().is_empty() {
                return Err("メッセージは空にできません".to_string());
            }
            
            if message.len() > 500 {
                return Err("メッセージは500文字以内にしてください".to_string());
            }
            
            // ルームに参加しているか確認
            if let Some(room_id) = current_room_id {
                // ルームが存在するか確認
                let manager = room_manager.read().await;
                if let Some(room) = manager.get_room(&room_id) {
                    // チャットメッセージを全員に送信
                    let chat_msg = serde_json::to_string(&ServerMessage::Chat {
                        player_id: player_id.to_string(),
                        player_name: player_name.clone(),
                        message,
                    }).map_err(|e| format!("JSONシリアライズエラー: {}", e))?;
                    
                    room.broadcast_message(&chat_msg);
                }
            } else {
                return Err("ルームに参加していません".to_string());
            }
        }
    }
    
    Ok(())
} 