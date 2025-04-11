fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) {
    // レンダリングコンテキストを取得
    let renderer = resources.get::<Renderer>().unwrap();
    
    // レンダリング開始
    renderer.begin_frame();
    
    // 背景のレンダリング
    // TODO: 背景のレンダリング処理を実装
    
    // エンティティのレンダリング
    for entity in world.iter() {
        if let Some(transform) = world.get_component::<Transform>(entity) {
            if let Some(sprite) = world.get_component::<Sprite>(entity) {
                // スプライトのレンダリング
                renderer.draw_sprite(transform, sprite);
            }
        }
    }
    
    // UIのレンダリング
    // TODO: UIのレンダリング処理を実装
    
    // レンダリング終了
    renderer.end_frame();
} 