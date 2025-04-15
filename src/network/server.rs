//! ネットワークサーバー実装
//! 
//! このモジュールは、サーバー側のネットワーク通信機能を実装します。
//! ただし、WebAssemblyコンテキストでは主にスタブとして機能し、実際のサーバーは別プロセスで実行されます。

use std::collections::{HashMap, VecDeque};
use js_sys::Date;

use super::protocol::{NetworkMessage, MessageType};
use super::messages::{PlayerData, ComponentData};
use super::{ConnectionState, NetworkError, NetworkConfig};
use crate::ecs::World;

/// サーバー接続クライアント情報
#[derive(Debug, Clone)]
pub struct ServerClient {
    /// クライアントID
    pub id: u32,
    /// プレイヤーデータ
    pub player_data: PlayerData,
    /// 接続状態
    pub connection_state: ConnectionState,
    /// 最後のメッセージを受信した時刻
    pub last_message_time: f64,
    /// クライアントのシーケンス番号
    pub sequence_number: u32,
    /// 往復遅延時間(RTT)
    pub rtt: f64,
}

/// サーバーモードを表す列挙型
#[derive(Debug, Clone, PartialEq)]
pub enum ServerMode {
    /// 実サーバーモード（別プロセスで動作）
    RemoteServer,
    /// ローカルシミュレーション（テスト用）
    LocalSimulation,
    /// P2Pサーバーモード（WebRTC対応時に使用）
    P2PHost,
}

/// ネットワークサーバー
/// 
/// この実装はWebAssemblyコンテキストで動作することを前提としています。
/// 実際のサーバー実装は別の実行環境で行われることが多いですが、
/// ここではローカルシミュレーションやP2P接続のホスト側としての機能を提供します。
pub struct NetworkServer {
    /// サーバーモード
    pub mode: ServerMode,
    /// クライアント一覧
    pub clients: HashMap<u32, ServerClient>,
    /// 受信メッセージキュー
    pub message_queue: VecDeque<(u32, NetworkMessage)>,
    /// 送信待ちメッセージキュー
    pub pending_messages: VecDeque<(Option<u32>, NetworkMessage)>,
    /// 次のクライアントID
    pub next_client_id: u32,
    /// シーケンス番号カウンタ
    pub sequence_number: u32,
    /// サーバーの現在時刻（論理時間）
    pub server_time: f64,
    /// サーバー設定
    pub config: NetworkConfig,
    /// サーバー状態
    pub active: bool,
}

impl NetworkServer {
    /// 新しいネットワークサーバーを作成
    pub fn new(config: NetworkConfig, mode: ServerMode) -> Self {
        Self {
            mode,
            clients: HashMap::new(),
            message_queue: VecDeque::new(),
            pending_messages: VecDeque::new(),
            next_client_id: 1,
            sequence_number: 0,
            server_time: Date::now(),
            config,
            active: false,
        }
    }

    /// サーバーを起動
    pub fn start(&mut self) -> Result<(), NetworkError> {
        if self.active {
            return Ok(());
        }
        
        match self.mode {
            ServerMode::RemoteServer => {
                return Err(NetworkError::ConnectionError(
                    "リモートサーバーモードはWebAssemblyコンテキストで直接サポートされていません".to_string()
                ));
            },
            ServerMode::LocalSimulation => {
                // ローカルシミュレーションの初期化
                web_sys::console::log_1(&"ローカルサーバーシミュレーションを開始しました".into());
            },
            ServerMode::P2PHost => {
                // P2Pホストモードの初期化
                // WebRTC関連の初期化など
                web_sys::console::log_1(&"P2Pホストモードを開始しました".into());
            }
        }
        
        self.active = true;
        Ok(())
    }

    /// サーバーを停止
    pub fn stop(&mut self) -> Result<(), NetworkError> {
        if !self.active {
            return Ok(());
        }
        
        // すべてのクライアントに切断メッセージを送信
        for client_id in self.clients.keys().copied().collect::<Vec<_>>() {
            self.disconnect_client(client_id, Some("サーバーシャットダウン".to_string()))?;
        }
        
        self.active = false;
        web_sys::console::log_1(&"サーバーを停止しました".into());
        
        Ok(())
    }

