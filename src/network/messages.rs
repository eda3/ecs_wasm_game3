//! ネットワークメッセージのデータ型定義
//! 
//! このモジュールは、ネットワーク通信で使用される具体的なデータ構造を定義します。

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// プレイヤーに関連するデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    /// プレイヤー名
    pub name: String,
    /// プレイヤーのアバター識別子
    pub avatar: Option<String>,
    /// プレイヤーのチーム識別子
    pub team: Option<u32>,
    /// カスタムプレイヤー設定
    pub settings: Option<HashMap<String, serde_json::Value>>,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            name: "Player".to_string(),
            avatar: None,
            team: None,
            settings: None,
        }
    }
}

/// 入力データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputData {
    /// 移動入力 (x, y) - 通常は -1.0 から 1.0 の範囲
    pub movement: (f32, f32),
    /// アクション入力 (アクション名 => 実行状態)
    pub actions: HashMap<String, bool>,
    /// 照準座標 (存在する場合)
    pub aim: Option<(f32, f32)>,
    /// 入力のタイムスタンプ
    pub timestamp: f64,
}

impl Default for InputData {
    fn default() -> Self {
        Self {
            movement: (0.0, 0.0),
            actions: HashMap::new(),
            aim: None,
            timestamp: 0.0,
        }
    }
}

/// コンポーネントデータの型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ComponentData {
    /// 位置コンポーネント
    Position { 
        /// X座標
        x: f32, 
        /// Y座標
        y: f32,
        /// Z座標 (オプション)
        z: Option<f32>,
    },
    /// 速度コンポーネント
    Velocity { 
        /// X方向速度
        x: f32, 
        /// Y方向速度
        y: f32, 
        /// Z方向速度 (オプション)
        z: Option<f32>,
    },
    /// 回転コンポーネント
    Rotation { 
        /// 回転角度（ラジアン）
        angle: f32 
    },
    /// 体力コンポーネント
    Health { 
        /// 現在の体力
        current: u32, 
        /// 最大体力
        max: u32 
    },
    /// スプライトコンポーネント
    Sprite { 
        /// スプライト識別子
        id: String, 
        /// 表示フラグ
        visible: bool 
    },
    /// プレイヤー情報コンポーネント
    PlayerInfo { 
        /// プレイヤーID
        player_id: u32, 
        /// プレイヤー名
        name: String 
    },
    /// カスタムデータコンポーネント
    Custom { 
        /// カスタムデータ
        data: serde_json::Value 
    },
}

/// エンティティの完全なスナップショット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    /// エンティティID
    pub entity_id: u32,
    /// エンティティの各コンポーネント
    pub components: HashMap<String, ComponentData>,
    /// スナップショットのタイムスタンプ
    pub timestamp: f64,
    /// 所有者プレイヤーID（存在する場合）
    pub owner_id: Option<u32>,
}

impl EntitySnapshot {
    /// 新しいエンティティスナップショットを作成
    pub fn new(entity_id: u32, timestamp: f64) -> Self {
        Self {
            entity_id,
            components: HashMap::new(),
            timestamp,
            owner_id: None,
        }
    }

    /// コンポーネントを追加
    pub fn add_component(&mut self, name: &str, data: ComponentData) {
        self.components.insert(name.to_string(), data);
    }

    /// 所有者を設定
    pub fn set_owner(&mut self, player_id: u32) {
        self.owner_id = Some(player_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_data_serialization() {
        let mut actions = HashMap::new();
        actions.insert("jump".to_string(), true);
        actions.insert("fire".to_string(), false);
        
        let input = InputData {
            movement: (0.5, -0.3),
            actions,
            aim: Some((100.0, 200.0)),
            timestamp: 12345.0,
        };
        
        let json = serde_json::to_string(&input).unwrap();
        let deserialized: InputData = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.movement.0, 0.5);
        assert_eq!(deserialized.movement.1, -0.3);
        assert_eq!(deserialized.actions["jump"], true);
        assert_eq!(deserialized.actions["fire"], false);
        assert_eq!(deserialized.aim, Some((100.0, 200.0)));
    }

    #[test]
    fn test_component_data_serialization() {
        let position = ComponentData::Position { 
            x: 10.0, 
            y: 20.0, 
            z: Some(5.0) 
        };
        
        let json = serde_json::to_string(&position).unwrap();
        let deserialized: ComponentData = serde_json::from_str(&json).unwrap();
        
        if let ComponentData::Position { x, y, z } = deserialized {
            assert_eq!(x, 10.0);
            assert_eq!(y, 20.0);
            assert_eq!(z, Some(5.0));
        } else {
            panic!("Wrong component type after deserialization");
        }
    }
} 