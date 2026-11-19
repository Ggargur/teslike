//! Opponents: a training dummy that strikes on a fixed rhythm, and a
//! humanoid with a state-machine AI. Both are plain `rules::Fighter`s, the
//! same as the player; this module only decides their intents and
//! animates their rigid-part bodies procedurally.

use std::collections::HashMap;

use avian3d::prelude::*;
use bevy::{prelude::*, world_serialization::WorldInstanceReady};
use rules::{Event, Intent, Phase, Strike};
use serde::{Deserialize, Serialize};

use crate::{
    combat::{Blade, CombatMsg, CombatRules, CombatSet, Combatant},
    config::{ConfigFile, ConfigPlugin},
    feel::{FeelConfig, Sfx, WoodenTarget, play, rand_f32},
    mover::{Layer, Mover},
    player::Player,
};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Euler degrees per joint. Missing joints stay at rest.
#[derive(Clone, Copy, Default, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct BodyPose {
    pub torso: [f32; 3],
    pub head: [f32; 3],
    pub upper_arm_r: [f32; 3],
    pub lower_arm_r: [f32; 3],
    pub upper_arm_l: [f32; 3],
    pub lower_arm_l: [f32; 3],
    pub thigh_r: [f32; 3],
    pub shin_r: [f32; 3],
    /// Pelvis drop in meters (crouch).
    pub crouch: f32,
}

impl BodyPose {
    fn get(&self, joint: &str) -> Vec3 {
        let a = match joint {
            "torso" => self.torso,
            "head" => self.head,
            "upper_arm_r" => self.upper_arm_r,
            "lower_arm_r" => self.lower_arm_r,
            "upper_arm_l" => self.upper_arm_l,
            "lower_arm_l" => self.lower_arm_l,
            "thigh_r" => self.thigh_r,
            "shin_r" => self.shin_r,
            _ => [0.0; 3],
        };
        Vec3::from_array(a)
    }

    fn lerp(&self, b: &BodyPose, k: f32) -> BodyPose {
        let l = |x: [f32; 3], y: [f32; 3]| Vec3::from_array(x).lerp(Vec3::from_array(y), k).to_array();
        BodyPose {
            torso: l(self.torso, b.torso),
            head: l(self.head, b.head),
            upper_arm_r: l(self.upper_arm_r, b.upper_arm_r),
            lower_arm_r: l(self.lower_arm_r, b.lower_arm_r),
            upper_arm_l: l(self.upper_arm_l, b.upper_arm_l),
            lower_arm_l: l(self.lower_arm_l, b.lower_arm_l),
            thigh_r: l(self.thigh_r, b.thigh_r),
            shin_r: l(self.shin_r, b.shin_r),
            crouch: self.crouch + (b.crouch - self.crouch) * k,
        }
    }
}

const fn pose(torso: [f32; 3], ua_r: [f32; 3], la_r: [f32; 3], crouch: f32) -> BodyPose {
    BodyPose {
        torso,
        head: [0.0; 3],
        upper_arm_r: ua_r,
        lower_arm_r: la_r,
        upper_arm_l: [25.0, 0.0, -12.0],
        lower_arm_l: [45.0, 0.0, 0.0],
        thigh_r: [0.0; 3],
        shin_r: [0.0; 3],
        crouch,
    }
}

/// One of the humanoid's attacks.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct AttackMove {
    pub name: String,
    /// Seconds of readable wind-back before the strike starts.
    pub telegraph: f32,
    /// Holds attack through the telegraph: becomes a charged strike.
    pub charged: bool,
    /// Forward push when the strike starts (m/s).
    pub lunge: f32,
    pub telegraph_pitch: f32,
    pub telegraph_pose: BodyPose,
    pub end_pose: BodyPose,
    /// Relative likelihood.
    pub weight: f32,
}

