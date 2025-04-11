use std::collections::HashSet;
use std::fmt;
use rand;

/// エンティティの一意な識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(u64);

impl EntityId {
    /// 新しいエンティティIDを生成
    pub fn new() -> Self {
        Self(rand::random())
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

/// エンティティを表す構造体
/// エンティティはIDとバージョンからなり、再利用されたIDを区別できる
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    id: EntityId,
    generation: u32,
}

impl Entity {
    /// 新しいエンティティを作成
    pub fn new() -> Self {
        Self {
            id: EntityId::new(),
            generation: 0,
        }
    }

    /// エンティティのIDを取得
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// エンティティの世代を取得
    pub fn generation(&self) -> u32 {
        self.generation
    }
}

/// エンティティの生成と削除を管理する構造体
pub struct EntityManager {
    active_entities: HashSet<Entity>,
    next_generation: u32,
}

impl EntityManager {
    /// 新しいエンティティマネージャーを作成
    pub fn new() -> Self {
        Self {
            active_entities: HashSet::new(),
            next_generation: 0,
        }
    }

    /// 新しいエンティティを作成
    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity::new();
        self.active_entities.insert(entity);
        entity
    }

    /// エンティティを削除
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.active_entities.remove(&entity);
    }

    /// エンティティが有効かどうかを確認
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.active_entities.contains(&entity)
    }

    /// アクティブなエンティティの数を取得
    pub fn entity_count(&self) -> usize {
        self.active_entities.len()
    }
}

/// エンティティを便利に構築するためのビルダー
pub struct EntityBuilder {
    entity: Entity,
}

impl EntityBuilder {
    /// 新しいエンティティビルダーを作成
    pub fn new() -> Self {
        Self {
            entity: Entity::new(),
        }
    }

    /// エンティティを構築
    pub fn build(self) -> Entity {
        self.entity
    }
} 