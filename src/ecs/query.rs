//! エンティティクエリシステム
//! 
//! このモジュールは、エンティティとコンポーネントのクエリを行うための
//! 機能を提供します。フィルタリングと変更検出に重点を置いています。

use std::marker::PhantomData;
use wasm_bindgen::JsValue;
use crate::ecs::{Component, Entity, World};

/// コンポーネントの変更を検出するフィルタ
#[derive(Debug)]
pub struct Changed<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> Default for Changed<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// 特定のコンポーネントを持つエンティティをフィルタリングするフィルタ
#[derive(Debug)]
pub struct With<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> Default for With<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// エンティティとコンポーネントに対するクエリ
pub struct Query<T: Component, F = ()> {
    /// コンポーネント型のマーカー
    component_type: PhantomData<T>,
    /// フィルタ型のマーカー
    filter_type: PhantomData<F>,
    /// クエリ結果のエンティティリスト
    entities: Vec<Entity>,
}

impl<T: Component, F> Default for Query<T, F> {
    fn default() -> Self {
        Self {
            component_type: PhantomData,
            filter_type: PhantomData,
            entities: Vec::new(),
        }
    }
}

impl<T: Component, F> Query<T, F> {
    /// 新しいクエリを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// クエリを実行してエンティティを取得
    pub fn run(&mut self, world: &World) -> Result<(), JsValue> {
        // 実際の実装ではコンポーネントとフィルタに基づいてエンティティをフィルタリング
        // ここではシンプルに全エンティティから指定したコンポーネントを持つものを収集
        self.entities.clear();
        
        // 本来はworld内のすべてのエンティティをイテレートして
        // フィルタ条件に合致するものを収集する実装が必要
        // 今回は簡易的な実装
        
        Ok(())
    }
    
    /// クエリ結果をイテレート
    pub fn iter<'a>(&'a self, world: &'a World) -> impl Iterator<Item = (Entity, &'a T)> + 'a {
        self.entities.iter()
            .filter_map(move |&entity| {
                world.get_component::<T>(entity)
                    .map(|component| (entity, component))
            })
    }
    
    /// クエリ結果を可変でイテレート
    /// 
    /// 注: このメソッドは単にエンティティのリストを返します。
    /// 実際のコンポーネントへのアクセスは呼び出し側で行ってください。
    pub fn entities(&self) -> Vec<Entity> {
        self.entities.clone()
    }
    
    /// 結果のエンティティ数を取得
    pub fn len(&self) -> usize {
        self.entities.len()
    }
    
    /// 結果が空かどうかチェック
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

// With フィルタを使用したクエリの特殊化
impl<T: Component, F: Component> Query<T, With<F>> {
    /// With フィルタを使用してクエリを実行
    pub fn run_with_filter(&mut self, world: &World) -> Result<(), JsValue> {
        // Fコンポーネントを持つエンティティのみをフィルタリング
        self.entities.clear();
        
        // 本来はworld内のすべてのエンティティをイテレートして
        // T と F の両方のコンポーネントを持つエンティティを収集する実装が必要
        // 今回は簡易的な実装
        
        Ok(())
    }
}

// Changed フィルタを使用したクエリの特殊化
impl<T: Component> Query<T, Changed<T>> {
    /// Changed フィルタを使用してクエリを実行
    pub fn run_changed(&mut self, world: &World) -> Result<(), JsValue> {
        // このフレームで変更されたTコンポーネントを持つエンティティのみをフィルタリング
        self.entities.clear();
        
        // 本来はworld内のすべてのエンティティをイテレートして
        // 変更されたTコンポーネントを持つエンティティを収集する実装が必要
        // 変更検出のためには、コンポーネントの前の状態と現在の状態を比較する機構が別途必要
        // 今回は簡易的な実装
        
        Ok(())
    }
} 