//! ネットワーククライアント実装
//! 
//! このモジュールは、WebSocketを使用したクライアント側のネットワーク通信機能を実装します。
//! サーバーとの接続管理、メッセージの送受信、状態同期などの機能を提供します。

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent};
use js_sys::{Function, Date, Array, JSON};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;

use super::protocol::{NetworkMessage, MessageType};
use super::messages::{InputData, PlayerData, EntitySnapshot, ComponentData};
use super::{ConnectionState, NetworkError, TimeSyncData, NetworkConfig};
use crate::ecs::{World, Entity, Component, Resource};

/// ネットワークコンポーネント（エンティティに付与される）
#[derive(Debug, Clone, Component)]
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
        }
    }

    /// サーバーに接続
    pub fn connect(&mut self) -> Result<(), NetworkError> {
        if self.connection_state == ConnectionState::Connected || 
           self.connection_state == ConnectionState::Connecting {
            return Ok(());
        }

        self.connection_state = ConnectionState::Connecting;
        self.connection_attempts += 1;

        // WebSocketの作成
        let ws = match WebSocket::new(&self.config.server_url) {
            Ok(ws) => ws,
            Err(err) => {
                let error_msg = format!("WebSocket接続の作成に失敗: {:?}", err);
                self.connection_state = ConnectionState::Error(error_msg.clone());
                return Err(NetworkError::ConnectionError(error_msg));
            }
        };

        // イベントハンドラの設定
        let on_open = Closure::wrap(Box::new(move |_| {
            // 接続が確立されたときの処理
            web_sys::console::log_1(&"WebSocket接続が確立されました".into());
        }) as Box<dyn FnMut(JsValue)>);

        let client_ptr = Rc::new(RefCell::new(self as *mut NetworkClient));
        
        // メッセージ受信ハンドラ
        let on_message_client = Rc::clone(&client_ptr);
        let on_message = Closure::wrap(Box::new(move |event: MessageEvent| {
            let client = unsafe { &mut *on_message_client.borrow_mut() };
            if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
                let text_str = String::from(text);
                match NetworkMessage::from_json(&text_str) {
                    Ok(message) => {
                        unsafe {
                            (*(*client)).message_queue.push_back(message);
                        }
                    },
                    Err(err) => {
                        web_sys::console::error_1(&format!("メッセージの解析エラー: {:?}", err).into());
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        // エラーハンドラ
        let on_error_client = Rc::clone(&client_ptr);
        let on_error = Closure::wrap(Box::new(move |event: ErrorEvent| {
            let client = unsafe { &mut *on_error_client.borrow_mut() };
            let error_msg = format!("WebSocketエラー: {:?}", event);
            unsafe {
                (*(*client)).connection_state = ConnectionState::Error(error_msg.clone());
                (*(*client)).last_error = Some(error_msg);
            }
            web_sys::console::error_1(&"WebSocketエラーが発生しました".into());
        }) as Box<dyn FnMut(ErrorEvent)>);

        // 切断ハンドラ
        let on_close_client = Rc::clone(&client_ptr);
        let on_close = Closure::wrap(Box::new(move |event: CloseEvent| {
            let client = unsafe { &mut *on_close_client.borrow_mut() };
            unsafe {
                (*(*client)).connection_state = ConnectionState::Disconnected;
            }
            let code = event.code();
            let reason = event.reason();
            web_sys::console::log_2(
                &format!("WebSocket接続が閉じられました（コード: {}）", code).into(),
                &reason.into()
            );
        }) as Box<dyn FnMut(CloseEvent)>);

        // イベントハンドラをWebSocketにセット
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));

        // イベントハンドラをリークさせないようにする
        on_open.forget();
        on_message.forget();
        on_error.forget();
        on_close.forget();

        self.connection = Some(ws);
        
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

    /// メッセージを送信
    pub fn send_message(&mut self, message: NetworkMessage) -> Result<(), NetworkError> {
        if self.connection_state != ConnectionState::Connected {
            self.pending_messages.push_back(message);
            return Ok(());
        }

        if let Some(ws) = &self.connection {
            if let Ok(json) = message.to_json() {
                if let Err(err) = ws.send_with_str(&json) {
                    let error_msg = format!("メッセージ送信エラー: {:?}", err);
                    self.pending_messages.push_back(message);
                    return Err(NetworkError::MessageProcessingError(error_msg));
                }
            } else {
                let error_msg = "メッセージのJSON変換に失敗".to_string();
                return Err(NetworkError::MessageProcessingError(error_msg));
            }
        } else {
            self.pending_messages.push_back(message);
        }
        
        Ok(())
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
        self.process_messages(world);
        
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
                    self.connect().ok();
                } else {
                    // 再接続試行回数を超えた場合
                    self.connection_state = ConnectionState::Error("接続タイムアウト".to_string());
                }
            }
        }
    }

    /// 受信メッセージの処理
    fn process_messages(&mut self, world: &mut World) {
        while let Some(message) = self.message_queue.pop_front() {
            match message.message_type {
                MessageType::ConnectResponse { player_id, success, message: msg } => {
                    if success {
                        self.connection_state = ConnectionState::Connected;
                        self.player_id = Some(player_id);
                        self.connected_at = Some(Date::now());
                        
                        if self.config.debug_mode {
                            web_sys::console::log_1(&format!("接続成功: Player ID = {}", player_id).into());
                        }
                    } else {
                        let error_msg = format!("接続拒否: {}", msg.unwrap_or_default());
                        self.connection_state = ConnectionState::Error(error_msg.clone());
                        self.last_error = Some(error_msg);
                    }
                },
                MessageType::EntityCreate { entity_id } => {
                    // 新しいエンティティの作成
                    // 実際の実装ではWorldにエンティティを追加する処理が必要
                    if self.config.debug_mode {
                        web_sys::console::log_1(&format!("エンティティ作成: {}", entity_id).into());
                    }
                },
                MessageType::EntityDelete { entity_id } => {
                    // エンティティの削除
                    // 実際の実装ではWorldからエンティティを削除する処理が必要
                    if self.config.debug_mode {
                        web_sys::console::log_1(&format!("エンティティ削除: {}", entity_id).into());
                    }
                },
                MessageType::ComponentUpdate => {
                    // コンポーネント更新の処理
                    if let Some(entity_id) = message.entity_id {
                        if let Some(components) = message.components {
                            // エンティティスナップショットの作成と保存
                            let mut snapshot = EntitySnapshot::new(entity_id, message.timestamp);
                            for (name, data) in components {
                                snapshot.add_component(&name, data);
                            }
                            
                            if let Some(owner) = message.player_id {
                                snapshot.set_owner(owner);
                            }
                            
                            // スナップショットを保存
                            self.entity_snapshots
                                .entry(entity_id)
                                .or_insert_with(Vec::new)
                                .push(snapshot);
                                
                            // 古いスナップショットを削除
                            self.cleanup_old_snapshots(entity_id);
                        }
                    }
                },
                MessageType::TimeSync { client_time, server_time } => {
                    // 時間同期の処理
                    let now = Date::now();
                    let rtt = now - client_time;
                    self.rtt = rtt;
                    
                    // サーバー時間とクライアント時間の差を計算
                    // RTTの半分をオフセットとして使用
                    let time_offset = server_time - (now - rtt / 2.0);
                    self.time_sync_data.time_offset = time_offset;
                    self.time_sync_data.rtt = rtt;
                    self.time_sync_data.last_sync = now;
                    
                    if self.config.debug_mode {
                        web_sys::console::log_1(&format!("時間同期: オフセット = {}ms, RTT = {}ms", 
                                           time_offset, rtt).into());
                    }
                },
                MessageType::Pong { client_time, server_time } => {
                    // Pongメッセージの処理
                    let now = Date::now();
                    let rtt = now - client_time;
                    self.rtt = rtt;
                    
                    if self.config.debug_mode {
                        web_sys::console::log_1(&format!("Pong: RTT = {}ms", rtt).into());
                    }
                },
                MessageType::Error { code, message: error_msg } => {
                    // エラーメッセージの処理
                    self.last_error = Some(format!("サーバーエラー ({}): {}", code, error_msg));
                    web_sys::console::error_1(&format!("サーバーエラー: {}", error_msg).into());
                },
                _ => {}
            }
        }
    }

    /// 古いスナップショットのクリーンアップ
    fn cleanup_old_snapshots(&mut self, entity_id: u32) {
        const MAX_SNAPSHOTS: usize = 20; // エンティティごとに保持する最大スナップショット数
        
        if let Some(snapshots) = self.entity_snapshots.get_mut(&entity_id) {
            if snapshots.len() > MAX_SNAPSHOTS {
                // 古い順にソート
                snapshots.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
                
                // 最大数を超えた分を削除
                snapshots.drain(0..(snapshots.len() - MAX_SNAPSHOTS));
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
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        
        while let Some(message) = self.pending_messages.pop_front() {
            if let Err(err) = self.send_message(message) {
                web_sys::console::error_1(&format!("保留メッセージの送信エラー: {:?}", err).into());
                break;
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