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
    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, resource: T) {
        self.processor.insert_resource(resource);
    }

    /// リソースを取得
    pub fn get_resource<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.processor.get_resource()
    }

    /// リソースを可変で取得
    pub fn get_resource_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.processor.get_resource_mut()
    }

    /// リソースを削除
    pub fn remove_resource<T: 'static + Send + Sync>(&mut self) -> Option<T> {
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
}

// ECSの初期化関数
pub fn init() {
    // ECSシステムの初期化処理
    println!("ECS System initialized");
} 