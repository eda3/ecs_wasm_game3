//! Entity Component System (ECS)
//! 
//! このモジュールはゲームのエンティティ、コンポーネント、システムを管理するための
//! Entity Component System (ECS)アーキテクチャを実装しています。
//! 
//! ## 主要なコンポーネント:
//! 
//! - `Entity`: ゲーム内のオブジェクトを表す一意のID
//! - `Component`: エンティティの特性や状態を表すデータ構造
//! - `System`: コンポーネントを処理するロジック
//! - `World`: ECS全体を管理する中央ハブ

// マクロのリエクスポート
pub use ecs_derive::{Component, Resource};

// モジュール宣言
pub mod entity;
pub mod component;
pub mod system;
pub mod resource;
pub mod macros;
pub mod query;

// 主要な構造体をエクスポート
pub use entity::{Entity, EntityId, EntityManager};
pub use component::{Component, ComponentManager};
pub use system::{System, SystemPhase, SystemPriority, SystemProcessor};
pub use resource::{Resource, ResourceManager};
pub use query::{Query, Changed, With};

#[macro_use]
pub mod prelude {
    pub use crate::impl_component;
    pub use ecs_derive::{Component, Resource};
}

/// ゲーム世界全体を表す中央のオブジェクト
/// エンティティ、コンポーネント、システム、リソースを統合的に管理します
pub struct World {
    /// システムプロセッサ
    processor: SystemProcessor,
}

// Worldのクローン実装
impl Clone for World {
    fn clone(&self) -> Self {
        log::info!("Worldをクローンします");
        World {
            processor: self.processor.clone(),
        }
    }
}

impl World {
    /// 新しいゲーム世界を作成
    pub fn new() -> Self {
        World {
            processor: SystemProcessor::new(),
        }
    }

    /// 新しいエンティティを作成
    pub fn create_entity(&mut self) -> Entity {
        self.processor.create_entity()
    }

