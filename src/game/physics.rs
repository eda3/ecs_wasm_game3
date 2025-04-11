fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) {
    // 物理エンジンを取得
    let physics_engine = resources.get::<PhysicsEngine>().unwrap();
    
    // 物理シミュレーションの更新
    physics_engine.update(delta_time);
    
    // エンティティの物理状態を更新
    for entity in world.iter() {
        if let Some(transform) = world.get_component::<Transform>(entity) {
            if let Some(rigid_body) = world.get_component::<RigidBody>(entity) {
                // 物理状態の更新
                physics_engine.update_entity(entity, transform, rigid_body);
                
                // 衝突判定
                if let Some(collider) = world.get_component::<Collider>(entity) {
                    physics_engine.check_collisions(entity, collider);
                }
            }
        }
    }
} 