impl Default for AttackMove {
    fn default() -> Self {
        Self {
            name: "slash".into(),
            telegraph: 0.45,
            charged: false,
            lunge: 1.5,
            telegraph_pitch: 1.0,
            telegraph_pose: BodyPose::default(),
            end_pose: BodyPose::default(),
            weight: 1.0,
        }
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EnemyConfig {
    pub walk_speed: f32,
    pub strafe_speed: f32,
    pub turn_speed: f32,
    /// Turn speed while a strike is in its windup/active window (lets the player sidestep).
    pub turn_speed_striking: f32,
    pub engage_range: f32,
    pub attack_range: f32,
    pub circle_min: f32,
    pub circle_max: f32,
    /// Chance to raise the guard when the player starts a strike nearby.
    pub block_chance: f32,
    pub block_hold: f32,
    pub reaction_time: f32,
    /// Chance to kick a player who's turtling behind their guard.
    pub kick_chance: f32,
    pub recover_pause: f32,
    pub attacks: Vec<AttackMove>,
    pub guard: BodyPose,
    pub block: BodyPose,
    pub reel: BodyPose,
    pub kick: BodyPose,
    pub joint_stiffness: f32,
    pub joint_damping: f32,
    pub strike_stiffness: f32,
    pub walk_swing_deg: f32,
    pub walk_cycle_per_meter: f32,
    pub hit_flinch_deg: f32,
    pub death_push: f32,
    pub death_spin: f32,
    // Dummy
    pub dummy_interval: f32,
    pub dummy_telegraph: f32,
    pub dummy_wobble_stiffness: f32,
    pub dummy_wobble_damping: f32,
    pub dummy_wobble_kick: f32,
}

impl Default for EnemyConfig {
    fn default() -> Self {
        Self {
            walk_speed: 2.6,
            strafe_speed: 1.6,
            turn_speed: 6.0,
            turn_speed_striking: 1.5,
            engage_range: 3.2,
            attack_range: 1.75,
            circle_min: 0.6,
            circle_max: 1.8,
            block_chance: 0.45,
            block_hold: 0.7,
            reaction_time: 0.18,
            kick_chance: 0.5,
            recover_pause: 0.35,
            attacks: vec![
                AttackMove {
                    name: "corte".into(),
                    telegraph: 0.5,
                    charged: false,
                    lunge: 1.8,
                    telegraph_pitch: 1.1,
                    telegraph_pose: pose([0.0, -40.0, 0.0], [60.0, 0.0, 75.0], [25.0, 0.0, 0.0], 0.03),
                    end_pose: pose([0.0, 45.0, 0.0], [85.0, 0.0, -25.0], [10.0, 0.0, 0.0], 0.05),
                    weight: 1.0,
                },
                AttackMove {
                    name: "golpe por cima".into(),
                    telegraph: 0.95,
                    charged: true,
                    lunge: 2.5,
                    telegraph_pitch: 0.7,
                    telegraph_pose: pose([14.0, -10.0, 0.0], [165.0, 0.0, 10.0], [35.0, 0.0, 0.0], 0.0),
                    end_pose: pose([-28.0, 5.0, 0.0], [75.0, 0.0, 0.0], [5.0, 0.0, 0.0], 0.15),
                    weight: 0.6,
                },
                AttackMove {
                    name: "estocada".into(),
                    telegraph: 0.65,
                    charged: false,
                    lunge: 5.0,
                    telegraph_pitch: 1.4,
                    telegraph_pose: pose([0.0, -30.0, 0.0], [10.0, 0.0, 25.0], [95.0, 0.0, 0.0], 0.12),
                    end_pose: pose([-12.0, 10.0, 0.0], [92.0, 0.0, 0.0], [2.0, 0.0, 0.0], 0.08),
                    weight: 0.8,
                },
            ],
            guard: pose([0.0, -10.0, 0.0], [45.0, 0.0, 12.0], [55.0, 0.0, 0.0], 0.05),
            block: pose([0.0, 15.0, 0.0], [55.0, 0.0, -25.0], [80.0, 0.0, -70.0], 0.1),
            reel: BodyPose { head: [25.0, 0.0, 0.0], ..pose([22.0, 10.0, 8.0], [15.0, 0.0, 45.0], [20.0, 0.0, 0.0], 0.0) },
            kick: BodyPose { thigh_r: [85.0, 0.0, 0.0], shin_r: [-10.0, 0.0, 0.0], ..pose([12.0, 0.0, 0.0], [45.0, 0.0, 30.0], [40.0, 0.0, 0.0], 0.0) },
            joint_stiffness: 180.0,
            joint_damping: 16.0,
            strike_stiffness: 900.0,
            walk_swing_deg: 28.0,
            walk_cycle_per_meter: 0.9,
            hit_flinch_deg: 600.0,
            death_push: 6.0,
            death_spin: 5.0,
            dummy_interval: 2.4,
            dummy_telegraph: 0.75,
            dummy_wobble_stiffness: 70.0,
            dummy_wobble_damping: 5.0,
            dummy_wobble_kick: 4.0,
        }
    }
}

impl ConfigFile for EnemyConfig {
    const FILE: &'static str = "enemy.ron";
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Enemy;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Dummy,
    Humanoid,
}

#[derive(Clone, Debug, PartialEq)]
enum Brain {
    Approach,
    Circle { dir: f32, left: f32 },
    Close { attack: usize },
    Telegraph { attack: usize, t: f32 },
    /// Strike handed to the rules; waiting for it to finish.
    Striking { attack: usize },
    Blocking { left: f32 },
    Pause { left: f32 },
}

#[derive(Component)]
pub struct Ai {
    brain: Brain,
    /// Delay before reacting to what the player does.
    react: f32,
    /// Seconds the player has been blocking in front of us.
    turtle: f32,
    /// Dummy rhythm clock.
    clock: f32,
    /// Last attack used (for animation during the rules' phases).
    attack: usize,
}

/// A procedurally animated joint of the rigid-part body.
#[derive(Component)]
struct Joint {
    name: String,
    rest: Quat,
    rest_pos: Vec3,
    rot: Vec3,
    vel: Vec3,
}

/// The visual model under the body.
#[derive(Component)]
struct Model {
    owner: Entity,
}

/// Walk cycle and dummy wobble state.
#[derive(Component, Default)]
struct Gait {
    phase: f32,
    wobble: Vec2,
    wobble_vel: Vec2,
}

#[derive(Component, Default)]
pub struct Joints(HashMap<String, Entity>);

pub const MODEL_DROP: f32 = 0.9;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConfigPlugin::<EnemyConfig>::default())
            .add_observer(rig_model)
            .add_systems(
                Update,
                ((think_dummy, think_humanoid).before(CombatSet), (animate, flinch, die).after(CombatSet)),
            );
    }
}

