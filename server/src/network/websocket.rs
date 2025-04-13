use std::sync::{Arc, Mutex};
use std::time::Instant;

use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use dashmap::DashMap;
use log::{debug, error, info, warn};
use serde_json::json;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::config::ServerConfig;
use crate::message::{ClientMessage, ServerMessage};
use crate::player::{PlayerId, PlayerSession};
use crate::network::server::GameServer;

/// WebSocketコネクション
pub struct WebSocketConnection {
    /// サーバー設定
    config: ServerConfig,
    
    /// ゲームサーバー
    game_server: Arc<Mutex<GameServer>>,
    
    /// クライアントアドレス
    client_addr: String,
    
    /// プレイヤーID
    player_id: Option<PlayerId>,
    
    /// 現在のルームID
    current_room: Option<String>,
    
    /// 最後のアクティビティ時間
    last_activity: Instant,
    
    /// 接続確立時間
    connected_at: Instant,
    
    /// メッセージ送信チャネル
    tx: mpsc::Sender<String>,
    
    /// メッセージ受信チャネル
    rx: mpsc::Receiver<String>,
    
    /// 全クライアントのメッセージ送信チャネル
    message_channels: Arc<DashMap<String, mpsc::Sender<String>>>,
}

impl WebSocketConnection {
    /// 新しいWebSocketコネクションを作成
    pub fn new(
        config: ServerConfig,
        game_server: Arc<Mutex<GameServer>>,
        client_addr: String,
        message_channels: Arc<DashMap<String, mpsc::Sender<String>>>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        // チャネルを保存
        message_channels.insert(client_addr.clone(), tx.clone());
        
        Self {
            config,
            game_server,
            client_addr,
            player_id: None,
            current_room: None,
            last_activity: Instant::now(),
            connected_at: Instant::now(),
            tx,
            rx,
            message_channels,
        }
    }
    
    /// アクティビティ時間を更新
    fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }
    
    /// メッセージ送信処理
    fn send_message(&self, msg: ServerMessage) {
        if let Ok(json_str) = serde_json::to_string(&msg) {
            let _ = self.tx.try_send(json_str);
        } else {
            error!("Failed to serialize server message");
        }
    }
    
    /// エラーメッセージを送信
    fn send_error(&self, message: String) {
        self.send_message(ServerMessage::Error { message });
    }
    
    /// メッセージ処理ハンドラ
    fn handle_message(&mut self, msg: ClientMessage) {
        self.update_activity();
        
        match msg {
            ClientMessage::CreateRoom { player_name, game_type, settings } => {
                // ゲームサーバーでルームを作成
                let result = {
                    let mut server = self.game_server.lock().unwrap();
                    server.create_room(player_name.clone(), game_type, settings)
                };
                
                match result {
                    Ok(room) => {
                        // プレイヤーIDを保存
                        let player_id = room.players.first().unwrap().id.clone();
                        self.player_id = Some(player_id.clone());
                        self.current_room = Some(room.id.to_string());
                        
                        // 成功メッセージを送信
                        self.send_message(ServerMessage::RoomCreated {
                            room_id: room.id.to_string(),
                            room_code: room.code.clone(),
                            player_id: player_id.to_string(),
                        });
                    },
                    Err(e) => {
                        self.send_error(format!("Failed to create room: {}", e));
                    }
                }
            },
            ClientMessage::JoinRoom { room_code, player_name } => {
                // ゲームサーバーでルームに参加
                let result = {
                    let mut server = self.game_server.lock().unwrap();
                    server.join_room(&room_code, player_name)
                };
                
                match result {
                    Ok((room_id, player_id)) => {
                        // プレイヤーIDとルームIDを保存
                        self.player_id = Some(player_id.clone());
                        self.current_room = Some(room_id.clone());
                        
                        // 成功メッセージを送信
                        self.send_message(ServerMessage::RoomJoined {
                            room_id,
                            player_id: player_id.to_string(),
                        });
                    },
                    Err(e) => {
                        self.send_error(format!("Failed to join room: {}", e));
                    }
                }
            },
            ClientMessage::LeaveRoom => {
                // プレイヤーIDとルームIDをチェック
                if self.player_id.is_none() || self.current_room.is_none() {
                    self.send_error("Not in a room".to_string());
                    return;
                }
                
                let player_id = self.player_id.as_ref().unwrap().to_string();
                let room_id = self.current_room.as_ref().unwrap().clone();
                
                // ゲームサーバーでルームから退出
                let result = {
                    let mut server = self.game_server.lock().unwrap();
                    server.leave_room(&room_id, &player_id)
                };
                
                match result {
                    Ok(_) => {
                        // プレイヤーIDとルームIDをクリア
                        self.current_room = None;
                        
                        // 成功メッセージを送信
                        self.send_message(ServerMessage::RoomLeft {
                            room_id,
                        });
                    },
                    Err(e) => {
                        self.send_error(format!("Failed to leave room: {}", e));
                    }
                }
            },
            ClientMessage::GetRoomList => {
                // ゲームサーバーでルーム一覧を取得
                let room_list = {
                    let server = self.game_server.lock().unwrap();
                    server.get_room_list()
                };
                
                // ルーム一覧を送信
                self.send_message(ServerMessage::RoomList {
                    rooms: room_list,
                });
            },
            ClientMessage::GameAction { action } => {
                // プレイヤーIDとルームIDをチェック
                if self.player_id.is_none() || self.current_room.is_none() {
                    self.send_error("Not in a room".to_string());
                    return;
                }
                
                let player_id = self.player_id.as_ref().unwrap().to_string();
                let room_id = self.current_room.as_ref().unwrap().clone();
                
                // ゲームサーバーでアクションを処理
                let result = {
                    let mut server = self.game_server.lock().unwrap();
                    server.process_action(&room_id, &player_id, action)
                };
                
                match result {
                    Ok(action_result) => {
                        // 成功メッセージを送信
                        self.send_message(ServerMessage::ActionResult {
                            result: action_result,
                        });
                    },
                    Err(e) => {
                        self.send_error(format!("Failed to process action: {}", e));
                    }
                }
            },
            ClientMessage::Ping { timestamp } => {
                // Pingに応答
                self.send_message(ServerMessage::Pong {
                    client_timestamp: timestamp,
                    server_timestamp: crate::utils::current_timestamp_ms(),
                });
            },
            ClientMessage::Reconnect { player_id, room_id } => {
                // 将来的な実装のための準備
                // TODO: 再接続処理の実装
                self.send_error("Reconnect not implemented yet".to_string());
            },
        }
    }
}

