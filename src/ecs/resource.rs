use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;

#[cfg(not(target_arch = "wasm32"))]
/// リソースの基本トレイト（非Wasm環境用）
/// 
/// グローバルに共有される状態を管理するための基本インターフェースを提供します。
/// すべてのリソースはこのトレイトを実装する必要があります。
pub trait Resource: 'static + Send + Sync + Any {
    /// リソースの型IDを取得
    /// 
    /// # 戻り値
    /// 
    /// リソースの型ID
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[cfg(target_arch = "wasm32")]
/// リソースの基本トレイト（Wasm環境用）
/// 
/// WebAssembly環境では通常シングルスレッドで動作するため、
/// SendとSyncトレイトの制約を取り除いています。
pub trait Resource: 'static + Any {
    /// リソースの型IDを取得
    /// 
    /// # 戻り値
    /// 
    /// リソースの型ID
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// リソースが見つからない場合のエラー
#[derive(Debug, Clone)]
pub struct ResourceNotFoundError {
    type_id: TypeId,
}

impl fmt::Display for ResourceNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resource of type {:?} not found", self.type_id)
    }
}

/// リソースマネージャ
/// 
/// ゲーム内のすべてのリソースを管理し、型安全なアクセスを提供します。
/// システム間で共有される状態を効率的に管理します。
pub struct ResourceManager {
    /// リソースの型IDと実体のマップ
    resources: HashMap<TypeId, Box<dyn Resource>>,
}

impl ResourceManager {
    /// 新しいリソースマネージャを作成
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// リソースを追加
    #[cfg(not(target_arch = "wasm32"))]
    pub fn insert<T: 'static + Send + Sync + Resource>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }

    /// リソースを追加（Wasm環境用）
    #[cfg(target_arch = "wasm32")]
    pub fn insert<T: 'static + Resource>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }

    /// リソースを取得
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get<T: 'static + Send + Sync + Resource>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.resources.get(&type_id).and_then(|r| r.as_any().downcast_ref::<T>())
    }

    /// リソースを取得（Wasm環境用）
    #[cfg(target_arch = "wasm32")]
    pub fn get<T: 'static + Resource>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.resources.get(&type_id).and_then(|r| r.as_any().downcast_ref::<T>())
    }

    /// リソースを可変で取得
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_mut<T: 'static + Send + Sync + Resource>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.resources.get_mut(&type_id).and_then(|r| r.as_any_mut().downcast_mut::<T>())
    }

    /// リソースを可変で取得（Wasm環境用）
    #[cfg(target_arch = "wasm32")]
    pub fn get_mut<T: 'static + Resource>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.resources.get_mut(&type_id).and_then(|r| r.as_any_mut().downcast_mut::<T>())
    }

    /// リソースを削除
    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove<T: 'static + Send + Sync + Resource>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.resources.remove(&type_id).map(|boxed_resource| {
            // Box<dyn Resource>からBox<T>に変換
            let raw_ptr = Box::into_raw(boxed_resource);
            unsafe {
                // rawポインタをBox<T>にキャストして所有権を取り戻す
                *Box::from_raw(raw_ptr as *mut T)
            }
        })
    }

    /// リソースを削除（Wasm環境用）
    #[cfg(target_arch = "wasm32")]
    pub fn remove<T: 'static + Resource>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.resources.remove(&type_id).map(|boxed_resource| {
            // Box<dyn Resource>からBox<T>に変換
            let raw_ptr = Box::into_raw(boxed_resource);
            unsafe {
                // rawポインタをBox<T>にキャストして所有権を取り戻す
                *Box::from_raw(raw_ptr as *mut T)
            }
        })
    }

    /// リソースが存在するか確認
    #[cfg(not(target_arch = "wasm32"))]
    pub fn contains<T: 'static + Send + Sync + Resource>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.resources.contains_key(&type_id)
    }

    /// リソースが存在するか確認（Wasm環境用）
    #[cfg(target_arch = "wasm32")]
    pub fn contains<T: 'static + Resource>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.resources.contains_key(&type_id)
    }

    /// すべてのリソースをクリア
    pub fn clear(&mut self) {
        self.resources.clear();
    }

    /// リソースの数を取得
    /// 
    /// # 戻り値
    /// 
    /// 現在管理されているリソースの数
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// リソースが空かどうかを確認
    /// 
    /// # 戻り値
    /// 
    /// リソースが空の場合はtrue、そうでない場合はfalse
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestResource {
        value: i32,
    }

    impl Resource for TestResource {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_resource_manager() {
        let mut manager = ResourceManager::new();

        // リソースの追加と取得
        let resource = TestResource { value: 42 };
        manager.insert(resource);

        let retrieved = manager.get::<TestResource>().unwrap();
        assert_eq!(retrieved.value, 42);

        // 可変リソースの取得
        let mut_retrieved = manager.get_mut::<TestResource>().unwrap();
        mut_retrieved.value = 84;

        // 変更の確認
        let retrieved = manager.get::<TestResource>().unwrap();
        assert_eq!(retrieved.value, 84);

        // リソースの削除
        let removed = manager.remove::<TestResource>().unwrap();
        assert_eq!(removed.value, 84);
        assert!(!manager.contains::<TestResource>());
    }
} 