pub fn spawn_enemy(commands: &mut Commands, assets: &AssetServer, rules: &CombatRules, kind: Kind, at: Vec3) -> Entity {
    let (radius, half) = (0.38, 0.9);
    let body = commands
        .spawn((
            Name::new(format!("{kind:?}")),
            Enemy,
            kind,
            Combatant::new(&rules.0),
            Ai { brain: Brain::Approach, react: 0.0, turtle: 0.0, clock: 0.0, attack: 0 },
            Mover { accel: 14.0, decel: 14.0, ..default() },
            Gait::default(),
            Joints::default(),
            Transform::from_translation(at + Vec3::Y * (half + 0.05)),
            Visibility::default(),
            RigidBody::Kinematic,
            Collider::capsule(radius, half * 2.0 - radius * 2.0),
            CollisionLayers::new(Layer::Character, [Layer::World, Layer::Character]),
            CustomPositionIntegration,
        ))
        .id();
    if kind == Kind::Dummy {
        commands.entity(body).insert(WoodenTarget);
    }
    let file = match kind {
        Kind::Dummy => "models/dummy.glb",
        Kind::Humanoid => "models/humanoid.glb",
    };
    commands.spawn((
        Name::new("Model"),
        Model { owner: body },
        WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(file))),
        Transform::from_xyz(0.0, -MODEL_DROP, 0.0),
        ChildOf(body),
    ));
    body
}