impl Actor for WebSocketConnection {
    type Context = ws::WebsocketContext<Self>;
    
    /// アクター開始時の処理
    fn started(&mut self, ctx: &mut Self::Context) {
        // ハートビートを設定
        ctx.run_interval(self.config.heartbeat_interval(), |act, ctx| {
            // クライアントからの最後のアクティビティをチェック
            if Instant::now().duration_since(act.last_activity) > act.config.client_timeout() {
                info!("Client timeout, disconnecting: {}", act.client_addr);
                ctx.stop();
                return;
            }
            
            // クライアントに定期的なピングを送信
            act.send_message(ServerMessage::Ping {
                timestamp: crate::utils::current_timestamp_ms(),
            });
        });
        
        // 受信チャネルからのメッセージ処理
        ctx.run_interval(std::time::Duration::from_millis(100), |act, ctx| {
            if let Ok(msg) = act.rx.try_recv() {
                ctx.text(msg);
            }
        });
        
        info!("WebSocket connection established: {}", self.client_addr);
    }
    
    /// アクター終了時の処理
    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        // プレイヤーがルームにいる場合は、ルームから退出
        if let (Some(player_id), Some(room_id)) = (&self.player_id, &self.current_room) {
            let mut server = self.game_server.lock().unwrap();
            
            if let Err(e) = server.leave_room(room_id, &player_id.to_string()) {
                warn!("Failed to leave room on disconnect: {}", e);
            }
        }
        
        // メッセージチャネルを削除
        self.message_channels.remove(&self.client_addr);
        
        info!("WebSocket connection closed: {}", self.client_addr);
        actix::Running::Stop
    }
}

/// WebSocketテキストメッセージのハンドラ
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        self.update_activity();
        
        match msg {
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        debug!("Received client message: {:?}", client_msg);
                        self.handle_message(client_msg);
                    },
                    Err(e) => {
                        error!("Failed to parse client message: {}", e);
                        self.send_error(format!("Invalid message format: {}", e));
                    }
                }
            },
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            },
            Ok(ws::Message::Pong(_)) => {
                // pongを受信した場合は何もしない
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            },
            _ => {
                // その他のメッセージは無視
            },
        }
    }
}

/// WebSocketハンドラ
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<ServerConfig>,
    game_server: web::Data<Arc<Mutex<GameServer>>>,
    message_channels: web::Data<Arc<DashMap<String, mpsc::Sender<String>>>>,
) -> Result<HttpResponse, Error> {
    // クライアントアドレスを取得
    let client_addr = match req.peer_addr() {
        Some(addr) => addr.to_string(),
        None => Uuid::new_v4().to_string(),
    };
    
    // WebSocketコネクションを作成
    let websocket = WebSocketConnection::new(
        config.get_ref().clone(),
        game_server.get_ref().clone(),
        client_addr.clone(),
        message_channels.get_ref().clone(),
    );
    
    // WebSocketコネクションを開始
    let resp = ws::start(websocket, &req, stream)?;
    Ok(resp)
} 