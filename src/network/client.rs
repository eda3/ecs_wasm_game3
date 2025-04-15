//! ネットワーククライアント実装
//! 
//! このモジュールは、WebSocketを使用したクライアント側のネットワーク通信機能を実装します。
//! サーバーとの接続管理、メッセージの送受信、状態同期などの機能を提供します。

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent, Event};
use js_sys::Date;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;
use log;

use super::protocol::{NetworkMessage, MessageType};
use super::messages::{InputData, PlayerData, EntitySnapshot};
use super::{ConnectionState, NetworkError, TimeSyncData, NetworkConfig};
use crate::ecs::World;

/// ネットワークコンポーネント（エンティティに付与される）
#[derive(Debug, Clone)]
pub struct NetworkComponent {
    /// エンティティIDがネットワーク全体で同期されているか
    pub is_synced: bool,
    /// 最後の同期時刻
    pub last_sync_time: f64,
    /// 補間係数
    pub interpolation_factor: f32,
    /// リモートエンティティか（他のプレイヤーから同期されたもの）
    pub is_remote: bool,
    /// このエンティティの所有者ID
    pub owner_id: Option<u32>,
}

impl Default for NetworkComponent {
    fn default() -> Self {
        Self {
            is_synced: false,
            last_sync_time: 0.0,
            interpolation_factor: 0.0,
            is_remote: false,
            owner_id: None,
        }
    }
}

/// ネットワーククライアント
#[derive(Clone)]
pub struct NetworkClient {
    /// WebSocket接続
    connection: Option<WebSocket>,
    /// 接続状態
    connection_state: ConnectionState,
    /// 受信メッセージキュー
    message_queue: VecDeque<NetworkMessage>,
    /// 送信待ちメッセージキュー
    pending_messages: VecDeque<NetworkMessage>,
    /// プレイヤーID
    player_id: Option<u32>,
    /// シーケンス番号カウンタ
    sequence_number: u32,
    /// 往復遅延時間（ms）
    rtt: f64,
    /// 時間同期データ
    time_sync_data: TimeSyncData,
    /// エンティティスナップショットキャッシュ
    entity_snapshots: HashMap<u32, Vec<EntitySnapshot>>,
    /// 他プレイヤーのプレイヤーデータ
    players: HashMap<u32, PlayerData>,
    /// ネットワーク設定
    config: NetworkConfig,
    /// エラーメッセージ
    last_error: Option<String>,
    /// 接続が確立された時刻
    connected_at: Option<f64>,
    /// 最後にPingを送信した時刻
    last_ping_time: Option<f64>,
    /// 接続試行回数
    connection_attempts: u32,
    /// サーバーURL
    server_url: String,
}

