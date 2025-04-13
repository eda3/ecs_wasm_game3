use std::env;
use std::sync::Arc;
use std::time::Duration;
use std::path::PathBuf;

use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, get};
use futures_util::{SinkExt, StreamExt};
use log::{info, error, warn, debug};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use clap::Parser;
use tokio::sync::RwLock;

mod game;
mod room;
mod message;
mod player;
mod network;
mod config;
mod utils;
mod http;
mod websocket;

use crate::network::server::GameServer;
use crate::room::RoomManager;

/// WebSocketゲームサーバー
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    /// 静的ファイルディレクトリ
    #[clap(short, long, default_value = "client/dist")]
    static_dir: PathBuf,
    
    /// HTTPサーバーポート
    #[clap(short = 'p', long, default_value = "8001")]
    http_port: u16,
    
    /// WebSocketサーバーポート
    #[clap(short = 'w', long, default_value = "8101")]
    ws_port: u16,
}

// HTTPサーバーのルートエンドポイント
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("ECS WebAssembly Game Server")
}

// ヘルスチェックエンドポイント
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// アクティブなルーム一覧を取得するAPI
#[get("/api/rooms")]
async fn get_rooms(server: web::Data<Arc<Mutex<GameServer>>>) -> impl Responder {
    let game_server = server.lock().await;
    let rooms = game_server.get_room_list();
    HttpResponse::Ok().json(rooms)
}

#[tokio::main]
async fn main() {
    // コマンドライン引数の解析
    let args = Cli::parse();
    
    // 環境変数からの設定読み込み
    dotenv::dotenv().ok();
    
    // ロガーの初期化
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    log::info!("🎮 マルチプレイヤーゲームサーバーを起動中...");
    
    // 静的ファイルディレクトリの確認
    if !args.static_dir.exists() {
        log::warn!("⚠️ 静的ファイルディレクトリが存在しません: {:?}", args.static_dir);
        log::info!("💡 ディレクトリを作成します...");
        std::fs::create_dir_all(&args.static_dir).expect("静的ファイルディレクトリを作成できませんでした");
    }
    
    // ルームマネージャーの初期化
    let room_manager = Arc::new(RwLock::new(RoomManager::new()));
    
    // WebSocketサーバーの起動 (別スレッド)
    let ws_room_manager = room_manager.clone();
    let ws_port = args.ws_port;
    tokio::spawn(async move {
        websocket::start_websocket_server(ws_room_manager, ws_port).await;
    });
    
    // HTTPサーバーの起動 (メインスレッド)
    http::start_http_server(room_manager, args.static_dir, args.http_port).await;
}

// WebSocketサーバーの処理
async fn start_websocket_server(
    addr: String,
    game_server: Arc<Mutex<GameServer>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TCPリスナーの設定
    let listener = TcpListener::bind(&addr).await?;
    info!("WebSocket server listening on: {}", addr);
    
    // 接続ループ
    while let Ok((stream, addr)) = listener.accept().await {
        info!("New WebSocket connection from: {}", addr);
        let server = game_server.clone();
        
        // 各接続用に新しいタスクを生成
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr.to_string(), server).await {
                error!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
    
    Ok(())
}

// 個別のWebSocket接続の処理
async fn handle_connection(
    stream: TcpStream,
    addr: String,
    game_server: Arc<Mutex<GameServer>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    info!("WebSocket connection established: {}", addr);
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // 初期メッセージを送信
    let welcome_msg = message::ServerMessage::Welcome {
        client_id: uuid::Uuid::new_v4().to_string(),
        message: "Welcome to ECS WebAssembly Game Server".to_string(),
    };
    
    ws_sender.send(Message::Text(serde_json::to_string(&welcome_msg)?)).await?;
    
    // クライアントとのメッセージ処理ループ
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("Received message from {}: {}", addr, text);
                
                // メッセージの処理
                let response = match serde_json::from_str::<message::ClientMessage>(&text) {
                    Ok(client_msg) => {
                        process_message(client_msg, &addr, game_server.clone()).await?
                    },
                    Err(e) => {
                        warn!("Failed to parse message from {}: {}", addr, e);
                        message::ServerMessage::Error {
                            message: format!("Invalid message format: {}", e),
                        }
                    }
                };
                
                // レスポンスの送信
                ws_sender.send(Message::Text(serde_json::to_string(&response)?)).await?;
            },
            Ok(Message::Binary(data)) => {
                debug!("Received binary message from {}, length: {}", addr, data.len());
                // バイナリメッセージの処理は省略
            },
            Ok(Message::Ping(data)) => {
                // Pingに対してはPongを返信
                ws_sender.send(Message::Pong(data)).await?;
            },
            Ok(Message::Pong(_)) => {
                // Pongの処理は省略
            },
            Ok(Message::Close(reason)) => {
                info!("Connection closed by client {}: {:?}", addr, reason);
                break;
            },
            Err(e) => {
                error!("Error receiving message from {}: {}", addr, e);
                break;
            }
        }
    }
    
    info!("WebSocket connection closed: {}", addr);
    
    Ok(())
}

// クライアントメッセージの処理
async fn process_message(
    msg: message::ClientMessage,
    client_addr: &str,
    game_server: Arc<Mutex<GameServer>>,
) -> Result<message::ServerMessage, Box<dyn std::error::Error>> {
    let mut server = game_server.lock().await;
    
    match msg {
        message::ClientMessage::CreateRoom { player_name, game_type, settings } => {
            debug!("Client {} requested room creation", client_addr);
            match server.create_room(player_name, game_type, settings) {
                Ok(room) => Ok(message::ServerMessage::RoomCreated { room }),
                Err(e) => Ok(message::ServerMessage::Error { message: e.to_string() }),
            }
        },
        message::ClientMessage::JoinRoom { player_name, room_code } => {
            debug!("Client {} requested to join room {}", client_addr, room_code);
            match server.join_room(&room_code, player_name) {
                Ok((room_id, player_id)) => {
                    let room = server.get_room(&room_id)
                        .ok_or_else(|| "Room not found after joining".to_string())?;
                    Ok(message::ServerMessage::JoinedRoom {
                        room,
                        player_id: player_id.to_string(),
                    })
                },
                Err(e) => Ok(message::ServerMessage::Error { message: e.to_string() }),
            }
        },
        message::ClientMessage::LeaveRoom { player_id, room_id } => {
            debug!("Client {} requested to leave room", client_addr);
            match server.leave_room(&room_id, &player_id) {
                Ok(_) => Ok(message::ServerMessage::LeftRoom {
                    room_id,
                    player_id,
                }),
                Err(e) => Ok(message::ServerMessage::Error { message: e.to_string() }),
            }
        },
        message::ClientMessage::GameAction { room_id, player_id, action } => {
            debug!("Client {} sent game action", client_addr);
            match server.process_action(&room_id, &player_id, action) {
                Ok(result) => Ok(message::ServerMessage::ActionResult { result }),
                Err(e) => Ok(message::ServerMessage::Error { message: e.to_string() }),
            }
        },
        message::ClientMessage::Ping { timestamp } => {
            Ok(message::ServerMessage::Pong { timestamp })
        },
    }
} 