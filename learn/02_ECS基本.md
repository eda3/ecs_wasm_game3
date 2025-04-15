# Entity Component System（ECS）入門 🚀

こんにちは！今回はゲーム開発の強力なアーキテクチャである**ECS**（Entity Component System）について詳しく解説するよ！

## ECSとは何か？ 🤔

**ECS**は、ゲームエンジンやシミュレーションなどで使われる**データ指向**のアーキテクチャだよ。従来のオブジェクト指向と比べて、より柔軟でパフォーマンスに優れたシステムを作れるんだ！

ECSは主に3つの要素からなります：

1. **Entity（エンティティ）** - ゲーム内のオブジェクトを表す単なるID
2. **Component（コンポーネント）** - エンティティに紐づいたデータの集まり
3. **System（システム）** - コンポーネントを持つエンティティに対して処理を行うロジック

## エンティティ（Entity）🏷️

エンティティは単なる識別子（ID）です。中身は空っぽで、ただの番号だけ！

```rust
/// エンティティの一意な識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(u64);

/// エンティティを表す構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    id: EntityId,
    generation: u32,
}
```

エンティティ自体には振る舞いやデータは含まれません。ただの「名札」みたいなものです。

## コンポーネント（Component）📦

コンポーネントは**純粋なデータ**の塊です。メソッドや振る舞いを持たず、ただのデータ構造です！

```rust
/// コンポーネント型を識別するためのトレイト
pub trait Component: 'static + Send + Sync {
    /// コンポーネントの名前を取得
    fn name() -> &'static str where Self: Sized;
}

// 位置コンポーネントの例
#[derive(Component, Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// 速度コンポーネントの例
#[derive(Component, Debug, Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}
```

コンポーネントは単一の責任を持つように設計されます。例えば：
- `Position`（位置）
- `Velocity`（速度）
- `Health`（体力）
- `Sprite`（見た目）

これらを**組み合わせる**ことで、様々なゲームオブジェクトを表現できます！

## システム（System）⚙️

システムはゲームの**ロジック**を実装する部分です。特定のコンポーネントを持つエンティティに対して処理を行います：

```rust
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
```

システムの例：
- `MovementSystem` - 位置と速度を更新
- `RenderSystem` - 画面に描画
- `CollisionSystem` - 衝突判定
- `AISystem` - NPCの動きを制御

## リソース（Resource）🧩

リソースはエンティティに紐づかない**グローバルな状態**を表します：

```rust
pub trait Resource: 'static + Send + Sync + Any {
    /// リソースの型IDを取得
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

リソースの例：
- `Time` - ゲーム内の時間情報
- `InputState` - キーボードやマウスの入力状態
- `AudioPlayer` - 音楽や効果音の再生器
- `GameConfig` - ゲーム設定

## ワールド（World）🌍

ワールドはECSの**中心的な管理者**です。エンティティの作成・削除、コンポーネントの追加・取得、システムの実行などを管理します：

```rust
pub struct World {
    entities: EntityManager,
    components: ComponentManager,
    systems: SystemProcessor,
    resources: ResourceManager,
}
```

使用例：

```rust
// ワールドを作成
let mut world = World::new();

// エンティティを作成
let player = world.create_entity();

// コンポーネントを追加
world.add_component(player, Position { x: 0.0, y: 0.0 });
world.add_component(player, Velocity { x: 1.0, y: 0.0 });

// システムを登録
world.register_system(MovementSystem::new());

// リソースを追加
world.insert_resource(GameConfig::default());

// ゲームループ
loop {
    // 時間を更新
    let delta_time = 0.016; // 約60FPS
    
    // ワールドを更新（すべてのシステムを実行）
    world.update(delta_time);
}
```

## ECSのメリット👍

1. **データとロジックの分離** - コンポーネント（データ）とシステム（ロジック）を明確に分けられる
2. **柔軟性** - 新しい種類のゲームオブジェクトを簡単に作成できる
3. **パフォーマンス** - データをキャッシュフレンドリーに配置し、効率的に処理できる
4. **並列処理** - システムを独立して並列実行できる可能性がある
5. **拡張性** - 新機能を既存のコードを変更せずに追加できる

## クエリシステム🔍

特定のコンポーネントを持つエンティティを効率的に検索するためのクエリシステムもあります：

```rust
// Position AND Velocityを持つエンティティをすべて取得
for (entity, (position, velocity)) in world.query::<(Position, Velocity)>() {
    // 位置を更新
    position.x += velocity.x * delta_time;
    position.y += velocity.y * delta_time;
}
```

これにより、必要なコンポーネントを持つエンティティだけを効率的に処理できます！

## まとめ📝

ECSは現代のゲーム開発で広く使われている強力なアーキテクチャです。データ指向の考え方を取り入れることで、柔軟でパフォーマンスの高いゲームを作ることができます。

次のステップとして、実際にコンポーネントやシステムを作成してみましょう！

[次へ：コンポーネントの作り方](03_コンポーネント.md) 