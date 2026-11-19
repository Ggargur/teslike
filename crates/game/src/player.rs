//! The Deus da Persistência in first person: body, look, head bob, tilt.

use avian3d::prelude::*;
use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use leafwing_input_manager::prelude::*;
use rules::Phase;
use serde::{Deserialize, Serialize};

use crate::{
    combat::Combatant,
    config::{ConfigFile, ConfigPlugin},
    input::{Action, input_map},
    mover::{Layer, Mover},
};

#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerConfig {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub accel: f32,
    pub decel: f32,
    pub gravity: f32,
    pub step_height: f32,
    pub eye_height: f32,
    pub radius: f32,
    pub height: f32,
    pub fov_deg: f32,
    /// Radians per pixel of mouse motion.
    pub mouse_sensitivity: f32,
    /// Radians per second at full stick.
    pub stick_sensitivity: f32,
    /// Head bob cycles per meter walked.
    pub bob_per_meter: f32,
    pub bob_vertical: f32,
    pub bob_lateral: f32,
    /// Camera roll when strafing at full speed (degrees).
    pub strafe_tilt_deg: f32,
    pub tilt_smoothing: f32,
    /// Speed multipliers while doing things.
    pub attack_speed_mult: f32,
    pub charge_speed_mult: f32,
    pub block_speed_mult: f32,
    pub dodge_speed: f32,
    pub dodge_roll_deg: f32,
    pub dodge_dip: f32,
    /// Forward lunge given by each strike (m/s of knockback-like push).
    pub light_lunge: f32,
    pub charged_lunge: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            walk_speed: 3.6,
            run_speed: 5.8,
            accel: 28.0,
            decel: 20.0,
            gravity: 22.0,
            step_height: 0.3,
            eye_height: 1.62,
            radius: 0.35,
            height: 1.8,
            fov_deg: 78.0,
            mouse_sensitivity: 0.0022,
            stick_sensitivity: 3.2,
            bob_per_meter: 0.55,
            bob_vertical: 0.035,
            bob_lateral: 0.025,
            strafe_tilt_deg: 2.2,
            tilt_smoothing: 9.0,
            attack_speed_mult: 0.45,
            charge_speed_mult: 0.6,
            block_speed_mult: 0.5,
            dodge_speed: 9.5,
            dodge_roll_deg: 7.0,
            dodge_dip: 0.18,
            light_lunge: 2.0,
            charged_lunge: 4.5,
        }
    }
}

impl ConfigFile for PlayerConfig {
    const FILE: &'static str = "player.ron";
}

#[derive(Component)]
pub struct Player;

/// The camera entity (child of the player body).
#[derive(Component, Default, Reflect)]
pub struct Head {
    pub yaw: f32,
    pub pitch: f32,
    pub bob_phase: f32,
    pub bob_weight: f32,
    roll: f32,
    /// Set by game feel: extra rotation (shake) and offset.
    pub shake_rot: Vec3,
    pub shake_offset: Vec3,
    /// 0 alive, rises to 1 while dying.
    pub death: f32,
    dodge: f32,
}

impl Head {
    /// Back to a fresh, upright view facing `yaw`.
    pub fn reset(&mut self, yaw: f32) {
        *self = Self { yaw, ..Self::default() };
    }
}

/// Direction of the current dodge (world space, horizontal).
#[derive(Component, Default)]
pub struct DodgeDir(pub Vec3);

pub const VIEWMODEL_LAYER: usize = 1;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConfigPlugin::<PlayerConfig>::default())
            .register_type::<Head>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (look, walk.before(crate::mover::MoveSet), compose_head.after(crate::mover::MoveSet))
                    .chain(),
            )
            .add_systems(Update, (grab_cursor, apply_config));
    }
}

pub fn spawn_point() -> Transform {
    Transform::from_xyz(-3.0, 0.0, 4.0)
}