/// When a model finishes spawning, find its named parts and wire joints
/// and the blade.
fn rig_model(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    models: Query<&Model>,
    children: Query<&Children>,
    nodes: Query<(&Name, &Transform)>,
    mut joints: Query<&mut Joints>,
    kinds: Query<&Kind>,
) {
    let Ok(model) = models.get(ready.entity) else { return };
    let owner = model.owner;
    let Ok(kind) = kinds.get(owner) else { return };
    let mut found = HashMap::new();
    for e in children.iter_descendants(ready.entity) {
        let Ok((name, tf)) = nodes.get(e) else { continue };
        let n = name.as_str().to_string();
        let known = matches!(
            n.as_str(),
            "pelvis" | "torso" | "head" | "upper_arm_l" | "lower_arm_l" | "upper_arm_r" | "lower_arm_r" | "thigh_l"
                | "shin_l" | "thigh_r" | "shin_r" | "sword" | "arm" | "dummy"
        );
        if !known || found.contains_key(&n) {
            continue;
        }
        commands.entity(e).insert(Joint { name: n.clone(), rest: tf.rotation, rest_pos: tf.translation, rot: Vec3::ZERO, vel: Vec3::ZERO });
        found.insert(n, e);
    }
    let blade = match kind {
        Kind::Humanoid => found.get("sword").map(|&e| (e, Vec3::NEG_Y, 0.12, 1.15)),
        Kind::Dummy => found.get("arm").map(|&e| (e, Vec3::X, 0.1, 1.0)),
    };
    if let Some((e, axis, from, to)) = blade {
        commands.entity(e).insert(Blade { owner, axis, from, to, prev: None });
    }
    if let Ok(mut j) = joints.get_mut(owner) {
        j.0 = found;
    }
}

// ---------------------------------------------------------------------------
// Brains
// ---------------------------------------------------------------------------

fn face(tf: &mut Transform, target: Vec3, speed: f32, dt: f32) {
    let to = (target - tf.translation).with_y(0.0);
    if to.length_squared() < 1e-4 {
        return;
    }
    let want = Quat::from_rotation_arc(Vec3::NEG_Z, to.normalize());
    tf.rotation = tf.rotation.slerp(want, (speed * dt).min(1.0));
}

fn think_dummy(
    mut commands: Commands,
    time: Res<Time>,
    rules: Res<CombatRules>,
    cfg: Res<EnemyConfig>,
    feel: Res<FeelConfig>,
    sfx: Res<Sfx>,
    player: Single<&Transform, (With<Player>, Without<Enemy>)>,
    mut q: Query<(&Kind, &mut Ai, &mut Combatant, &mut Transform, &mut Mover), With<Enemy>>,
) {
    let dt = time.delta_secs();
    for (kind, mut ai, mut c, mut tf, mut mv) in &mut q {
        if *kind != Kind::Dummy || c.fighter.phase() == Phase::Dead {
            continue;
        }
        mv.wish = Vec3::ZERO;
        face(&mut tf, player.translation, 2.0, dt);
        if c.fighter.phase() != Phase::Idle {
            continue;
        }
        ai.clock += dt;
        let tele_start = cfg.dummy_interval - cfg.dummy_telegraph;
        if ai.clock - dt < tele_start && ai.clock >= tele_start {
            play(&mut commands, &feel, &sfx.telegraph, 0.9, 1.2);
        }
        if ai.clock >= cfg.dummy_interval {
            ai.clock = 0.0;
            c.fighter.act(Intent::AttackPressed, &rules.0);
            c.fighter.act(Intent::AttackReleased, &rules.0);
        }
    }
}