    /// クライアントを接続
    pub fn connect_client(&mut self, player_data: PlayerData) -> Result<u32, NetworkError> {
        if !self.active {
            return Err(NetworkError::ConnectionError("サーバーが起動していません".to_string()));
        }
        
        let client_id = self.next_client_id;
        self.next_client_id += 1;
        
        let client = ServerClient {
            id: client_id,
            player_data,
            connection_state: ConnectionState::Connected,
            last_message_time: Date::now(),
            sequence_number: 0,
            rtt: 0.0,
        };
        
        self.clients.insert(client_id, client);
        
        // 接続応答メッセージをキューに追加
        let response = NetworkMessage::new(MessageType::ConnectResponse {
            player_id: client_id,
            success: true,
            message: None,
        }).with_sequence(self.next_sequence_number());
        
        self.pending_messages.push_back((Some(client_id), response));
        
        if self.config.debug_mode {
            web_sys::console::log_1(&format!("クライアント {} が接続しました", client_id).into());
        }
        
        Ok(client_id)
    }

    /// クライアントを切断
    pub fn disconnect_client(&mut self, client_id: u32, reason: Option<String>) -> Result<(), NetworkError> {
        if let Some(client) = self.clients.get_mut(&client_id) {
            client.connection_state = ConnectionState::Disconnecting;
            
            // 切断メッセージをキューに追加
            let disconnect_msg = NetworkMessage::new(MessageType::Disconnect {
                reason: reason.clone(),
            }).with_sequence(self.next_sequence_number());
            
            self.pending_messages.push_back((Some(client_id), disconnect_msg));
            
            // クライアントをリストから削除
            self.clients.remove(&client_id);
            
            if self.config.debug_mode {
                web_sys::console::log_1(&format!("クライアント {} が切断しました: {:?}", client_id, reason).into());
            }
        }
        
        Ok(())
    }

    /// メッセージをクライアントに送信
    pub fn send_message(&mut self, client_id: Option<u32>, message: NetworkMessage) -> Result<(), NetworkError> {
        if !self.active {
            return Err(NetworkError::ConnectionError("サーバーが起動していません".to_string()));
        }
        
        // 特定のクライアントが指定されている場合は、そのクライアントが接続中か確認
        if let Some(id) = client_id {
            if !self.clients.contains_key(&id) {
                return Err(NetworkError::ConnectionError(format!("クライアント {} は接続されていません", id)));
            }
        }
        
        // メッセージをキューに追加
        self.pending_messages.push_back((client_id, message));
        
        Ok(())
    }

    /// ブロードキャストメッセージ送信
    pub fn broadcast_message(&mut self, message: NetworkMessage, exclude_client: Option<u32>) -> Result<(), NetworkError> {
        for client_id in self.clients.keys().copied().collect::<Vec<_>>() {
            if let Some(excluded) = exclude_client {
                if client_id == excluded {
                    continue;
                }
            }
            
            let client_message = message.clone();
            self.send_message(Some(client_id), client_message)?;
        }
        
        Ok(())
    }

    /// エンティティ作成メッセージ送信
    pub fn send_entity_create(&mut self, entity_id: u32, client_id: Option<u32>) -> Result<(), NetworkError> {
        let message = NetworkMessage::new(MessageType::EntityCreate { entity_id })
            .with_sequence(self.next_sequence_number())
            .with_entity_id(entity_id);
            
        if let Some(id) = client_id {
            self.send_message(Some(id), message)
        } else {
            self.broadcast_message(message, None)
        }
    }

    /// エンティティ削除メッセージ送信
    pub fn send_entity_delete(&mut self, entity_id: u32, client_id: Option<u32>) -> Result<(), NetworkError> {
        let message = NetworkMessage::new(MessageType::EntityDelete { entity_id })
            .with_sequence(self.next_sequence_number())
            .with_entity_id(entity_id);
            
        if let Some(id) = client_id {
            self.send_message(Some(id), message)
        } else {
            self.broadcast_message(message, None)
        }
    }

    /// コンポーネント更新メッセージ送信
    pub fn send_component_update(&mut self, entity_id: u32, components: HashMap<String, ComponentData>, client_id: Option<u32>) -> Result<(), NetworkError> {
        let message = NetworkMessage::new(MessageType::ComponentUpdate)
            .with_sequence(self.next_sequence_number())
            .with_entity_id(entity_id)
            .with_components(components);
            
        if let Some(id) = client_id {
            self.send_message(Some(id), message)
        } else {
            self.broadcast_message(message, None)
        }
    }

    /// 更新処理
    pub fn update(&mut self, world: &mut World, delta_time: f32) -> Result<(), NetworkError> {
        if !self.active {
            return Ok(());
        }
        
        // サーバー時間の更新
        self.server_time += delta_time as f64 * 1000.0; // ミリ秒単位
        
        // 受信メッセージの処理
        self.process_messages(world);
        
        // 送信キューの処理
        self.process_pending_messages();
        
        // クライアントの状態チェック
        self.check_clients();
        
        Ok(())
    }

