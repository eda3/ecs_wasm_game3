fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) {
    // 入力マネージャーを取得
    let input_manager = resources.get::<InputManager>().unwrap();
    
    // 入力状態の更新
    input_manager.update();
    
    // プレイヤーエンティティの入力処理
    for entity in world.iter() {
        if let Some(player) = world.get_component::<Player>(entity) {
            if let Some(transform) = world.get_component::<Transform>(entity) {
                // キーボード入力の処理
                if input_manager.is_key_pressed(KeyCode::Up) {
                    transform.position.y += player.speed * delta_time;
                }
                if input_manager.is_key_pressed(KeyCode::Down) {
                    transform.position.y -= player.speed * delta_time;
                }
                if input_manager.is_key_pressed(KeyCode::Left) {
                    transform.position.x -= player.speed * delta_time;
                }
                if input_manager.is_key_pressed(KeyCode::Right) {
                    transform.position.x += player.speed * delta_time;
                }
                
                // マウス入力の処理
                if let Some(mouse_pos) = input_manager.get_mouse_position() {
                    // マウスの方向を向く
                    let direction = mouse_pos - transform.position;
                    transform.rotation = direction.y.atan2(direction.x);
                }
                
                // マウスクリックの処理
                if input_manager.is_mouse_button_pressed(MouseButton::Left) {
                    // 攻撃などのアクション
                    player.attack();
                }
            }
        }
    }
    
    // タッチ入力の処理
    for touch in &input_manager.touches {
        // タッチ位置の処理
        // TODO: タッチ位置に応じた処理を実装
    }
} 