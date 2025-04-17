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
// ecs_deriveクレートで定義されたマクロをここでリエクスポートして、外部からアクセス可能にする
pub use ecs_derive::{Component, Resource};

// モジュール宣言
// ECSアーキテクチャの各部分を別々のモジュールに分けて整理
pub mod entity;      // エンティティ（ゲーム内のオブジェクトID）を管理
pub mod component;   // コンポーネント（エンティティのデータ）を管理
pub mod system;      // システム（ゲームロジック）を管理
pub mod resource;    // リソース（グローバルデータ）を管理
pub mod macros;      // 便利なマクロを定義
pub mod query;       // エンティティとコンポーネントのクエリ機能を提供

// 主要な構造体をエクスポート
// 外部からこれらの型を直接インポートできるようにする
pub use entity::{Entity, EntityId, EntityManager};
pub use component::{Component, ComponentManager};
pub use system::{System, SystemPhase, SystemPriority, SystemProcessor};
pub use resource::{Resource, ResourceManager};
pub use query::{Query, Changed, With};

// プレリュードモジュール
// よく使われる型やマクロを一括でインポートするための便利な場所
#[macro_use]
pub mod prelude {
    pub use crate::impl_component;
    pub use ecs_derive::{Component, Resource};
}

/// ゲーム世界全体を表す中央のオブジェクト
/// エンティティ、コンポーネント、システム、リソースを統合的に管理します
pub struct World {
    /// システムプロセッサ
    /// すべてのECSロジックを処理するコアエンジン
    processor: SystemProcessor,
}