    /// 受信メッセージの処理
    fn process_messages(&mut self, world: &mut World) {
        while let Some((client_id, message)) = self.message_queue.pop_front() {
            // クライアントが存在するか確認
            if !self.clients.contains_key(&client_id) {
                continue;
            }
            
            // クライアントの最終メッセージ受信時間を更新
            if let Some(client) = self.clients.get_mut(&client_id) {
                client.last_message_time = Date::now();
                
                // シーケンス番号を更新（必要に応じて）
                if let Some(seq) = message.sequence {
                    if seq > client.sequence_number {
                        client.sequence_number = seq;
                    }
                }
            }
            
            match message.message_type {
                MessageType::Connect => {
                    // 接続メッセージの処理
                    // すでに接続済みのクライアントでは無視
                },
                MessageType::Disconnect { reason } => {
                    // 切断メッセージの処理
                    self.disconnect_client(client_id, reason).ok();
                },
                MessageType::Input => {
                    // 入力メッセージの処理
                    if let Some(input_data) = message.input_data {
                        // 入力の処理（実際のゲームロジック）
                        if self.config.debug_mode {
                            web_sys::console::log_1(&format!("クライアント {} からの入力を受信: {:?}", 
                                            client_id, input_data.movement).into());
                        }
                    }
                },
                MessageType::Ping { client_time } => {
                    // Pingへの応答
                    let pong = NetworkMessage::new(MessageType::Pong {
                        client_time,
                        server_time: self.server_time,
                    }).with_sequence(self.next_sequence_number());
                    
                    self.send_message(Some(client_id), pong).ok();
                },
                MessageType::TimeSync { client_time, .. } => {
                    // 時間同期メッセージへの応答
                    let time_sync = NetworkMessage::new(MessageType::TimeSync {
                        client_time,
                        server_time: self.server_time,
                    }).with_sequence(self.next_sequence_number());
                    
                    self.send_message(Some(client_id), time_sync).ok();
                },
                _ => {
                    // その他のメッセージ処理
                    if self.config.debug_mode {
                        web_sys::console::log_1(&format!("クライアント {} から未処理のメッセージを受信: {:?}", 
                                        client_id, message.message_type).into());
                    }
                }
            }
        }
    }

    /// 送信キューの処理
    fn process_pending_messages(&mut self) {
        // ここでは実際のWebSocketなどの送信処理を行うことになるが、
        // WebAssemblyコンテキストではモックとして実装
        
        while let Some((client_id, message)) = self.pending_messages.pop_front() {
            // ローカルシミュレーションモードの場合は、メッセージをコンソールに出力
            if self.config.debug_mode {
                let target = client_id.map_or("すべてのクライアント".to_string(), |id| format!("クライアント {}", id));
                web_sys::console::log_1(&format!("サーバーから {} へメッセージ送信: {:?}", target, message.message_type).into());
            }
            
            // 実際の送信処理はサーバーモードによって異なる実装になる
        }
    }

    /// クライアントの状態チェック
    fn check_clients(&mut self) {
        let now = Date::now();
        let timeout = self.config.connection_timeout_ms as f64;
        
        // タイムアウトしたクライアントのIDを収集
        let timed_out_clients: Vec<u32> = self.clients.iter()
            .filter(|(_, client)| now - client.last_message_time > timeout)
            .map(|(id, _)| *id)
            .collect();
        
        // タイムアウトしたクライアントを切断
        for client_id in timed_out_clients {
            self.disconnect_client(client_id, Some("接続タイムアウト".to_string())).ok();
        }
    }

    /// 次のシーケンス番号を取得
    fn next_sequence_number(&mut self) -> u32 {
        let seq = self.sequence_number;
        self.sequence_number = self.sequence_number.wrapping_add(1);
        seq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let config = NetworkConfig::default();
        let server = NetworkServer::new(config, ServerMode::LocalSimulation);
        
        assert_eq!(server.mode, ServerMode::LocalSimulation);
        assert!(!server.active);
        assert!(server.clients.is_empty());
    }

    #[test]
    fn test_client_connection() {
        let config = NetworkConfig::default();
        let mut server = NetworkServer::new(config, ServerMode::LocalSimulation);
        
        // サーバーを起動
        server.active = true;
        
        // クライアントを接続
        let player_data = PlayerData::default();
        let client_id = server.connect_client(player_data).unwrap();
        
        // クライアントが追加されたことを確認
        assert!(server.clients.contains_key(&client_id));
        assert_eq!(server.clients.len(), 1);
        
        // 接続応答メッセージがキューに追加されたことを確認
        assert_eq!(server.pending_messages.len(), 1);
    }
} 