fn pick_attack(cfg: &EnemyConfig) -> usize {
    let total: f32 = cfg.attacks.iter().map(|a| a.weight.max(0.0)).sum();
    let mut r = rand_f32() * total;
    for (i, a) in cfg.attacks.iter().enumerate() {
        r -= a.weight.max(0.0);
        if r <= 0.0 {
            return i;
        }
    }
    0
}

fn think_humanoid(
    mut commands: Commands,
    time: Res<Time>,
    rules: Res<CombatRules>,
    cfg: Res<EnemyConfig>,
    feel: Res<FeelConfig>,
    sfx: Res<Sfx>,
    player: Single<(&Transform, &Combatant), (With<Player>, Without<Enemy>)>,
    mut q: Query<(&Kind, &mut Ai, &mut Combatant, &mut Transform, &mut Mover), (With<Enemy>, Without<Player>)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 || cfg.attacks.is_empty() {
        return;
    }
    let (ptf, pc) = player.into_inner();
    let pphase = pc.fighter.phase();
    let player_alive = pphase != Phase::Dead;
    let r = &rules.0;
    for (kind, mut ai, mut c, mut tf, mut mv) in &mut q {
        if *kind != Kind::Humanoid {
            continue;
        }
        let f = &mut c.fighter;
        let phase = f.phase();
        if phase == Phase::Dead {
            mv.wish = Vec3::ZERO;
            continue;
        }
        let to = (ptf.translation - tf.translation).with_y(0.0);
        let dist = to.length();
        let dir = to.normalize_or(Vec3::NEG_Z);
        let side = Vec3::new(-dir.z, 0.0, dir.x);

        // Reeling: the rules own the body.
        if matches!(phase, Phase::Staggered | Phase::Stunned | Phase::Unbalanced) {
            mv.wish = Vec3::ZERO;
            if !matches!(ai.brain, Brain::Pause { .. }) {
                ai.brain = Brain::Pause { left: cfg.recover_pause };
            }
            continue;
        }
        let striking = matches!(phase, Phase::Windup(_) | Phase::Active(_));
        face(&mut tf, ptf.translation, if striking { cfg.turn_speed_striking } else { cfg.turn_speed }, dt);
        if !player_alive {
            mv.wish = Vec3::ZERO;
            if phase == Phase::Blocking {
                f.act(Intent::BlockReleased, r);
            }
            continue;
        }

        // Track turtling, and react to the player's strikes with the guard.
        if pphase == Phase::Blocking && dist < cfg.attack_range + 0.4 {
            ai.turtle += dt;
        } else {
            ai.turtle = 0.0;
        }
        let threat = matches!(pphase, Phase::Charging | Phase::Windup(_)) && dist < cfg.attack_range + 1.0;
        let can_react = matches!(ai.brain, Brain::Approach | Brain::Circle { .. } | Brain::Close { .. } | Brain::Pause { .. });
        if threat && can_react && phase == Phase::Idle {
            ai.react += dt;
            if ai.react >= cfg.reaction_time {
                ai.react = 0.0;
                if rand_f32() < cfg.block_chance {
                    f.act(Intent::BlockPressed, r);
                    // Only commit to blocking if the guard actually went up.
                    if f.phase() == Phase::Blocking {
                        ai.brain = Brain::Blocking { left: cfg.block_hold };
                    }
                }
            }
        } else if !threat {
            ai.react = 0.0;
        }

        let brain = ai.brain.clone();
        ai.brain = match brain {
            Brain::Approach => {
                mv.wish = dir * cfg.walk_speed;
                if dist < cfg.engage_range {
                    Brain::Circle { dir: if rand_f32() < 0.5 { -1.0 } else { 1.0 }, left: cfg.circle_min + rand_f32() * (cfg.circle_max - cfg.circle_min) }
                } else {
                    Brain::Approach
                }
            }
            Brain::Circle { dir: s, left } => {
                // Strafe around, drifting to keep engage distance.
                let keep = (dist - cfg.engage_range * 0.85).clamp(-1.0, 1.0);
                mv.wish = side * s * cfg.strafe_speed + dir * keep * cfg.walk_speed * 0.6;
                let left = left - dt;
                if dist > cfg.engage_range * 1.6 {
                    Brain::Approach
                } else if ai.turtle > 0.6 && rand_f32() < cfg.kick_chance * dt * 4.0 && dist < cfg.attack_range {
                    f.act(Intent::Kick, r);
                    Brain::Striking { attack: usize::MAX }
                } else if left <= 0.0 {
                    Brain::Close { attack: pick_attack(&cfg) }
                } else {
                    Brain::Circle { dir: s, left }
                }
            }
            Brain::Close { attack } => {
                mv.wish = dir * cfg.walk_speed;
                if dist <= cfg.attack_range {
                    mv.wish = Vec3::ZERO;
                    let a = &cfg.attacks[attack.min(cfg.attacks.len() - 1)];
                    play(&mut commands, &feel, &sfx.telegraph, 1.0, a.telegraph_pitch);
                    if a.charged {
                        f.act(Intent::AttackPressed, r);
                    }
                    ai.attack = attack;
                    Brain::Telegraph { attack, t: 0.0 }
                } else if ai.turtle > 0.6 && dist < cfg.attack_range + 0.3 && rand_f32() < cfg.kick_chance {
                    f.act(Intent::Kick, r);
                    Brain::Striking { attack: usize::MAX }
                } else {
                    Brain::Close { attack }
                }
            }
            Brain::Telegraph { attack, t } => {
                let a = &cfg.attacks[attack.min(cfg.attacks.len() - 1)];
                mv.wish = dir * 0.4;
                let t = t + dt;
                if t >= a.telegraph {
                    if !a.charged {
                        f.act(Intent::AttackPressed, r);
                    }
                    f.act(Intent::AttackReleased, r);
                    mv.knockback += dir * a.lunge;
                    Brain::Striking { attack }
                } else {
                    Brain::Telegraph { attack, t }
                }
            }
            Brain::Striking { attack } => {
                mv.wish = Vec3::ZERO;
                if matches!(phase, Phase::Idle) {
                    let _ = attack;
                    Brain::Pause { left: cfg.recover_pause * (0.5 + rand_f32()) }
                } else {
                    Brain::Striking { attack }
                }
            }
            Brain::Blocking { left } => {
                mv.wish = -dir * 0.6;
                let left = left - dt;
                if phase != Phase::Blocking || left <= 0.0 {
                    f.act(Intent::BlockReleased, r);
                    // Counter right after a successful guard, sometimes.
                    if rand_f32() < 0.5 { Brain::Close { attack: pick_attack(&cfg) } } else { Brain::Pause { left: 0.2 } }
                } else {
                    Brain::Blocking { left }
                }
            }
            Brain::Pause { left } => {
                mv.wish = -dir * 0.5;
                let left = left - dt;
                if left <= 0.0 {
                    Brain::Circle { dir: if rand_f32() < 0.5 { -1.0 } else { 1.0 }, left: cfg.circle_min + rand_f32() * (cfg.circle_max - cfg.circle_min) }
                } else {
                    Brain::Pause { left }
                }
            }
        };
    }
}

