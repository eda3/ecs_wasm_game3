//! ネットワークシステムの実装
//! 
//! このモジュールは、クライアント/サーバー間の通信を担当します。
//! プレイヤーの同期、ゲーム状態の共有、予測と補正を行います。

use crate::ecs::{Component, Entity, World};
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

/// ネットワーク接続の状態
#[derive(Debug, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
}

/// ネットワークマネージャー
#[derive(Debug)]
pub struct NetworkManager {
    pub connection_state: ConnectionState,
    pub player_id: Option<u32>,
    pub players: HashMap<u32, NetworkPlayer>,
    pub latency: f32,
    pub packet_loss: f32,
}

/// ネットワークプレイヤー
#[derive(Debug)]
pub struct NetworkPlayer {
    pub id: u32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub last_update: f64,
}

impl NetworkManager {
    /// 新しいネットワークマネージャーを作成
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            player_id: None,
            players: HashMap::new(),
            latency: 0.0,
            packet_loss: 0.0,
        }
    }

    /// サーバーに接続
    pub fn connect(&mut self, server_url: &str) -> Result<(), String> {
        self.connection_state = ConnectionState::Connecting;
        // TODO: WebSocket接続の実装
        Ok(())
    }

    /// サーバーから切断
    pub fn disconnect(&mut self) {
        self.connection_state = ConnectionState::Disconnecting;
        // TODO: 切断処理の実装
    }

    /// プレイヤーの状態を更新
    pub fn update_player(&mut self, player_id: u32, position: [f32; 2], velocity: [f32; 2]) {
        if let Some(player) = self.players.get_mut(&player_id) {
            player.position = position;
            player.velocity = velocity;
            player.last_update = js_sys::Date::now();
        } else {
            self.players.insert(player_id, NetworkPlayer {
                id: player_id,
                position,
                velocity,
                last_update: js_sys::Date::now(),
            });
        }
    }
}

/// ネットワークコンポーネント
#[derive(Debug, Component)]
pub struct NetworkComponent {
    pub is_synced: bool,
    pub last_sync_time: f64,
    pub interpolation_factor: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_manager_creation() {
        let manager = NetworkManager::new();
        assert_eq!(manager.connection_state, ConnectionState::Disconnected);
        assert!(manager.player_id.is_none());
        assert!(manager.players.is_empty());
    }

    #[test]
    fn test_player_update() {
        let mut manager = NetworkManager::new();
        let position = [100.0, 200.0];
        let velocity = [10.0, 20.0];
        
        manager.update_player(1, position, velocity);
        let player = manager.players.get(&1).unwrap();
        
        assert_eq!(player.position, position);
        assert_eq!(player.velocity, velocity);
    }
} 