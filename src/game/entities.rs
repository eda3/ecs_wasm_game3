//! ゲームエンティティモジュール
//! 
//! ゲーム固有のエンティティとコンポーネントを実装します。

use wasm_bindgen::prelude::*;
use crate::ecs::{World, Entity};

/// プレイヤーエンティティ
/// 
/// プレイヤーキャラクターを表すエンティティです。
pub struct Player;

impl Player {
    /// 新しいプレイヤーエンティティを作成します。
    pub fn create(world: &mut World) -> Result<Entity, JsValue> {
        let entity = world.create_entity();

        // TODO: プレイヤーコンポーネントの追加

        Ok(entity)
    }
}

/// 敵エンティティ
/// 
/// 敵キャラクターを表すエンティティです。
pub struct Enemy;

impl Enemy {
    /// 新しい敵エンティティを作成します。
    pub fn create(world: &mut World) -> Result<Entity, JsValue> {
        let entity = world.create_entity();

        // TODO: 敵コンポーネントの追加

        Ok(entity)
    }
}

/// アイテムエンティティ
/// 
/// ゲーム内のアイテムを表すエンティティです。
pub struct Item;

impl Item {
    /// 新しいアイテムエンティティを作成します。
    pub fn create(world: &mut World) -> Result<Entity, JsValue> {
        let entity = world.create_entity();

        // TODO: アイテムコンポーネントの追加

        Ok(entity)
    }
}

/// エフェクトエンティティ
/// 
/// ゲーム内のエフェクトを表すエンティティです。
pub struct Effect;

impl Effect {
    /// 新しいエフェクトエンティティを作成します。
    pub fn create(world: &mut World) -> Result<Entity, JsValue> {
        let entity = world.create_entity();

        // TODO: エフェクトコンポーネントの追加

        Ok(entity)
    }
}

/// 背景エンティティ
/// 
/// ゲームの背景を表すエンティティです。
pub struct Background;

impl Background {
    /// 新しい背景エンティティを作成します。
    pub fn create(world: &mut World) -> Result<Entity, JsValue> {
        let entity = world.create_entity();

        // TODO: 背景コンポーネントの追加

        Ok(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_entity_creation() {
        let mut world = World::new();

        // プレイヤーエンティティの作成
        let player = Player::create(&mut world).unwrap();
        assert!(world.is_alive(player));

        // 敵エンティティの作成
        let enemy = Enemy::create(&mut world).unwrap();
        assert!(world.is_alive(enemy));

        // アイテムエンティティの作成
        let item = Item::create(&mut world).unwrap();
        assert!(world.is_alive(item));

        // エフェクトエンティティの作成
        let effect = Effect::create(&mut world).unwrap();
        assert!(world.is_alive(effect));

        // 背景エンティティの作成
        let background = Background::create(&mut world).unwrap();
        assert!(world.is_alive(background));
    }
} 