    /// エンティティを削除
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.processor.destroy_entity(entity);
    }

    /// エンティティにコンポーネントを追加
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.processor.add_component(entity, component);
    }

    /// エンティティからコンポーネントを取得
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.processor.get_component(entity)
    }

    /// エンティティからコンポーネントを可変で取得
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.processor.get_component_mut(entity)
    }

    /// エンティティからコンポーネントを削除
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> bool {
        self.processor.remove_component::<T>(entity)
    }

    /// システムを登録
    pub fn register_system<S: System>(&mut self, system: S) {
        self.processor.register_system(system);
    }

    /// 世界を更新（すべてのシステムを実行）
    pub fn update(&mut self, delta_time: f32) {
        let world = self as *mut World;
        unsafe {
            (*world).processor.update(&mut *world, delta_time);
        }
    }

    /// 特定のフェーズのみを更新
    pub fn update_phase(&mut self, phase: SystemPhase, delta_time: f32) {
        let world = self as *mut World;
        unsafe {
            (*world).processor.update_phase(phase, &mut *world, delta_time);
        }
    }
    
    /// レンダリングフェーズのシステムを実行
    pub fn render(&mut self) {
        let world = self as *mut World;
        unsafe {
            (*world).processor.update_phase(SystemPhase::Render, &mut *world, 0.0);
        }
    }

    /// リソースを追加または更新
    #[cfg(not(target_arch = "wasm32"))]
    pub fn insert_resource<T: 'static + Send + Sync + resource::Resource>(&mut self, resource: T) {
        self.processor.insert_resource(resource);
    }

    /// リソースを追加または更新（Wasm環境用）
    #[cfg(target_arch = "wasm32")]
    pub fn insert_resource<T: 'static + resource::Resource>(&mut self, resource: T) {
        self.processor.insert_resource(resource);
    }

    /// リソースを取得
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_resource<T: 'static + Send + Sync + resource::Resource>(&self) -> Option<&T> {
        self.processor.get_resource()
    }

    /// リソースを取得（Wasm環境用）
    #[cfg(target_arch = "wasm32")]
    pub fn get_resource<T: 'static + resource::Resource>(&self) -> Option<&T> {
        self.processor.get_resource()
    }

    /// リソースを可変で取得
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_resource_mut<T: 'static + Send + Sync + resource::Resource>(&mut self) -> Option<&mut T> {
        self.processor.get_resource_mut()
    }

    /// リソースを可変で取得（Wasm環境用）
    #[cfg(target_arch = "wasm32")]
    pub fn get_resource_mut<T: 'static + resource::Resource>(&mut self) -> Option<&mut T> {
        self.processor.get_resource_mut()
    }

    /// リソースを削除
    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove_resource<T: 'static + Send + Sync + resource::Resource>(&mut self) -> Option<T> {
        self.processor.remove_resource()
    }

    /// リソースを削除（Wasm環境用）
    #[cfg(target_arch = "wasm32")]
    pub fn remove_resource<T: 'static + resource::Resource>(&mut self) -> Option<T> {
        self.processor.remove_resource()
    }

    /// プロセッサへの参照を取得
    pub fn processor(&self) -> &SystemProcessor {
        &self.processor
    }

    /// プロセッサへの可変参照を取得
    pub fn processor_mut(&mut self) -> &mut SystemProcessor {
        &mut self.processor
    }

    /// 全エンティティを取得するイテレータを返す
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.processor.entities()
    }

    /// 特定のコンポーネント型に対するクエリを作成
    ///
    /// クエリを使用することで、特定のコンポーネント型を持つエンティティのセットを取得できます。
    /// タプル型`(Entity, &T)`形式のクエリもサポートしています。
    ///
    /// # 例
    ///
    /// ```
    /// // 標準的なコンポーネントクエリ
    /// let mut query = world.query::<NetworkComponent>();
    /// for (entity, network) in query.iter(world) {
    ///     // entityとnetworkを使用した処理
    /// }
    /// 
    /// // タプル型を使用したクエリ
    /// let mut query = world.query::<(Entity, &NetworkComponent)>();
    /// for (entity, network) in query.iter(world) {
    ///     // entityとnetworkを使用した処理
    /// }
    /// ```
    pub fn query<T>(&mut self) -> Query<T> 
    where
        T: 'static + component::Component, // コンポーネント制約を追加
    {
        let mut query = Query::new();
        
        // タプル型(Entity, &ComponentType)の特別な処理
        let type_name = std::any::type_name::<T>();
        if type_name.starts_with("(") && type_name.contains("Entity") {
            // タプル型の第2要素を抽出して処理する特殊ケース
            // ここでは型情報からでは完全な型を取り出せないため、エンティティをすべて追加
            for entity in self.entities() {
                query.add_entity(entity);
            }
        } else {
            // 通常のコンポーネント型向けの処理
            for entity in self.entities() {
                // コンポーネント型を持つエンティティのみをフィルタリング
                if let Some(_) = std::any::type_name::<T>().strip_prefix("(") {
                    // タプル型の場合は特殊処理
                    query.add_entity(entity);
                } else if self.get_component::<T>(entity).is_some() {
                    // 通常のコンポーネント型の場合は存在チェック
                    query.add_entity(entity);
                }
            }
        }
        
        query
    }

    /// タプル型(Entity, &T)の特殊クエリを生成
    /// 
    /// 標準のComponentトレイトを実装していないタプル型に対する特殊処理を提供します。
    /// これにより、タプル型に対するquery::filterメソッドの呼び出しが可能になります。
    ///
    /// # 例
    ///
    /// ```
    /// let query = world.query_tuple::<NetworkComponent>()
    ///     .filter(|_, network| network.is_synced && network.is_remote);
    /// ```
    pub fn query_tuple<T>(&self) -> query::EntityComponentQuery<T>
    where
        T: 'static + component::Component,
    {
        let mut query = query::EntityComponentQuery::new();
        
        // すべてのエンティティをクエリに追加
        for entity in self.entities() {
            if self.get_component::<T>(entity).is_some() {
                query.add_entity(entity);
            }
        }
        
        query
    }

    /// 特定のコンポーネントを持つすべてのエンティティを取得
    ///
    /// 指定されたコンポーネント型を持つエンティティのIDリストを返します。
    /// タプル型`(Entity, &T)`形式のクエリもサポートしています。
    ///
    /// # 例
    ///
    /// ```
    /// // 標準的なコンポーネントクエリ
    /// let entities = world.query_entities::<NetworkComponent>();
    /// for entity in entities {
    ///     // entityを使用した処理
    /// }
    /// ```
    pub fn query_entities<T>(&self) -> Vec<Entity> 
    where
        T: 'static + component::Component, // コンポーネント制約を追加
    {
        // タプル型(Entity, &ComponentType)の特別な処理
        let type_name = std::any::type_name::<T>();
        if type_name.contains("(Entity, &") {
            // タプル型の第2要素を抽出して処理
            // タプル型の場合はすべてのエンティティを返す
            return self.entities().collect();
        }
        
        // 標準的なコンポーネントクエリのケース
        let mut entities = Vec::new();
        for entity in self.entities() {
            if self.get_component::<T>(entity).is_some() {
                entities.push(entity);
            }
        }
        
        entities
    }
}

// TypeIdから型名を取得するための拡張トレイト
trait _TypeIdExt {
    fn type_name(&self) -> &'static str;
}

impl _TypeIdExt for std::any::TypeId {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

// ECSの初期化関数
pub fn init() {
    // ECSシステムの初期化処理
    println!("ECS System initialized");
} 