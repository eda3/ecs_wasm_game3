//! ゲームシステムモジュール
//! 
//! ゲーム固有のシステムを実装します。

use wasm_bindgen::prelude::*;
use crate::ecs::{System, World, SystemPhase, SystemPriority};
use crate::ecs::resource::ResourceManager;
use crate::game::resources::TimeResource;

/// 時間管理システム
/// 
/// ゲームの時間を管理し、デルタタイムを計算します。
pub struct TimeSystem;

impl TimeSystem {
    /// 新しい時間管理システムを作成します。
    pub fn new() -> Self {
        Self {}
    }
}

impl System for TimeSystem {
    fn name(&self) -> &'static str {
        "TimeSystem"
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(0)
    }

    fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        // 時間リソースを取得または作成
        let time = world
            .get_resource_mut::<TimeResource>()
            .ok_or_else(|| JsValue::from_str("Time resource not found"))?;

        // 時間を更新
        time.elapsed_time += delta_time * time.time_scale;
        time.delta_time = delta_time * time.time_scale;

        Ok(())
    }
}

/// 入力処理システム
/// 
/// キーボードとマウスの入力を処理します。
pub struct InputSystem;

impl System for InputSystem {
    fn name(&self) -> &'static str {
        "InputSystem"
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Input
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(0)
    }

    fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        // 入力状態リソースを取得
        let input = world
            .get_resource::<crate::game::resources::InputState>()
            .ok_or_else(|| JsValue::from_str("InputState not found"))?;

        // TODO: 入力処理の実装

        Ok(())
    }
}

/// レンダリングシステム
/// 
/// ゲームの描画を行います。
pub struct RenderingSystem;

impl System for RenderingSystem {
    fn name(&self) -> &'static str {
        "RenderingSystem"
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Render
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(0)
    }

    fn run(&mut self, _world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: レンダリング処理の実装

        Ok(())
    }
}

/// 物理システム
/// 
/// ゲームの物理演算を行います。
pub struct PhysicsSystem;

impl System for PhysicsSystem {
    fn name(&self) -> &'static str {
        "PhysicsSystem"
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(1)
    }

    fn run(&mut self, _world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: 物理演算の実装

        Ok(())
    }
}

/// アニメーションシステム
/// 
/// スプライトのアニメーションを管理します。
pub struct AnimationSystem;

impl System for AnimationSystem {
    fn name(&self) -> &'static str {
        "AnimationSystem"
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(2)
    }

    fn run(&mut self, _world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: アニメーション処理の実装

        Ok(())
    }
}

/// サウンドシステム
/// 
/// ゲームのサウンドを管理します。
pub struct SoundSystem;

impl System for SoundSystem {
    fn name(&self) -> &'static str {
        "SoundSystem"
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(3)
    }

    fn run(&mut self, _world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: サウンド処理の実装

        Ok(())
    }
}

/// ゲーム状態管理システム
/// 
/// ゲームの状態遷移を管理します。
pub struct GameStateSystem;

impl GameStateSystem {
    /// 新しいゲーム状態管理システムを作成します。
    pub fn new() -> Self {
        Self {}
    }
}

impl System for GameStateSystem {
    fn name(&self) -> &'static str {
        "GameStateSystem"
    }

    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }

    fn priority(&self) -> SystemPriority {
        SystemPriority::new(4)
    }

    fn run(&mut self, _world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        // TODO: ゲーム状態管理の実装

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_system_phases() {
        let time_system = TimeSystem;
        assert_eq!(time_system.phase(), SystemPhase::Update);
        assert_eq!(time_system.priority(), SystemPriority::new(0));

        let input_system = InputSystem;
        assert_eq!(input_system.phase(), SystemPhase::Input);
        assert_eq!(input_system.priority(), SystemPriority::new(0));

        let rendering_system = RenderingSystem;
        assert_eq!(rendering_system.phase(), SystemPhase::Render);
        assert_eq!(rendering_system.priority(), SystemPriority::new(0));

        let physics_system = PhysicsSystem;
        assert_eq!(physics_system.phase(), SystemPhase::Update);
        assert_eq!(physics_system.priority(), SystemPriority::new(1));

        let animation_system = AnimationSystem;
        assert_eq!(animation_system.phase(), SystemPhase::Update);
        assert_eq!(animation_system.priority(), SystemPriority::new(2));

        let sound_system = SoundSystem;
        assert_eq!(sound_system.phase(), SystemPhase::Update);
        assert_eq!(sound_system.priority(), SystemPriority::new(3));

        let game_state_system = GameStateSystem;
        assert_eq!(game_state_system.phase(), SystemPhase::Update);
        assert_eq!(game_state_system.priority(), SystemPriority::new(4));
    }
} 