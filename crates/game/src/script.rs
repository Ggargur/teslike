//! Scripted input for automated checks. `TESLIKE_SCRIPT` holds
//! `seconds:command` steps separated by `;`, run on real time:
//!
//! - `down:attack` / `up:attack` (also block, dodge, kick, run, w, a, s, d)
//! - `aim:on` / `aim:off`: keep looking at the enemy
//! - `yaw:deg`, `pitch:deg`
//! - `shot:name`: screenshot to `$TESLIKE_SHOTS/name.png`
//! - `tp:x,z` / `etp:x,z`: teleport the player / the enemy
//! - `log`: print fighter states
//! - `exit`

use bevy::{
    app::AppExit,
    input::InputSystems,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use leafwing_input_manager::plugin::InputManagerSystem;

use crate::{
    combat::Combatant,
    enemy::Enemy,
    player::{Head, Player},
};

#[derive(Resource)]
struct Script {
    steps: Vec<(f32, String)>,
    /// Buttons the script holds; any other (real) input is ignored.
    mouse_held: Vec<MouseButton>,
    keys_held: Vec<KeyCode>,
    next: usize,
    aim: bool,
    t: f32,
}

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        let Ok(text) = std::env::var("TESLIKE_SCRIPT") else { return };
        let mut steps: Vec<(f32, String)> = text
            .split(';')
            .filter_map(|s| {
                let (t, c) = s.trim().split_once(':')?;
                Some((t.parse().ok()?, c.to_string()))
            })
            .collect();
        steps.sort_by(|a, b| a.0.total_cmp(&b.0));
        app.insert_resource(Script { steps, mouse_held: vec![], keys_held: vec![], next: 0, aim: false, t: 0.0 })
            .add_systems(PreUpdate, run.after(InputSystems).before(InputManagerSystem::Unify))
            .add_systems(Update, aim.before(crate::mover::MoveSet));
    }
}

fn run(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut script: ResMut<Script>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut head: Query<&mut Head>,
    mut fighters: Query<(&Combatant, &Name, &mut Transform, Has<Player>, Has<Enemy>)>,
    mut exit: MessageWriter<AppExit>,
) {
    script.t += time.delta_secs();
    // Drop whatever a human pressed while the window had focus.
    let real_mouse: Vec<MouseButton> = mouse.get_pressed().chain(mouse.get_just_released()).copied().collect();
    for b in real_mouse {
        if !script.mouse_held.contains(&b) {
            mouse.reset(b);
        }
    }
    let real_keys: Vec<KeyCode> = keys.get_pressed().chain(keys.get_just_released()).copied().collect();
    for k in real_keys {
        if !script.keys_held.contains(&k) {
            keys.reset(k);
        }
    }
    while script.next < script.steps.len() && script.steps[script.next].0 <= script.t {
        let cmd = script.steps[script.next].1.clone();
        script.next += 1;
        let (verb, arg) = cmd.split_once(':').unwrap_or((cmd.as_str(), ""));
        let key = |a: &str| match a {
            "dodge" => Some(KeyCode::Space),
            "kick" => Some(KeyCode::KeyF),
            "run" => Some(KeyCode::ShiftLeft),
            "w" => Some(KeyCode::KeyW),
            "a" => Some(KeyCode::KeyA),
            "s" => Some(KeyCode::KeyS),
            "d" => Some(KeyCode::KeyD),
            "f2" => Some(KeyCode::F2),
            "f3" => Some(KeyCode::F3),
            _ => None,
        };
        let button = |a: &str| match a {
            "attack" => Some(MouseButton::Left),
            "block" => Some(MouseButton::Right),
            _ => None,
        };
        match verb {
            "down" => {
                if let Some(b) = button(arg) {
                    mouse.press(b);
                    script.mouse_held.push(b);
                } else if let Some(k) = key(arg) {
                    keys.press(k);
                    script.keys_held.push(k);
                }
            }
            "up" => {
                if let Some(b) = button(arg) {
                    mouse.release(b);
                    script.mouse_held.retain(|x| *x != b);
                } else if let Some(k) = key(arg) {
                    keys.release(k);
                    script.keys_held.retain(|x| *x != k);
                }
            }
            "aim" => script.aim = arg == "on",
            "yaw" | "pitch" => {
                let v: f32 = arg.parse().unwrap_or(0.0);
                for mut h in &mut head {
                    if verb == "yaw" {
                        h.yaw = v.to_radians();
                    } else {
                        h.pitch = v.to_radians();
                    }
                }
            }
            "shot" => {
                let dir = std::env::var("TESLIKE_SHOTS").unwrap_or_else(|_| ".".into());
                commands.spawn(Screenshot::primary_window()).observe(save_to_disk(format!("{dir}/{arg}.png")));
            }
            "tp" | "etp" => {
                let v: Vec<f32> = arg.split(',').filter_map(|x| x.parse().ok()).collect();
                if let [x, z] = v[..] {
                    for (_, _, mut tf, is_p, is_e) in &mut fighters {
                        if (verb == "tp" && is_p) || (verb == "etp" && is_e) {
                            tf.translation.x = x;
                            tf.translation.z = z;
                        }
                    }
                }
            }
            "log" => {
                for (c, n, tf, _, _) in &fighters {
                    let f = &c.fighter;
                    info!("{n}: {:?} hp {:.0} st {:.0} at {:.2?}", f.phase(), f.health(), f.stamina(), tf.translation);
                }
            }
            "exit" => {
                exit.write(AppExit::Success);
            }
            _ => warn!("unknown script step {cmd}"),
        }
    }
}

fn aim(
    script: Res<Script>,
    mut head: Single<&mut Head>,
    player: Single<&Transform, With<Player>>,
    enemy: Query<&Transform, (With<Enemy>, Without<Player>)>,
) {
    if !script.aim {
        return;
    }
    let Some(e) = enemy.iter().next() else { return };
    let to = e.translation - player.translation;
    head.yaw = (-to.x).atan2(-to.z);
    head.pitch = (to.y - 0.6).atan2(to.with_y(0.0).length()).clamp(-0.6, 0.6);
}