// Worldのクローン実装
// 必要な場合にWorldをコピーするための機能
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
    /// 
    /// # 戻り値
    /// 
    /// * 初期化された新しいWorld構造体
    /// 
    /// # 例
    /// 
    /// ```
    /// let mut world = World::new();
    /// ```
    pub fn new() -> Self {
        World {
            processor: SystemProcessor::new(),
        }
    }

    /// 新しいエンティティを作成
    /// 
    /// エンティティとは、ゲーム内のオブジェクト（キャラクター、カード、アイテムなど）を表す識別子です。
    /// エンティティ自体にはデータは含まれず、一意のIDとしての役割のみを持ちます。
    /// 
    /// # 戻り値
    /// 
    /// * 作成されたEntity
    /// 
    /// # 例
    /// 
    /// ```
    /// let player_entity = world.create_entity();
    /// ```
    pub fn create_entity(&mut self) -> Entity {
        self.processor.create_entity()
    }

    /// エンティティを削除
    /// 
    /// 指定したエンティティとそれに関連するすべてのコンポーネントを削除します。
    /// 
    /// # 引数
    /// 
    /// * `entity` - 削除するエンティティ
    /// 
    /// # 例
    /// 
    /// ```
    /// world.destroy_entity(player_entity);
    /// ```
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.processor.destroy_entity(entity);
    }

    /// エンティティにコンポーネントを追加
    /// 
    /// コンポーネントはエンティティのデータや振る舞いを定義します。
    /// 例えば、位置、体力、カードの値などのデータはコンポーネントとして表現されます。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - コンポーネントの型（Componentトレイトを実装している必要がある）
    /// 
    /// # 引数
    /// 
    /// * `entity` - コンポーネントを追加するエンティティ
    /// * `component` - 追加するコンポーネント
    /// 
    /// # 例
    /// 
    /// ```
    /// let position = Position { x: 0.0, y: 0.0 };
    /// world.add_component(player_entity, position);
    /// ```
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.processor.add_component(entity, component);
    }

    /// エンティティからコンポーネントを取得
    /// 
    /// 指定したエンティティの特定の型のコンポーネントを参照として取得します。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - 取得するコンポーネントの型
    /// 
    /// # 引数
    /// 
    /// * `entity` - コンポーネントを取得するエンティティ
    /// 
    /// # 戻り値
    /// 
    /// * `Option<&T>` - コンポーネントが存在する場合はSome(参照)、存在しない場合はNone
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some(position) = world.get_component::<Position>(player_entity) {
    ///     println!("プレイヤーの位置: x={}, y={}", position.x, position.y);
    /// }
    /// ```
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.processor.get_component(entity)
    }

    /// エンティティからコンポーネントを可変で取得
    /// 
    /// 指定したエンティティの特定の型のコンポーネントを可変参照として取得します。
    /// これによりコンポーネントのデータを変更できます。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - 取得するコンポーネントの型
    /// 
    /// # 引数
    /// 
    /// * `entity` - コンポーネントを取得するエンティティ
    /// 
    /// # 戻り値
    /// 
    /// * `Option<&mut T>` - コンポーネントが存在する場合はSome(可変参照)、存在しない場合はNone
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some(position) = world.get_component_mut::<Position>(player_entity) {
    ///     position.x += 1.0; // プレイヤーを右に移動
    /// }
    /// ```
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.processor.get_component_mut(entity)
    }

    /// エンティティからコンポーネントを削除
    /// 
    /// 指定したエンティティから特定の型のコンポーネントを削除します。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - 削除するコンポーネントの型
    /// 
    /// # 引数
    /// 
    /// * `entity` - コンポーネントを削除するエンティティ
    /// 
    /// # 戻り値
    /// 
    /// * `bool` - コンポーネントが存在し削除された場合はtrue、存在しなかった場合はfalse
    /// 
    /// # 例
    /// 
    /// ```
    /// let removed = world.remove_component::<Invisibility>(player_entity);
    /// if removed {
    ///     println!("プレイヤーの不可視状態が解除されました");
    /// }
    /// ```
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> bool {
        self.processor.remove_component::<T>(entity)
    }

    /// システムを登録
    /// 
    /// システムはゲームロジックを実行する単位で、特定のコンポーネントを持つエンティティに対して
    /// 処理を行います。例えば、移動システム、カード効果処理システムなどがあります。
    /// 
    /// # 型パラメータ
    /// 
    /// * `S` - 登録するシステムの型（Systemトレイトを実装している必要がある）
    /// 
    /// # 引数
    /// 
    /// * `system` - 登録するシステム
    /// 
    /// # 例
    /// 
    /// ```
    /// world.register_system(MovementSystem::new());
    /// world.register_system(CardEffectSystem::new());
    /// ```
    pub fn register_system<S: System>(&mut self, system: S) {
        self.processor.register_system(system);
    }

    /// 世界を更新（すべてのシステムを実行）
    /// 
    /// すべてのシステムを実行し、ゲーム状態を1フレーム分進めます。
    /// システムは登録された順番とフェーズ、優先度に従って実行されます。
    /// 
    /// # 引数
    /// 
    /// * `delta_time` - 前回の更新からの経過時間（秒）
    /// 
    /// # 例
    /// 
    /// ```
    /// world.update(0.016); // 約60FPSに相当
    /// ```
    pub fn update(&mut self, delta_time: f32) {
        let world = self as *mut World;
        unsafe {
            (*world).processor.update(&mut *world, delta_time);
        }
    }

    /// 特定のフェーズのみを更新
    /// 
    /// 指定したフェーズのシステムのみを実行します。
    /// これにより、更新とレンダリングなどを分離して実行できます。
    /// 
    /// # 引数
    /// 
    /// * `phase` - 実行するシステムのフェーズ
    /// * `delta_time` - 前回の更新からの経過時間（秒）
    /// 
    /// # 例
    /// 
    /// ```
    /// // 物理シミュレーションのみを更新
    /// world.update_phase(SystemPhase::Physics, 0.016);
    /// ```
    pub fn update_phase(&mut self, phase: SystemPhase, delta_time: f32) {
        let world = self as *mut World;
        unsafe {
            (*world).processor.update_phase(phase, &mut *world, delta_time);
        }
    }
    
    /// レンダリングフェーズのシステムを実行
    /// 
    /// 描画処理を行うシステムのみを実行します。
    /// update()とは別にレンダリングのみを行うために使用します。
    /// 
    /// # 例
    /// 
    /// ```
    /// // ゲームロジックを更新
    /// world.update(delta_time);
    /// // 描画処理のみを実行
    /// world.render();
    /// ```
    pub fn render(&mut self) {
        let world = self as *mut World;
        unsafe {
            (*world).processor.update_phase(SystemPhase::Render, &mut *world, 0.0);
        }
    }

    /// リソースを追加または更新
    /// 
    /// リソースはエンティティに紐付かないグローバルデータです。
    /// 例えば、ゲーム設定、スコア、共有状態などを管理するのに適しています。
    /// 
    /// この実装は非Wasm環境（例：デスクトップ）用です。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - リソースの型
    /// 
    /// # 引数
    /// 
    /// * `resource` - 追加または更新するリソース
    #[cfg(not(target_arch = "wasm32"))]
    pub fn insert_resource<T: 'static + Send + Sync + resource::Resource>(&mut self, resource: T) {
        self.processor.insert_resource(resource);
    }

    /// リソースを追加または更新（Wasm環境用）
    /// 
    /// WebAssembly環境用のリソース追加・更新関数です。
    /// Wasmでは一部の制約（Send + Sync）が不要になります。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - リソースの型
    /// 
    /// # 引数
    /// 
    /// * `resource` - 追加または更新するリソース
    #[cfg(target_arch = "wasm32")]
    pub fn insert_resource<T: 'static + resource::Resource>(&mut self, resource: T) {
        self.processor.insert_resource(resource);
    }

    /// リソースを取得
    /// 
    /// 指定した型のリソースを参照として取得します。
    /// この実装は非Wasm環境用です。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - 取得するリソースの型
    /// 
    /// # 戻り値
    /// 
    /// * `Option<&T>` - リソースが存在する場合はSome(参照)、存在しない場合はNone
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_resource<T: 'static + Send + Sync + resource::Resource>(&self) -> Option<&T> {
        self.processor.get_resource()
    }

    /// リソースを取得（Wasm環境用）
    /// 
    /// WebAssembly環境用のリソース取得関数です。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - 取得するリソースの型
    /// 
    /// # 戻り値
    /// 
    /// * `Option<&T>` - リソースが存在する場合はSome(参照)、存在しない場合はNone
    #[cfg(target_arch = "wasm32")]
    pub fn get_resource<T: 'static + resource::Resource>(&self) -> Option<&T> {
        self.processor.get_resource()
    }

    /// リソースを可変で取得
    /// 
    /// 指定した型のリソースを可変参照として取得します。
    /// この実装は非Wasm環境用です。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - 取得するリソースの型
    /// 
    /// # 戻り値
    /// 
    /// * `Option<&mut T>` - リソースが存在する場合はSome(可変参照)、存在しない場合はNone
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_resource_mut<T: 'static + Send + Sync + resource::Resource>(&mut self) -> Option<&mut T> {
        self.processor.get_resource_mut()
    }

    /// リソースを可変で取得（Wasm環境用）
    /// 
    /// WebAssembly環境用のリソース可変取得関数です。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - 取得するリソースの型
    /// 
    /// # 戻り値
    /// 
    /// * `Option<&mut T>` - リソースが存在する場合はSome(可変参照)、存在しない場合はNone
    #[cfg(target_arch = "wasm32")]
    pub fn get_resource_mut<T: 'static + resource::Resource>(&mut self) -> Option<&mut T> {
        self.processor.get_resource_mut()
    }

    /// リソースを削除
    /// 
    /// 指定した型のリソースを削除し、そのリソースを返します。
    /// この実装は非Wasm環境用です。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - 削除するリソースの型
    /// 
    /// # 戻り値
    /// 
    /// * `Option<T>` - リソースが存在する場合はSome(リソース)、存在しない場合はNone
    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove_resource<T: 'static + Send + Sync + resource::Resource>(&mut self) -> Option<T> {
        self.processor.remove_resource()
    }

    /// リソースを削除（Wasm環境用）
    /// 
    /// WebAssembly環境用のリソース削除関数です。
    /// 
    /// # 型パラメータ
    /// 
    /// * `T` - 削除するリソースの型
    /// 
    /// # 戻り値
    /// 
    /// * `Option<T>` - リソースが存在する場合はSome(リソース)、存在しない場合はNone
    #[cfg(target_arch = "wasm32")]
    pub fn remove_resource<T: 'static + resource::Resource>(&mut self) -> Option<T> {
        self.processor.remove_resource()
    }

    /// プロセッサへの参照を取得
    /// 
    /// 内部のシステムプロセッサへの参照を取得します。
    /// 高度なカスタマイズが必要な場合に使用します。
    pub fn processor(&self) -> &SystemProcessor {
        &self.processor
    }

    /// プロセッサへの可変参照を取得
    /// 
    /// 内部のシステムプロセッサへの可変参照を取得します。
    /// 高度なカスタマイズが必要な場合に使用します。
    pub fn processor_mut(&mut self) -> &mut SystemProcessor {
        &mut self.processor
    }

    /// 全エンティティを取得するイテレータを返す
    /// 
    /// 現在ワールドに存在するすべてのエンティティを反復処理するイテレータを返します。
    /// 
    /// # 戻り値
    /// 
    /// * エンティティのイテレータ
    /// 
    /// # 例
    /// 
    /// ```
    /// for entity in world.entities() {
    ///     println!("エンティティID: {:?}", entity.id());
    /// }
    /// ```
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.processor.entities()
    }

    /// 特定のコンポーネント型に対するクエリを作成
    ///
    /// クエリを使用することで、特定のコンポーネント型を持つエンティティのセットを取得できます。
    /// タプル型`(Entity, &T)`形式のクエリもサポートしています。
    /// これはエンティティとそのコンポーネントをまとめて取得する場合に便利です。
    ///
    /// # 型パラメータ
    /// 
    /// * `T` - クエリ対象のコンポーネント型
    /// 
    /// # 戻り値
    /// 
    /// * 指定されたコンポーネント型を持つエンティティに対するクエリ
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
    /// # 型パラメータ
    /// 
    /// * `T` - クエリ対象のコンポーネント型
    /// 
    /// # 戻り値
    /// 
    /// * エンティティとコンポーネントのペアに対するクエリ
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
        
        // すべてのエンティティをチェック
        for entity in self.entities() {
            // 対象のコンポーネントを持つエンティティのみ追加
            if self.get_component::<T>(entity).is_some() {
                query.add_entity(entity);
            }
        }
        
        query
    }

    /// 特定のコンポーネントを持つすべてのエンティティを取得
    /// 
    /// 指定したコンポーネント型を持つすべてのエンティティのベクターを返します。
    ///
    /// # 型パラメータ
    /// 
    /// * `T` - 対象のコンポーネント型
    /// 
    /// # 戻り値
    /// 
    /// * 条件を満たすエンティティのベクター
    ///
    /// # 例
    ///
    /// ```
    /// // CardComponentを持つすべてのエンティティを取得
    /// let card_entities = world.query_entities::<CardComponent>();
    /// println!("カードの数: {}", card_entities.len());
    /// ```
    pub fn query_entities<T>(&self) -> Vec<Entity> 
    where
        T: 'static + component::Component, // コンポーネント制約を追加
    {
        let mut result = Vec::new();
        
        // すべてのエンティティをチェック
        for entity in self.entities() {
            // 対象のコンポーネントを持つエンティティのみ追加
            if self.get_component::<T>(entity).is_some() {
                result.push(entity);
            }
        }
        
        result
    }
}

// 型IDから型名を取得するための内部トレイト
trait _TypeIdExt {
    fn type_name(&self) -> &'static str;
}

// TypeIdに対する実装
impl _TypeIdExt for std::any::TypeId {
    fn type_name(&self) -> &'static str {
        "TypeId"
    }
}

// ECSシステムの初期化
pub fn init() {
    // 将来的な初期化コードをここに記述
}