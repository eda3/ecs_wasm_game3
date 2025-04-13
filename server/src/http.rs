use std::convert::Infallible;
use std::path::Path;
use std::sync::Arc;
use warp::{http::StatusCode, Filter, Rejection, Reply};

use crate::websocket::SharedRoomManager;

/// HTTP API応答型
#[derive(serde::Serialize)]
struct ApiResponse {
    status: String,
    message: String,
}

/// 成功応答を生成
fn json_success(message: &str) -> impl Reply {
    warp::reply::json(&ApiResponse {
        status: "success".to_string(),
        message: message.to_string(),
    })
}

/// エラー応答を生成
fn json_error(message: &str) -> impl Reply {
    warp::reply::with_status(
        warp::reply::json(&ApiResponse {
            status: "error".to_string(),
            message: message.to_string(),
        }),
        StatusCode::BAD_REQUEST,
    )
}

/// ルームリストAPIハンドラー
async fn handle_get_rooms(room_manager: SharedRoomManager) -> Result<impl Reply, Rejection> {
    let manager = room_manager.read().await;
    let rooms = manager.list_public_rooms();
    
    // JSONレスポンスを返す
    Ok(warp::reply::json(&rooms))
}

/// サーバー情報APIハンドラー
async fn handle_server_info() -> Result<impl Reply, Infallible> {
    // サーバー情報を返す
    Ok(warp::reply::json(&serde_json::json!({
        "name": "WebSocketゲームサーバー",
        "version": env!("CARGO_PKG_VERSION"),
        "gameModes": ["minesweeper"],
        "maxPlayers": 50,
    })))
}

/// ヘルスチェックAPIハンドラー
async fn handle_health_check() -> Result<impl Reply, Infallible> {
    Ok(json_success("サーバーは稼働中です"))
}

/// 404エラーハンドラー
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let message = if err.is_not_found() {
        "リソースが見つかりません".to_string()
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        format!("リクエストボディが無効です: {}", e)
    } else {
        format!("サーバーエラー: {:?}", err)
    };

    Ok(warp::reply::with_status(
        json_error(&message),
        if err.is_not_found() {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        },
    ))
}

/// HTTPルーターを構築
pub fn create_http_routes(
    room_manager: SharedRoomManager,
    static_dir: impl AsRef<Path>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let static_dir = static_dir.as_ref().to_path_buf();
    
    // 静的ファイルルート
    let static_files = warp::path("static")
        .and(warp::fs::dir(static_dir.clone()));
    
    // SPAルート (index.htmlにフォールバック)
    let spa_route = warp::path::end()
        .or(warp::path::tail().and(warp::path::full().not_matches(warp::path("api"))))
        .map(move |_| {
            let index_path = static_dir.join("index.html");
            warp::fs::file(index_path)
        })
        .and_then(|file_reply| async {
            Ok::<_, Rejection>(file_reply)
        });
    
    // ルームマネージャーの共有
    let with_room_manager = warp::any().map(move || room_manager.clone());
    
    // APIルート
    let api_routes = warp::path("api")
        .and(
            // GET /api/rooms - 公開ルーム一覧
            warp::path("rooms")
                .and(warp::get())
                .and(with_room_manager.clone())
                .and_then(handle_get_rooms)
                .or(
                    // GET /api/health - ヘルスチェック
                    warp::path("health")
                        .and(warp::get())
                        .and_then(handle_health_check)
                )
                .or(
                    // GET /api/info - サーバー情報
                    warp::path("info")
                        .and(warp::get())
                        .and_then(handle_server_info)
                )
        );
    
    // 全ルートの結合
    static_files
        .or(api_routes)
        .or(spa_route)
        .recover(handle_rejection)
}

/// HTTPサーバーを起動
pub async fn start_http_server(
    room_manager: SharedRoomManager,
    static_dir: impl AsRef<Path>,
    port: u16,
) {
    println!("🌐 HTTPサーバーをポート{}で起動中...", port);
    
    let routes = create_http_routes(room_manager, static_dir);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
} 