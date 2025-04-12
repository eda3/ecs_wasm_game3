impl System for NetworkReliabilitySystem {
    fn name(&self) -> &'static str {
        "NetworkReliabilitySystem"
    }
    
    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        if let Some(network) = resources.get_mut::<NetworkResource>() {
            if !matches!(network.state, ConnectionState::Connected) {
                return Ok(()); // 接続されていない場合は処理しない
            }
            
            let current_time = js_sys::Date::now();
            
            // 確認応答が必要なメッセージの再送処理
            self.process_message_retransmission(network, current_time);
            
            // 接続保持メッセージの送信（必要な場合）
            self.send_keepalive_if_needed(network, current_time);
            
            // ネットワーク品質測定の更新
            self.update_network_metrics(network);
        }
        
        Ok(())
    }

    fn phase(&self) -> crate::ecs::SystemPhase {
        crate::ecs::SystemPhase::Update
    }

    fn priority(&self) -> SystemPriority {
        // 高優先度（先に実行）
        SystemPriority::new(10)
    }
} 