fn spawn_player(mut commands: Commands, cfg: Res<PlayerConfig>, rules_cfg: Res<crate::combat::CombatRules>) {
    let body_h = cfg.height - 2.0 * cfg.radius;
    let start = spawn_point();
    commands
        .spawn((
            Name::new("Player"),
            Player,
            Combatant::new(&rules_cfg.0),
            ActionState::<Action>::default(),
            input_map(),
            DodgeDir::default(),
            Mover::default(),
            Transform::from_translation(start.translation + Vec3::Y * (cfg.height / 2.0 + 0.05)),
            Visibility::default(),
            RigidBody::Kinematic,
            Collider::capsule(cfg.radius, body_h),
            CollisionLayers::new(Layer::Character, [Layer::World, Layer::Character]),
            CustomPositionIntegration,
        ))
        .with_children(|p| {
            p.spawn((
                Name::new("Head"),
                Head { yaw: -std::f32::consts::FRAC_PI_4, ..default() },
                Camera3d::default(),
                Camera { order: 0, ..default() },
                Projection::Perspective(PerspectiveProjection { fov: cfg.fov_deg.to_radians(), near: 0.05, ..default() }),
                psx::PsxCamera,
                RenderLayers::layer(0),
                Transform::from_xyz(0.0, cfg.eye_height - cfg.height / 2.0, 0.0),
            ))
            .with_children(|h| {
                h.spawn((
                    Name::new("Viewmodel camera"),
                    Camera3d::default(),
                    Camera { order: 1, clear_color: ClearColorConfig::None, ..default() },
                    Projection::Perspective(PerspectiveProjection { fov: cfg.fov_deg.to_radians(), near: 0.02, ..default() }),
                    psx::PsxCamera,
                    RenderLayers::layer(VIEWMODEL_LAYER),
                    Transform::default(),
                ));
            });
        });
}

fn apply_config(cfg: Res<PlayerConfig>, mut q: Query<&mut Mover, With<Player>>) {
    if !cfg.is_changed() {
        return;
    }
    for mut m in &mut q {
        m.accel = cfg.accel;
        m.decel = cfg.decel;
        m.gravity = cfg.gravity;
        m.step_height = cfg.step_height;
    }
}

fn grab_cursor(
    mut cursor: Single<&mut CursorOptions>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    inspector: Res<crate::InspectorOpen>,
) {
    if inspector.0 || keys.just_pressed(KeyCode::Escape) {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    } else if mouse.just_pressed(MouseButton::Left) {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    }
}

fn look(
    time: Res<Time<Real>>,
    cfg: Res<PlayerConfig>,
    cursor: Single<&CursorOptions>,
    player: Single<(&ActionState<Action>, &Combatant), With<Player>>,
    mut head: Single<&mut Head>,
) {
    let (actions, fighter) = player.into_inner();
    if fighter.fighter.phase() == Phase::Dead {
        return;
    }
    let mut delta = Vec2::ZERO;
    if cursor.grab_mode != CursorGrabMode::None {
        delta += actions.axis_pair(&Action::Look) * cfg.mouse_sensitivity;
    }
    let stick = actions.clamped_axis_pair(&Action::LookStick);
    // Small deadzone and a response curve for precise aim.
    let stick = if stick.length() < 0.12 { Vec2::ZERO } else { stick * stick.length() };
    delta += Vec2::new(stick.x, -stick.y) * cfg.stick_sensitivity * time.delta_secs();
    head.yaw -= delta.x;
    head.pitch = (head.pitch - delta.y).clamp(-1.45, 1.45);
}

