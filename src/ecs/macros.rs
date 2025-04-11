//! コンポーネントマクロモジュール
//! 
//! このモジュールは、コンポーネントの実装を簡単にするためのマクロを提供します。

/// コンポーネントマクロ
/// 
/// このマクロは、構造体に`Component`トレイトを自動的に実装します。
/// 
/// # 使用例
/// ```rust
/// #[derive(Component)]
/// pub struct Position {
///     x: f32,
///     y: f32,
/// }
/// ```
#[macro_export]
macro_rules! impl_component {
    ($type:ty, $name:expr) => {
        impl Component for $type {
            fn name() -> &'static str {
                $name
            }
        }
    };
}

/// コンポーネントマクロのテスト
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Component;

    struct TestComponent {
        value: i32,
    }

    impl_component!(TestComponent, "TestComponent");

    #[test]
    fn test_component_macro() {
        assert_eq!(TestComponent::name(), "TestComponent");
    }
} 