impl NetworkClient {
    /// 新しいネットワーククライアントを作成
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            connection: None,
            connection_state: ConnectionState::Disconnected,
            message_queue: VecDeque::new(),
            pending_messages: VecDeque::new(),
            player_id: None,
            sequence_number: 0,
            rtt: 0.0,
            time_sync_data: TimeSyncData::default(),
            entity_snapshots: HashMap::new(),
            players: HashMap::new(),
            config,
            last_error: None,
            connected_at: None,
            last_ping_time: None,
            connection_attempts: 0,
            server_url: String::new(),
        }
    }

    /// サーバーに接続
    pub fn connect(&mut self, url: &str) -> Result<(), NetworkError> {
        if self.connection_state == ConnectionState::Connected {
            return Ok(());
        }

        self.connection_state = ConnectionState::Connecting;
        self.server_url = url.to_string();

        // WebSocketの作成
        let ws = match WebSocket::new(&self.server_url) {
            Ok(ws) => ws,
            Err(err) => {
                let error_msg = format!("WebSocket作成に失敗: {:?}", err);
                log::error!("{}", error_msg);
                self.connection_state = ConnectionState::Disconnected;
                return Err(NetworkError::ConnectionError(error_msg));
            }
        };

        // バイナリ形式を設定
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        // 自己参照のクロージャを回避するために弱参照を作成
        let message_queue = Rc::new(RefCell::new(self.message_queue.clone()));
        let message_queue_weak = Rc::downgrade(&message_queue);
        let connection_state = Rc::new(RefCell::new(self.connection_state.clone()));

        // WebSocketが開いたときのコールバック
        let connection_state_clone = connection_state.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_event: Event| {
            log::info!("🌐 WebSocket接続完了！");
            // 接続状態を更新
            if let Ok(mut state) = connection_state_clone.try_borrow_mut() {
                *state = ConnectionState::Connected;
            }
        }) as Box<dyn FnMut(Event)>);

        // メッセージを受信したときのコールバック
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            // メッセージキューが存在する場合のみ処理
            if let Some(message_queue) = message_queue_weak.upgrade() {
                if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
                    let text_str = text.as_string().unwrap();
                    match NetworkMessage::from_json(&text_str) {
                        Ok(message) => {
                            log::debug!("📩 メッセージ受信: {:?}", message);
                            // 安全にメッセージをキューに追加
                            message_queue.borrow_mut().push_back(message);
                        }
                        Err(err) => {
                            log::error!("❌ メッセージのパースに失敗: {:?}", err);
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        // エラーが発生したときのコールバック
        let onerror_callback = Closure::wrap(Box::new(move |event: ErrorEvent| {
            log::error!("❌ WebSocketエラー: {:?}", event);
        }) as Box<dyn FnMut(ErrorEvent)>);

        // WebSocketが閉じたときのコールバック
        let onclose_callback = Closure::wrap(Box::new(move |event: CloseEvent| {
            log::warn!("🔌 WebSocket切断: コード={}, 理由={}", event.code(), event.reason());
        }) as Box<dyn FnMut(CloseEvent)>);

        // コールバックの設定
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));

        // コールバックのリーク防止（クロージャをメモリに保持）
        onopen_callback.forget();
        onmessage_callback.forget();
        onerror_callback.forget();
        onclose_callback.forget();

        // 接続の保存
        self.connection = Some(ws);
        self.connected_at = Some(js_sys::Date::now() as f64);

        log::info!("🔄 サーバーに接続中: {}", url);
        Ok(())
    }

    /// サーバーから切断
    pub fn disconnect(&mut self) -> Result<(), NetworkError> {
        // connection と状態を先に取得して保存
        let connection_clone = self.connection.clone();
        let is_connected = self.connection_state == ConnectionState::Connected;
        
        if let Some(ws) = connection_clone {
            if is_connected {
                // シーケンス番号を取得
                let next_seq = self.next_sequence_number();
                
                // 切断メッセージを送信
                let disconnect_msg = NetworkMessage::new(MessageType::Disconnect { reason: None })
                    .with_sequence(next_seq);
                if let Ok(json) = disconnect_msg.to_json() {
                    if let Err(err) = ws.send_with_str(&json) {
                        web_sys::console::error_1(&format!("切断メッセージの送信エラー: {:?}", err).into());
                    }
                }
                
                // 接続を閉じる
                if let Err(err) = ws.close() {
                    let error_msg = format!("WebSocket接続のクローズに失敗: {:?}", err);
                    return Err(NetworkError::ConnectionError(error_msg));
                }
            }
        }
        
        self.connection_state = ConnectionState::Disconnected;
        self.connection = None;
        self.player_id = None;
        
        Ok(())
    }

    /// メッセージをサーバーに送信します。
    /// 接続が確立されていない場合は、メッセージを保留キューに追加します。
    pub fn send_message(&mut self, mut message: NetworkMessage) -> Result<(), NetworkError> {
        // シーケンス番号とタイムスタンプを先に設定
        let next_seq = self.next_sequence_number();
        message.sequence = Some(next_seq);
        message.timestamp = js_sys::Date::now() as f64;

        if let Some(ws) = &self.connection {
            // WebSocketの状態を確認
            match ws.ready_state() {
                WebSocket::OPEN => {
                    // メッセージをJSONに変換
                    let json_message = match message.to_json() {
                        Ok(json) => json,
                        Err(e) => {
                            log::error!("メッセージのシリアライズに失敗: {:?}", e);
                            return Err(NetworkError::MessageProcessingError("メッセージのシリアライズに失敗".to_string()));
                        }
                    };

                    // メッセージを送信
                    match ws.send_with_str(&json_message) {
                        Ok(_) => {
                            log::debug!("📤 メッセージ送信: {:?}", message);
                            Ok(())
                        }
                        Err(err) => {
                            log::error!("メッセージ送信エラー: {:?}", err);
                            // エラーが発生した場合も一旦保留キューに入れる (再接続後に送信試行)
                            self.pending_messages.push_back(message);
                            Err(NetworkError::MessageProcessingError(format!("メッセージ送信エラー: {:?}", err)))
                        }
                    }
                }
                WebSocket::CONNECTING => {
                    // 接続中の場合は保留キューへ
                    log::warn!("接続中のためメッセージを保留: {:?}", message);
                    self.pending_messages.push_back(message);
                    Ok(())
                }
                _ => {
                    // その他の状態（CLOSING, CLOSED）の場合はエラーまたは保留
                    log::error!("接続が確立されていないためメッセージを送信できません (状態: {})。メッセージを保留します。", ws.ready_state());
                    self.pending_messages.push_back(message);
                    // Err(NetworkError::ConnectionError("接続が確立されていません".to_string()))
                    Ok(()) // エラーではなく保留にする
                }
            }
        } else {
            // 接続オブジェクト自体がない場合は保留キューへ
            log::warn!("接続がないためメッセージを保留: {:?}", message);
            self.pending_messages.push_back(message);
            Ok(())
        }
    }

    /// 入力データを送信
    pub fn send_input(&mut self, input: InputData) -> Result<(), NetworkError> {
        let message = NetworkMessage::new(MessageType::Input)
            .with_sequence(self.next_sequence_number())
            .with_player_id(self.player_id.unwrap_or(0))
            .with_input(input);
            
        self.send_message(message)
    }

    /// 更新処理
    pub fn update(&mut self, world: &mut World) -> Result<(), NetworkError> {
        // 接続状態の確認
        self.check_connection_status();
        
        // 受信メッセージの処理
        self.process_messages();
        
        // 接続されている場合の定期処理
        if self.connection_state == ConnectionState::Connected {
            // 時間同期
            self.update_time_sync();
            
            // 保留中のメッセージの送信
            self.send_pending_messages();
        }
        
        Ok(())
    }

    /// 接続状態の確認
    fn check_connection_status(&mut self) {
        // 接続中の場合、タイムアウトをチェック
        if self.connection_state == ConnectionState::Connecting {
            let now = Date::now();
            let connected_since = self.connected_at.unwrap_or(now);
            if now - connected_since > self.config.connection_timeout_ms as f64 {
                // タイムアウト - 再接続を試みる
                if self.connection_attempts < self.config.reconnect_attempts {
                    self.disconnect().ok();
                    // 再帰的な参照を避けるために一時変数にURLを保存
                    let server_url = self.server_url.clone();
                    self.connect(&server_url).ok();
                } else {
                    // 再接続試行回数を超えた場合
                    self.connection_state = ConnectionState::Error("接続タイムアウト".to_string());
                }
            }
        }
    }

    /// メッセージキューを処理し、各メッセージに対して適切なアクションを実行
    pub fn process_messages(&mut self) {
        // 接続が確立されていない場合は処理しない
        if self.connection_state != ConnectionState::Connected {
            return;
        }

        // キューからすべてのメッセージを取り出し処理する
        let message_count = self.message_queue.len();
        if message_count > 0 {
            web_sys::console::log_1(&format!("処理するメッセージ数: {}", message_count).into());
        }

        for _ in 0..message_count {
            if let Some(message) = self.message_queue.pop_front() {
                match message.message_type {
                    MessageType::ConnectResponse { player_id, success, message: msg } => {
                        if success {
                            web_sys::console::log_1(&format!("クライアント接続: ID={}", player_id).into());
                            // クライアントIDを設定
                            self.player_id = Some(player_id);
                            self.connected_at = Some(Date::now());
                            web_sys::console::log_1(&format!("自身のクライアントID設定: {}", player_id).into());
                        } else {
                            web_sys::console::error_1(&format!("接続失敗: {}", msg.unwrap_or_default()).into());
                            self.player_id = None;
                            self.connected_at = None;
                        }
                    }
                    MessageType::Disconnect { reason } => {
                        web_sys::console::log_1(&format!("クライアント切断: {:?}", reason).into());
                        // 接続の切断を処理
                        self.player_id = None;
                        self.connected_at = None;
                    }
                    MessageType::EntityCreate { entity_id } => {
                        web_sys::console::log_1(&format!("エンティティ作成: ID={}", entity_id).into());
                        // ここでエンティティ作成処理を実装
                    }
                    MessageType::EntityDelete { entity_id } => {
                        web_sys::console::log_1(&format!("エンティティ削除: ID={}", entity_id).into());
                        // ここでエンティティ削除処理を実装
                    }
                    MessageType::ComponentUpdate => {
                        // コンポーネント更新の処理
                        web_sys::console::log_1(&"コンポーネント更新メッセージを受信".into());
                        if let Some(entity_id) = message.entity_id {
                            if let Some(components) = &message.components {
                                web_sys::console::log_1(&format!("エンティティ{}のコンポーネント更新", entity_id).into());
                                // コンポーネント更新の処理を実装
                            }
                        }
                    }
                    MessageType::Input => {
                        // 入力メッセージの処理
                        web_sys::console::log_1(&"入力メッセージを受信".into());
                        if let Some(player_id) = message.player_id {
                            if player_id != self.player_id.unwrap_or(0) { // 自分の入力はスキップ
                                web_sys::console::log_1(&format!("プレイヤー{}からの入力", player_id).into());
                                // 入力処理を実装
                            }
                        }
                    }
                    MessageType::TimeSync { client_time, server_time } => {
                        // 時間同期の処理
                        let now = Date::now();
                        let rtt = now - client_time;
                        self.rtt = rtt;
                        
                        // サーバー時間とクライアント時間の差を計算
                        let time_offset = server_time - (now - rtt / 2.0);
                        self.time_sync_data.time_offset = time_offset;
                        self.time_sync_data.rtt = rtt;
                        self.time_sync_data.last_sync = now;
                        
                        web_sys::console::log_1(&format!("時間同期: オフセット = {}ms, RTT = {}ms", 
                                                        time_offset, rtt).into());
                    }
                    MessageType::Ping { client_time } => {
                        // Pingメッセージの処理
                        web_sys::console::log_1(&format!("Pingメッセージを受信: {}", client_time).into());
                        // 必要に応じてPong応答を送信
                    }
                    MessageType::Pong { client_time, server_time: _ } => {
                        // Pongメッセージの処理
                        let now = Date::now();
                        let rtt = now - client_time;
                        self.rtt = rtt;
                        
                        web_sys::console::log_1(&format!("Pong: RTT = {}ms", rtt).into());
                    }
                    MessageType::Error { code, message: error_msg } => {
                        // エラーメッセージの処理
                        web_sys::console::error_1(&format!("サーバーエラー ({}): {}", code, error_msg).into());
                        self.last_error = Some(format!("サーバーエラー ({}): {}", code, error_msg));
                    }
                    MessageType::Connect => {
                        // Connectメッセージは通常クライアントからサーバーに送信されるもの
                        web_sys::console::warn_1(&"サーバーからConnectメッセージを受信（異常）".into());
                    }
                    _ => {
                        // 未知のメッセージタイプ
                        web_sys::console::warn_1(&format!("未知のメッセージタイプ: {:?}", message.message_type).into());
                    }
                }
            }
        }
    }

    /// 時間同期の更新
    fn update_time_sync(&mut self) {
        const TIME_SYNC_INTERVAL: f64 = 5000.0; // 時間同期の間隔（ミリ秒）
        
        let now = Date::now();
        let last_sync = self.time_sync_data.last_sync;
        
        if now - last_sync > TIME_SYNC_INTERVAL {
            // 時間同期メッセージを送信
            let message = NetworkMessage::new(MessageType::TimeSync { 
                client_time: now,
                server_time: 0.0, // サーバーが設定する値
            }).with_sequence(self.next_sequence_number());
            
            self.send_message(message).ok();
        }
        
        // Pingの送信
        if self.last_ping_time.is_none() || now - self.last_ping_time.unwrap() > 1000.0 {
            let message = NetworkMessage::new(MessageType::Ping { 
                client_time: now,
            }).with_sequence(self.next_sequence_number());
            
            self.send_message(message).ok();
            self.last_ping_time = Some(now);
        }
    }

    /// 保留中のメッセージを送信
    fn send_pending_messages(&mut self) {
        // 接続状態の確認 - 接続済みの場合のみ処理
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        
        // WebSocketの状態を確認 - OPEN状態の場合のみ処理
        if let Some(ws) = &self.connection {
            if ws.ready_state() != WebSocket::OPEN {
                return;
            }
            
            while let Some(message) = self.pending_messages.pop_front() {
                if let Err(err) = self.send_message(message) {
                    web_sys::console::error_1(&format!("保留メッセージの送信エラー: {:?}", err).into());
                    break;
                }
            }
        }
    }

    /// 次のシーケンス番号を取得
    fn next_sequence_number(&mut self) -> u32 {
        let seq = self.sequence_number;
        self.sequence_number = self.sequence_number.wrapping_add(1);
        seq
    }

    /// プレイヤーIDを取得
    pub fn get_player_id(&self) -> Option<u32> {
        self.player_id
    }

    /// 接続状態を取得
    pub fn get_connection_state(&self) -> &ConnectionState {
        &self.connection_state
    }

    /// RTTを取得
    pub fn get_rtt(&self) -> f64 {
        self.rtt
    }

    /// 最後のエラーメッセージを取得
    pub fn get_last_error(&self) -> Option<&String> {
        self.last_error.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_client_creation() {
        let config = NetworkConfig::default();
        let client = NetworkClient::new(config);
        
        assert_eq!(*client.get_connection_state(), ConnectionState::Disconnected);
        assert_eq!(client.get_player_id(), None);
    }

    #[test]
    fn test_sequence_number_generation() {
        let config = NetworkConfig::default();
        let mut client = NetworkClient::new(config);
        
        let seq1 = client.next_sequence_number();
        let seq2 = client.next_sequence_number();
        
        assert_eq!(seq2, seq1 + 1);
    }
} 