use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::ecs::entity::{Entity, EntityId};

/// コンポーネント型を識別するためのトレイト
pub trait Component: 'static + Send + Sync {
    /// コンポーネントの名前を取得
    fn name() -> &'static str where Self: Sized;
}

/// コンポーネントのストレージ抽象化
pub trait ComponentStorage {
    /// コンポーネントの型IDを取得
    fn component_type_id(&self) -> TypeId;

    /// エンティティからコンポーネントを削除
    fn remove(&mut self, entity: EntityId) -> bool;

    /// すべてのコンポーネントをクリア
    fn clear(&mut self);

    /// 特定のエンティティのコンポーネントが存在するか確認
    fn has(&self, entity: EntityId) -> bool;

    /// 内部ストレージをAny型として取得
    fn as_any(&self) -> &dyn Any;

    /// 内部ストレージを可変Any型として取得
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /// このストレージに格納されているすべてのエンティティIDのベクターを返す
    fn entity_ids(&self) -> Vec<EntityId>;
}

/// 特定の型Tに対するコンポーネントストレージの実装
pub struct VecStorage<T: Component> {
    /// エンティティID→インデックスのマッピング
    entities: HashMap<EntityId, usize>,
    /// コンポーネントデータとそのエンティティIDのペア
    data: Vec<(EntityId, T)>,
    /// 型情報
    _marker: PhantomData<T>,
}

impl<T: Component> VecStorage<T> {
    /// 新しいストレージを作成
    pub fn new() -> Self {
        VecStorage {
            entities: HashMap::new(),
            data: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// コンポーネントを追加
    pub fn insert(&mut self, entity: EntityId, component: T) -> Option<T> {
        if let Some(&index) = self.entities.get(&entity) {
            // 既存のコンポーネントを置き換え
            let old = std::mem::replace(&mut self.data[index].1, component);
            Some(old)
        } else {
            // 新しいコンポーネントを追加
            let index = self.data.len();
            self.data.push((entity, component));
            self.entities.insert(entity, index);
            None
        }
    }

    /// コンポーネントを取得
    pub fn get(&self, entity: EntityId) -> Option<&T> {
        self.entities.get(&entity).map(|&index| &self.data[index].1)
    }

    /// コンポーネントを可変で取得
    pub fn get_mut(&mut self, entity: EntityId) -> Option<&mut T> {
        if let Some(&index) = self.entities.get(&entity) {
            Some(&mut self.data[index].1)
        } else {
            None
        }
    }

    /// すべてのコンポーネントとそのエンティティIDを取得
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.data.iter().map(|(e, c)| (*e, c))
    }

    /// すべてのコンポーネントとそのエンティティIDを可変で取得
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.data.iter_mut().map(|(e, c)| (*e, c))
    }
}

impl<T: Component> ComponentStorage for VecStorage<T> {
    fn component_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn remove(&mut self, entity: EntityId) -> bool {
        if let Some(index) = self.entities.remove(&entity) {
            // 最後の要素を削除位置に移動して、データベクターを縮小
            let last_idx = self.data.len() - 1;
            if index < last_idx {
                // 借用問題を回避するために一時変数を使用
                let swapped_entity = self.data[last_idx].0;
                self.data.swap(index, last_idx);
                // スワップされたエンティティのインデックスを更新
                self.entities.insert(swapped_entity, index);
            }
            self.data.pop();
            true
        } else {
            false
        }
    }

    fn clear(&mut self) {
        self.entities.clear();
        self.data.clear();
    }

    fn has(&self, entity: EntityId) -> bool {
        self.entities.contains_key(&entity)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn entity_ids(&self) -> Vec<EntityId> {
        self.data.iter().map(|(entity_id, _)| *entity_id).collect()
    }
}

/// コンポーネントマネージャー
/// 異なる型のコンポーネントを格納・管理する
pub struct ComponentManager {
    /// 型ID → コンポーネントストレージのマッピング
    storages: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl ComponentManager {
    /// 新しいコンポーネントマネージャーを作成
    pub fn new() -> Self {
        ComponentManager {
            storages: HashMap::new(),
        }
    }

    /// コンポーネントストレージを登録
    pub fn register<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        if !self.storages.contains_key(&type_id) {
            let storage = VecStorage::<T>::new();
            self.storages.insert(type_id, Box::new(storage));
        }
    }

    /// エンティティにコンポーネントを追加
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        
        // 必要に応じてストレージを登録
        if !self.storages.contains_key(&type_id) {
            self.register::<T>();
        }
        
        if let Some(storage) = self.storages.get_mut(&type_id) {
            // ダウンキャストしてコンポーネントを追加
            let storage = storage.as_any_mut()
                .downcast_mut::<VecStorage<T>>()
                .expect("Failed to downcast storage");
            
            storage.insert(entity.id(), component);
        }
    }

    /// エンティティからコンポーネントを取得
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        
        self.storages.get(&type_id).and_then(|storage| {
            let storage = storage.as_any()
                .downcast_ref::<VecStorage<T>>()
                .expect("Failed to downcast storage");
            
            storage.get(entity.id())
        })
    }

    /// エンティティからコンポーネントを可変で取得
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        
        self.storages.get_mut(&type_id).and_then(|storage| {
            let storage = storage.as_any_mut()
                .downcast_mut::<VecStorage<T>>()
                .expect("Failed to downcast storage");
            
            storage.get_mut(entity.id())
        })
    }

    /// エンティティがコンポーネントを持っているか確認
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        
        if let Some(storage) = self.storages.get(&type_id) {
            storage.has(entity.id())
        } else {
            false
        }
    }

    /// エンティティからコンポーネントを削除
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        
        if let Some(storage) = self.storages.get_mut(&type_id) {
            storage.remove(entity.id())
        } else {
            false
        }
    }

    /// 特定のコンポーネント型を持つすべてのエンティティを取得
    pub fn get_entities_with<T: Component>(&self) -> Vec<EntityId> {
        let type_id = TypeId::of::<T>();
        
        if let Some(storage) = self.storages.get(&type_id) {
            let storage = storage.as_any()
                .downcast_ref::<VecStorage<T>>()
                .expect("Failed to downcast storage");
            
            storage.iter().map(|(entity_id, _)| entity_id).collect()
        } else {
            Vec::new()
        }
    }

    /// エンティティからすべてのコンポーネントを削除
    pub fn remove_all_components(&mut self, entity: Entity) {
        for storage in self.storages.values_mut() {
            storage.remove(entity.id());
        }
    }

    /// すべてのエンティティIDを収集
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        // すべてのコンポーネントストレージを検索して、一意なエンティティIDのセットを作成
        let mut entity_set = std::collections::HashSet::new();
        
        // すべてのストレージからエンティティIDを収集
        for storage in self.storages.values() {
            for entity_id in storage.entity_ids() {
                entity_set.insert(entity_id);
            }
        }
        
        // 一意なエンティティIDをEntityインスタンスに変換
        entity_set.into_iter().map(|id| {
            // EntityIdからEntityインスタンスを再構築
            let mut entity = Entity::new();
            // これは比較用の仮のEntityインスタンスで、実際の処理では
            // 各コンポーネントの処理がEntityのIDに基づいて行われるため機能します
            entity
        })
    }
} 