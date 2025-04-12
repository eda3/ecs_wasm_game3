//! ネットワークプロトコルの定義
//! 
//! このモジュールは、クライアントとサーバー間で交換されるメッセージの形式と
//! シリアライズ/デシリアライズの処理を定義します。

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use js_sys::{Date, JSON};
use wasm_bindgen::prelude::*;
use super::messages::{InputData, PlayerData, ComponentData};
use crate::ecs::Entity;

/// メッセージ種別を表す列挙型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum MessageType {
    /// 接続
    Connect,
    /// 接続応答
    ConnectResponse { player_id: u32, success: bool, message: Option<String> },
    /// 切断
    Disconnect { reason: Option<String> },
    /// エンティティ作成
    EntityCreate { entity_id: u32 },
    /// エンティティ削除
    EntityDelete { entity_id: u32 },
    /// コンポーネント更新
    ComponentUpdate,
    /// 入力データ
    Input,
    /// 時間同期
    TimeSync { client_time: f64, server_time: f64 },
    /// Ping
    Ping { client_time: f64 },
    /// Pong
    Pong { client_time: f64, server_time: f64 },
    /// エラー
    Error { code: u32, message: String },
}

/// ネットワークメッセージの構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// メッセージ種別
    #[serde(flatten)]
    pub message_type: MessageType,
    /// シーケンス番号
    pub sequence: Option<u32>,
    /// タイムスタンプ
    pub timestamp: f64,
    /// エンティティID（関連する場合）
    pub entity_id: Option<u32>,
    /// プレイヤーID（関連する場合）
    pub player_id: Option<u32>,
    /// コンポーネントデータ（ComponentUpdate型の場合）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<HashMap<String, ComponentData>>,
    /// 入力データ（Input型の場合）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_data: Option<InputData>,
    /// プレイヤーデータ（プレイヤー関連のメッセージの場合）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_data: Option<PlayerData>,
}

impl NetworkMessage {
    /// 新しいメッセージを作成
    pub fn new(message_type: MessageType) -> Self {
        Self {
            message_type,
            sequence: None,
            timestamp: Date::now(),
            entity_id: None,
            player_id: None,
            components: None,
            input_data: None,
            player_data: None,
        }
    }

    /// エンティティIDを設定
    pub fn with_entity_id(mut self, entity_id: u32) -> Self {
        self.entity_id = Some(entity_id);
        self
    }

    /// プレイヤーIDを設定
    pub fn with_player_id(mut self, player_id: u32) -> Self {
        self.player_id = Some(player_id);
        self
    }

    /// シーケンス番号を設定
    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.sequence = Some(sequence);
        self
    }

    /// コンポーネントデータを設定
    pub fn with_components(mut self, components: HashMap<String, ComponentData>) -> Self {
        self.components = Some(components);
        self
    }

    /// 入力データを設定
    pub fn with_input(mut self, input: InputData) -> Self {
        self.input_data = Some(input);
        self
    }

    /// プレイヤーデータを設定
    pub fn with_player_data(mut self, player_data: PlayerData) -> Self {
        self.player_data = Some(player_data);
        self
    }

    /// JSON文字列からメッセージをデシリアライズ
    pub fn from_json(json: &str) -> Result<Self, JsValue> {
        let obj = JSON::parse(json).map_err(|e| {
            JsValue::from_str(&format!("JSON解析エラー: {:?}", e))
        })?;
        
        let message: Self = serde_wasm_bindgen::from_value(obj)?;
        Ok(message)
    }

    /// メッセージをJSON文字列にシリアライズ
    pub fn to_json(&self) -> Result<String, JsValue> {
        let js_value = serde_wasm_bindgen::to_value(self)?;
        let json = JSON::stringify(&js_value).map_err(|e| {
            JsValue::from_str(&format!("JSON文字列化エラー: {:?}", e))
        })?;
        
        Ok(String::from(js_value.as_string().unwrap_or_default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let message = NetworkMessage::new(MessageType::Connect)
            .with_player_id(123);
        
        assert_eq!(message.message_type, MessageType::Connect);
        assert_eq!(message.player_id, Some(123));
    }

    #[test]
    fn test_message_serialization() {
        let message = NetworkMessage::new(MessageType::Ping { client_time: 12345.0 })
            .with_sequence(42);
            
        let json = message.to_json().unwrap();
        let deserialized = NetworkMessage::from_json(&json).unwrap();
        
        assert_eq!(deserialized.sequence, Some(42));
        
        if let MessageType::Ping { client_time } = deserialized.message_type {
            assert_eq!(client_time, 12345.0);
        } else {
            panic!("Wrong message type after deserialization");
        }
    }
} 