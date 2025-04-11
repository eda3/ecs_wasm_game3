//! レンダリングレイヤーモジュール
//! 
//! ゲームのレンダリングレイヤーを管理します。
//! レイヤーごとの描画順序や可視性を制御します。

/// レンダリングレイヤー構造体
/// 
/// ゲームのレンダリングレイヤーを表します。
pub struct RenderLayer {
    /// レイヤー名
    pub name: String,
    /// Zインデックス（描画順序）
    pub z_index: i32,
    /// 可視性
    pub visible: bool,
    /// エンティティIDのリスト
    pub entities: Vec<u32>,
}

impl RenderLayer {
    /// 新しいレンダリングレイヤーを作成
    pub fn new(name: String, z_index: i32) -> Self {
        Self {
            name,
            z_index,
            visible: true,
            entities: Vec::new(),
        }
    }

    /// レイヤーの可視性を設定
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// エンティティを追加
    pub fn add_entity(&mut self, entity_id: u32) {
        if !self.entities.contains(&entity_id) {
            self.entities.push(entity_id);
        }
    }

    /// エンティティを削除
    pub fn remove_entity(&mut self, entity_id: u32) {
        if let Some(index) = self.entities.iter().position(|&id| id == entity_id) {
            self.entities.remove(index);
        }
    }

    /// エンティティが存在するか確認
    pub fn contains_entity(&self, entity_id: u32) -> bool {
        self.entities.contains(&entity_id)
    }

    /// エンティティのリストを取得
    pub fn get_entities(&self) -> &[u32] {
        &self.entities
    }

    /// エンティティのリストをクリア
    pub fn clear_entities(&mut self) {
        self.entities.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = RenderLayer::new("background".to_string(), 0);
        assert_eq!(layer.name, "background");
        assert_eq!(layer.z_index, 0);
        assert!(layer.visible);
        assert!(layer.entities.is_empty());
    }

    #[test]
    fn test_layer_visibility() {
        let mut layer = RenderLayer::new("background".to_string(), 0);
        layer.set_visible(false);
        assert!(!layer.visible);
        
        layer.set_visible(true);
        assert!(layer.visible);
    }

    #[test]
    fn test_entity_management() {
        let mut layer = RenderLayer::new("background".to_string(), 0);
        
        // エンティティの追加
        layer.add_entity(1);
        layer.add_entity(2);
        assert_eq!(layer.entities.len(), 2);
        assert!(layer.contains_entity(1));
        assert!(layer.contains_entity(2));
        
        // 重複追加の防止
        layer.add_entity(1);
        assert_eq!(layer.entities.len(), 2);
        
        // エンティティの削除
        layer.remove_entity(1);
        assert_eq!(layer.entities.len(), 1);
        assert!(!layer.contains_entity(1));
        assert!(layer.contains_entity(2));
        
        // 存在しないエンティティの削除
        layer.remove_entity(3);
        assert_eq!(layer.entities.len(), 1);
        
        // エンティティのクリア
        layer.clear_entities();
        assert!(layer.entities.is_empty());
    }

    #[test]
    fn test_entity_list() {
        let mut layer = RenderLayer::new("background".to_string(), 0);
        layer.add_entity(1);
        layer.add_entity(2);
        layer.add_entity(3);
        
        let entities = layer.get_entities();
        assert_eq!(entities.len(), 3);
        assert_eq!(entities[0], 1);
        assert_eq!(entities[1], 2);
        assert_eq!(entities[2], 3);
    }
} 