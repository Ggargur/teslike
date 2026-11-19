//! Every verb mapped once, for keyboard + mouse and gamepad.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Move,
    /// Mouse motion (pixels).
    #[actionlike(DualAxis)]
    Look,
    /// Right stick (-1..1).
    #[actionlike(DualAxis)]
    LookStick,
    Run,
    /// Tap for a light attack, hold to charge.
    Attack,
    Block,
    Dodge,
    Kick,
}

pub fn input_map() -> InputMap<Action> {
    let mut map = InputMap::default();
    map.insert_dual_axis(Action::Move, VirtualDPad::wasd())
        .insert_dual_axis(Action::Move, GamepadStick::LEFT)
        .insert_dual_axis(Action::Look, MouseMove::default())
        .insert_dual_axis(Action::LookStick, GamepadStick::RIGHT)
        .insert(Action::Run, KeyCode::ShiftLeft)
        .insert(Action::Run, GamepadButton::LeftThumb)
        .insert(Action::Attack, MouseButton::Left)
        .insert(Action::Attack, GamepadButton::RightTrigger2)
        .insert(Action::Block, MouseButton::Right)
        .insert(Action::Block, GamepadButton::LeftTrigger2)
        .insert(Action::Dodge, KeyCode::Space)
        .insert(Action::Dodge, GamepadButton::South)
        .insert(Action::Kick, KeyCode::KeyF)
        .insert(Action::Kick, GamepadButton::RightTrigger);
    map
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());
    }
}
