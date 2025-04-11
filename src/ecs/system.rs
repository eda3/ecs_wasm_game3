use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use crate::ecs::{Component, ComponentManager, Entity, Resource, World};
use wasm_bindgen::JsValue;

/// システムの実行フェーズ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemPhase {
    /// 初期化フェーズ
    Init,
    /// 入力処理フェーズ
    Input,
    /// 更新フェーズ
    Update,
    /// レンダリングフェーズ
    Render,
    /// 終了フェーズ
    Shutdown,
}

/// システムの優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemPriority(pub u32);

impl SystemPriority {
    pub fn new(priority: u32) -> Self {
        Self(priority)
    }
}

impl Default for SystemPriority {
    fn default() -> Self {
        Self(0)
    }
}

/// システムのトレイト
pub trait System: 'static + Send + Sync {
    /// システムの名前を取得
    fn name(&self) -> &'static str;
    
    /// システムの実行フェーズを取得
    fn phase(&self) -> SystemPhase;
    
    /// システムの優先度を取得
    fn priority(&self) -> SystemPriority;
    
    /// システムを実行
    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue>;
}

/// システムプロセッサー
pub struct SystemProcessor {
    component_manager: ComponentManager,
}

impl SystemProcessor {
    /// 新しいシステムプロセッサーを作成
    pub fn new() -> Self {
        Self {
            component_manager: ComponentManager::new(),
        }
    }

    /// エンティティを作成
    pub fn create_entity(&mut self) -> Entity {
        Entity::new()
    }

    /// エンティティを削除
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.component_manager.remove_all_components(entity);
    }

    /// コンポーネントを追加
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.component_manager.add_component(entity, component);
    }

    /// コンポーネントを取得
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.component_manager.get_component(entity)
    }

    /// コンポーネントを可変で取得
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.component_manager.get_component_mut(entity)
    }

    /// コンポーネントを削除
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> bool {
        self.component_manager.remove_component::<T>(entity)
    }
}

/// システムプロセッサ
/// システムの登録と実行を管理する
pub struct SystemProcessor {
    /// フェーズごとのシステムリスト
    systems: HashMap<SystemPhase, Vec<Box<dyn System>>>,
    /// リソースのマップ
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl SystemProcessor {
    /// 新しいシステムプロセッサを作成
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    /// システムを登録
    pub fn register_system<S: System>(&mut self, system: S) {
        let phase = system.phase();
        let systems = self.systems.entry(phase).or_insert_with(Vec::new);
        
        // 優先度に基づいてシステムを挿入
        let priority = system.priority();
        let index = systems.binary_search_by_key(&priority, |s| s.priority())
            .unwrap_or_else(|e| e);
        
        systems.insert(index, Box::new(system));
    }

    /// 特定のフェーズのシステムを実行
    pub fn update_phase(&mut self, phase: SystemPhase, world: &mut World, delta_time: f32) {
        if let Some(systems) = self.systems.get_mut(&phase) {
            for system in systems.iter_mut() {
                if let Err(e) = system.run(world, self, delta_time) {
                    log::error!("システムの実行中にエラーが発生: {:?}", e);
                }
            }
        }
    }

    /// すべてのシステムを実行
    pub fn update(&mut self, world: &mut World, delta_time: f32) {
        // 各フェーズを順番に実行
        for phase in [
            SystemPhase::Init,
            SystemPhase::Input,
            SystemPhase::Update,
            SystemPhase::Render,
            SystemPhase::Shutdown,
        ] {
            self.update_phase(phase, world, delta_time);
        }
    }

    /// リソースを追加または更新
    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }

    /// リソースを取得
    pub fn get_resource<T: 'static + Send + Sync>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.resources.get(&type_id).and_then(|r| r.downcast_ref())
    }

    /// リソースを可変で取得
    pub fn get_resource_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.resources.get_mut(&type_id).and_then(|r| r.downcast_mut())
    }

    /// リソースを削除
    pub fn remove_resource<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.resources.remove(&type_id).and_then(|r| r.downcast().ok().map(|b| *b))
    }
}

/// システムビルダー
/// システムの構築を支援する
pub struct SystemBuilder<S: System> {
    system: S,
    phase: SystemPhase,
    priority: SystemPriority,
}

impl<S: System> SystemBuilder<S> {
    /// 新しいシステムビルダーを作成
    pub fn new(system: S) -> Self {
        Self {
            system,
            phase: SystemPhase::Update,
            priority: SystemPriority::default(),
        }
    }

    /// システムのフェーズを設定
    pub fn with_phase(mut self, phase: SystemPhase) -> Self {
        self.phase = phase;
        self
    }

    /// システムの優先度を設定
    pub fn with_priority(mut self, priority: SystemPriority) -> Self {
        self.priority = priority;
        self
    }

    /// システムを構築
    pub fn build(self) -> S {
        self.system
    }
} 