// ---------------------------------------------------------------------------
// Bodies
// ---------------------------------------------------------------------------

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Target pose for a humanoid right now, and whether it's mid-strike.
fn humanoid_pose(cfg: &EnemyConfig, rules: &rules::CombatConfig, ai: &Ai, c: &Combatant) -> (BodyPose, bool) {
    let f = &c.fighter;
    let t = f.phase_time();
    let attack = cfg.attacks.get(ai.attack.min(cfg.attacks.len().saturating_sub(1)));
    match (f.phase(), &ai.brain, attack) {
        (Phase::Stunned | Phase::Unbalanced | Phase::Staggered, _, _) => (cfg.reel, false),
        (Phase::Dead, _, _) => (BodyPose { crouch: 0.0, ..BodyPose::default() }, false),
        (Phase::Blocking, _, _) => (cfg.block, false),
        (Phase::Windup(Strike::Kick) | Phase::Active(Strike::Kick), _, _) => (cfg.kick, true),
        (Phase::Recovery(Strike::Kick), _, _) => (cfg.guard, false),
        (_, Brain::Telegraph { t, .. }, Some(a)) => {
            let k = smooth(t / (a.telegraph * 0.7));
            (cfg.guard.lerp(&a.telegraph_pose, k), false)
        }
        (Phase::Charging | Phase::Windup(_), _, Some(a)) => (a.telegraph_pose, false),
        (Phase::Active(s), _, Some(a)) => {
            let k = smooth(t / rules.strike(s).active);
            (a.telegraph_pose.lerp(&a.end_pose, k * 1.1), true)
        }
        (Phase::Recovery(s), _, Some(a)) => {
            let k = smooth(t / rules.strike(s).recovery);
            (a.end_pose.lerp(&cfg.guard, k), false)
        }
        _ => (cfg.guard, false),
    }
}

