use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Component トレイトを自動的に実装するマクロ
/// 
/// # 使用例
/// ```rust
/// #[derive(Component)]
/// pub struct Position {
///     x: f32,
///     y: f32,
/// }
/// ```
#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    // 入力を解析
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    // Component トレイトの実装を生成
    let expanded = quote! {
        impl Component for #name {
            fn name() -> &'static str {
                stringify!(#name)
            }
        }
    };
    
    // トークンストリームに変換して返す
    expanded.into()
}

/// Resource トレイトを自動的に実装するマクロ
/// 
/// # 使用例
/// ```rust
/// #[derive(Resource)]
/// pub struct GameState {
///     score: u32,
/// }
/// ```
#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    // 入力を解析
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    // Resource トレイトの実装を生成
    let expanded = quote! {
        impl Resource for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
    
    // トークンストリームに変換して返す
    expanded.into()
} 