fn walk(
    time: Res<Time>,
    cfg: Res<PlayerConfig>,
    combat: Res<crate::combat::CombatRules>,
    head: Single<&Head>,
    player: Single<(&ActionState<Action>, &Combatant, &mut Mover, &mut Transform, &mut DodgeDir), With<Player>>,
) {
    let (actions, fighter, mut mover, mut tf, mut dodge) = player.into_inner();
    tf.rotation = Quat::from_rotation_y(head.yaw);
    let input = actions.clamped_axis_pair(&Action::Move);
    let input = if input.length() < 0.15 { Vec2::ZERO } else { input.clamp_length_max(1.0) };
    let fwd = tf.rotation * Vec3::NEG_Z;
    let right = tf.rotation * Vec3::X;
    let dir = fwd * input.y + right * input.x;
    let phase = fighter.fighter.phase();
    let speed = if actions.pressed(&Action::Run) && phase == Phase::Idle { cfg.run_speed } else { cfg.walk_speed };
    let mult = match phase {
        Phase::Idle => 1.0,
        Phase::Charging => cfg.charge_speed_mult,
        Phase::Blocking => cfg.block_speed_mult,
        Phase::Windup(_) | Phase::Active(_) | Phase::Recovery(_) => cfg.attack_speed_mult,
        Phase::Dodging => 0.0,
        Phase::Staggered | Phase::Unbalanced => 0.3,
        Phase::Stunned => 0.15,
        Phase::Dead => 0.0,
    };
    if phase == Phase::Dodging {
        // Dodge: fixed burst along the chosen direction, fading out.
        let k = 1.0 - (fighter.fighter.phase_time() / combat.0.dodge.duration).clamp(0.0, 1.0);
        mover.wish = dodge.0 * cfg.dodge_speed * (0.35 + 0.65 * k);
        mover.velocity = Vec3::new(mover.wish.x, mover.velocity.y, mover.wish.z);
    } else {
        mover.wish = dir * speed * mult;
        // Remember where a dodge would go: input direction, or backwards.
        dodge.0 = if dir.length_squared() > 0.01 { dir.normalize() } else { -fwd };
    }
    let _ = time;
}

fn compose_head(
    time: Res<Time>,
    real: Res<Time<Real>>,
    cfg: Res<PlayerConfig>,
    combat: Res<crate::combat::CombatRules>,
    player: Single<(&Mover, &Combatant, &Transform), (With<Player>, Without<Head>)>,
    head: Single<(&mut Head, &mut Transform)>,
) {
    let (mover, fighter, body) = player.into_inner();
    let (mut head, mut tf) = head.into_inner();
    let dt = time.delta_secs();
    let rdt = real.delta_secs();
    let local_v = body.rotation.inverse() * mover.total();
    let speed = Vec2::new(local_v.x, local_v.z).length();

    // Bob advances with distance walked, fades in/out with speed.
    if mover.grounded {
        head.bob_phase += speed * dt * cfg.bob_per_meter * std::f32::consts::TAU;
    }
    let target_w = if mover.grounded { (speed / cfg.walk_speed).min(1.3) } else { 0.0 };
    head.bob_weight += (target_w - head.bob_weight) * (1.0 - (-8.0 * dt).exp());
    let bob = Vec3::new(
        head.bob_phase.cos() * cfg.bob_lateral,
        -(head.bob_phase * 2.0).sin().abs() * cfg.bob_vertical,
        0.0,
    ) * head.bob_weight;

    // Roll into strafes, and into the dodge.
    let phase = fighter.fighter.phase();
    let dodge_t = if phase == Phase::Dodging {
        (fighter.fighter.phase_time() / combat.0.dodge.duration).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let dodge_curve = (dodge_t * std::f32::consts::PI).sin();
    head.dodge += (dodge_curve - head.dodge) * (1.0 - (-20.0 * rdt).exp());
    let strafe = (local_v.x / cfg.walk_speed).clamp(-1.5, 1.5);
    let target_roll = -strafe * cfg.strafe_tilt_deg.to_radians()
        - local_v.x.signum() * head.dodge * cfg.dodge_roll_deg.to_radians() * (phase == Phase::Dodging) as i32 as f32;
    head.roll += (target_roll - head.roll) * (1.0 - (-cfg.tilt_smoothing * dt).exp());

    // Death: slump to the floor.
    if phase == Phase::Dead {
        head.death = (head.death + dt * 1.6).min(1.0);
    } else {
        head.death = 0.0;
    }
    let d = head.death * head.death * (3.0 - 2.0 * head.death);

    let base = Vec3::Y * (cfg.eye_height - cfg.height / 2.0);
    tf.translation = base + bob + Vec3::Y * (-head.dodge * cfg.dodge_dip - d * (cfg.eye_height - 0.3)) + head.shake_offset;
    tf.rotation = Quat::from_euler(
        EulerRot::YXZ,
        head.shake_rot.y,
        head.pitch + head.shake_rot.x - d * 0.4,
        head.roll + head.shake_rot.z + d * 1.2,
    );
}