fn animate(
    time: Res<Time>,
    cfg: Res<EnemyConfig>,
    rules: Res<CombatRules>,
    mut bodies: Query<(&Kind, &Ai, &Combatant, &Mover, &mut Gait, &Joints, &Transform), With<Enemy>>,
    mut joints: Query<(&mut Joint, &mut Transform), Without<Enemy>>,
) {
    let dt = time.delta_secs().min(1.0 / 30.0);
    if dt <= 0.0 {
        return;
    }
    for (kind, ai, c, mv, mut gait, js, body_tf) in &mut bodies {
        let local_v = body_tf.rotation.inverse() * mv.velocity;
        let speed = Vec2::new(local_v.x, local_v.z).length();
        gait.phase += speed * dt * cfg.walk_cycle_per_meter * std::f32::consts::TAU;
        match kind {
            Kind::Humanoid => {
                let (target, swing) = humanoid_pose(&cfg, &rules.0, ai, c);
                let k = if swing { cfg.strike_stiffness } else { cfg.joint_stiffness };
                let d = 2.0 * k.sqrt() * (cfg.joint_damping / (2.0 * cfg.joint_stiffness.sqrt()));
                let walk = (speed / cfg.walk_speed).min(1.0) * cfg.walk_swing_deg;
                let s = gait.phase.sin() * walk;
                for &e in js.0.values() {
                    let Ok((mut j, mut tf)) = joints.get_mut(e) else { continue };
                    let mut goal = target.get(&j.name);
                    match j.name.as_str() {
                        "thigh_l" => goal.x += s,
                        "thigh_r" => goal.x -= s,
                        "shin_l" => goal.x -= s.max(0.0) * 1.2,
                        "shin_r" => goal.x -= (-s).max(0.0) * 1.2,
                        "upper_arm_l" => goal.x -= s * 0.4,
                        _ => {}
                    }
                    let acc = (goal - j.rot) * k - j.vel * d;
                    j.vel += acc * dt;
                    let v = j.vel;
                    j.rot += v * dt;
                    let r = j.rot.to_radians_vec();
                    tf.rotation = j.rest * Quat::from_euler(EulerRot::YXZ, r.y, r.x, r.z);
                    if j.name == "pelvis" {
                        let bob = (gait.phase * 2.0).sin().abs() * 0.03 * (speed / cfg.walk_speed).min(1.0);
                        tf.translation = j.rest_pos - Vec3::Y * (target.crouch + bob);
                    }
                }
            }
            Kind::Dummy => {
                // The whole post wobbles on a spring; the arm swings with the strike.
                let acc = -gait.wobble * cfg.dummy_wobble_stiffness - gait.wobble_vel * cfg.dummy_wobble_damping;
                gait.wobble_vel += acc * dt;
                let wv = gait.wobble_vel;
                gait.wobble += wv * dt;
                let f = &c.fighter;
                let t = f.phase_time();
                let tele = cfg.dummy_telegraph;
                let arm = match f.phase() {
                    Phase::Idle if ai.clock > cfg.dummy_interval - tele => {
                        -70.0 * smooth((ai.clock - (cfg.dummy_interval - tele)) / tele)
                    }
                    Phase::Windup(_) => -70.0,
                    Phase::Active(s) => -70.0 + 190.0 * smooth(t / rules.0.strike(s).active),
                    Phase::Recovery(s) => 120.0 * (1.0 - smooth(t / rules.0.strike(s).recovery)),
                    Phase::Stunned | Phase::Unbalanced => -20.0 + (t * 20.0).sin() * 10.0,
                    _ => 0.0,
                };
                for &e in js.0.values() {
                    let Ok((mut j, mut tf)) = joints.get_mut(e) else { continue };
                    match j.name.as_str() {
                        "dummy" => {
                            tf.rotation = j.rest * Quat::from_euler(EulerRot::XYZ, gait.wobble.x, 0.0, gait.wobble.y);
                        }
                        "arm" if matches!(f.phase(), Phase::Active(_)) => {
                            // The swing itself is keyed, not sprung: it must
                            // sweep the front within the active window.
                            j.rot.y = arm;
                            j.vel.y = 0.0;
                            tf.rotation = j.rest * Quat::from_rotation_y(arm.to_radians());
                        }
                        "arm" => {
                            let goal = arm;
                            let acc = (goal - j.rot.y) * cfg.strike_stiffness - j.vel.y * 40.0;
                            j.vel.y += acc * dt;
                            let v = j.vel.y;
                            j.rot.y += v * dt;
                            tf.rotation = j.rest * Quat::from_rotation_y(j.rot.y.to_radians());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

trait ToRadians {
    fn to_radians_vec(self) -> Vec3;
}

impl ToRadians for Vec3 {
    fn to_radians_vec(self) -> Vec3 {
        self * (std::f32::consts::PI / 180.0)
    }
}

/// Hits jolt the body: joints get a velocity kick, the dummy wobbles.
fn flinch(
    cfg: Res<EnemyConfig>,
    mut msgs: MessageReader<CombatMsg>,
    mut bodies: Query<(&Joints, &mut Gait, &Transform), With<Enemy>>,
    mut joints: Query<&mut Joint>,
) {
    for m in msgs.read() {
        let strength = match m.event {
            Event::Hit { damage } => (damage / 20.0).min(2.5),
            Event::Kicked | Event::GuardBroken => 1.5,
            Event::Blocked => 0.5,
            _ => continue,
        };
        let Ok((js, mut gait, tf)) = bodies.get_mut(m.target) else { continue };
        let local = tf.rotation.inverse() * m.dir;
        gait.wobble_vel += Vec2::new(-local.z, local.x) * cfg.dummy_wobble_kick * strength;
        for name in ["torso", "head"] {
            if let Some(&e) = js.0.get(name)
                && let Ok(mut j) = joints.get_mut(e)
            {
                j.vel += Vec3::new(-local.z * 0.6 + 0.4, local.x * 0.5, -local.x) * cfg.hit_flinch_deg * strength;
            }
        }
    }
}

/// Death hands the body to physics with a shove.
fn die(
    mut commands: Commands,
    cfg: Res<EnemyConfig>,
    mut msgs: MessageReader<CombatMsg>,
    enemies: Query<(), With<Enemy>>,
) {
    for m in msgs.read() {
        if m.event != Event::Died || !enemies.contains(m.target) {
            continue;
        }
        // Hazard deaths (no direction) just go limp.
        let push = m.dir;
        let spin = Vec3::new(push.z, 0.0, -push.x) * cfg.death_spin;
        commands
            .entity(m.target)
            .remove::<(Mover, CustomPositionIntegration)>()
            .insert((
                RigidBody::Dynamic,
                LinearVelocity(push * cfg.death_push + Vec3::Y * 2.0 * push.length()),
                AngularVelocity(spin),
                CollisionLayers::new(Layer::World, [Layer::World]),
            ));
    }
}
