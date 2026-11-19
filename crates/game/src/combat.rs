//! Bridge between the pure `rules` crate and the world: turns input into
//! intents, ticks fighters, sweeps blades and kicks through physics, applies
//! hazards, and publishes what happened as [`CombatMsg`] for game feel.

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rules::{CombatConfig, Event, Fighter, Intent, Phase, Strike};
use serde::{Deserialize, Serialize};

use crate::{
    config::{ConfigFile, ConfigPlugin},
    input::Action,
    mover::{Layer, Mover},
    player::{DodgeDir, Player, PlayerConfig},
    room::{FALL_Y, Spikes},
};

/// The combat rules' numbers, shared by every fighter.
#[derive(Resource, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CombatRules(pub CombatConfig);

impl ConfigFile for CombatRules {
    const FILE: &'static str = "combat.ron";
}

/// World-side physical numbers of combat.
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PhysicsFeel {
    pub kick_reach: f32,
    pub kick_radius: f32,
    pub kick_force: f32,
    /// Push given to a target by a clean hit.
    pub light_push: f32,
    pub charged_push: f32,
    pub guard_break_push: f32,
    /// Push given to the attacker when parried.
    pub parry_push: f32,
    pub spike_damage: f32,
    pub spike_cooldown: f32,
    /// Bounce off the spikes (m/s).
    pub spike_bounce: f32,
    /// Blade sweep radius.
    pub blade_radius: f32,
    /// Sample points along the blade.
    pub blade_samples: usize,
}

impl Default for PhysicsFeel {
    fn default() -> Self {
        Self {
            kick_reach: 1.5,
            kick_radius: 0.35,
            kick_force: 11.0,
            light_push: 1.8,
            charged_push: 5.0,
            guard_break_push: 3.0,
            parry_push: 3.5,
            spike_damage: 55.0,
            spike_cooldown: 0.8,
            spike_bounce: 4.0,
            blade_radius: 0.07,
            blade_samples: 10,
        }
    }
}

impl ConfigFile for PhysicsFeel {
    const FILE: &'static str = "physics.ron";
}

#[derive(Component)]
pub struct Combatant {
    pub fighter: Fighter,
    pub spike_cooldown: f32,
}

impl Combatant {
    pub fn new(cfg: &CombatConfig) -> Self {
        Self { fighter: Fighter::new(cfg), spike_cooldown: 0.0 }
    }
}

/// A blade segment along a local axis of this entity, owned by a fighter.
#[derive(Component)]
pub struct Blade {
    pub owner: Entity,
    pub axis: Vec3,
    /// Distance along `axis` where the cutting edge starts and ends.
    pub from: f32,
    pub to: f32,
    pub prev: Option<(Vec3, Vec3)>,
}

/// Something happened in combat. `source` acted on `target`.
#[derive(Message, Clone, Debug)]
pub struct CombatMsg {
    pub source: Entity,
    pub target: Entity,
    pub event: Event,
    /// Strike that caused it, if any.
    pub strike: Option<Strike>,
    pub point: Vec3,
    /// From source toward target.
    pub dir: Vec3,
}

/// Hazards: spikes and falls.
#[derive(Message, Clone, Debug)]
pub struct HazardMsg {
    pub target: Entity,
    pub spikes: bool,
    pub point: Vec3,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CombatSet;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ConfigPlugin::<CombatRules>::default(), ConfigPlugin::<PhysicsFeel>::default()))
            .add_message::<CombatMsg>()
            .add_message::<HazardMsg>()
            .add_systems(
                Update,
                (player_intents, tick_fighters, sweep_blades, kicks, hazards, react)
                    .chain()
                    .in_set(CombatSet)
                    .after(crate::mover::MoveSet),
            );
    }
}

fn player_intents(
    time: Res<Time>,
    rules: Res<CombatRules>,
    mut buffer: Local<Option<f32>>,
    player: Single<(&ActionState<Action>, &mut Combatant), With<Player>>,
) {
    let (a, mut c) = player.into_inner();
    let cfg = &rules.0;
    let f = &mut c.fighter;
    if a.just_pressed(&Action::Attack) {
        if f.phase() == Phase::Blocking {
            f.act(Intent::BlockReleased, cfg);
        }
        if f.phase() == Phase::Idle {
            f.act(Intent::AttackPressed, cfg);
        } else {
            *buffer = Some(0.3);
        }
    }
    if let Some(left) = buffer.as_mut() {
        *left -= time.delta_secs();
        if f.phase() == Phase::Idle {
            f.act(Intent::AttackPressed, cfg);
            if !a.pressed(&Action::Attack) {
                f.act(Intent::AttackReleased, cfg);
            }
            *buffer = None;
        } else if *left <= 0.0 {
            *buffer = None;
        }
    }
    if a.just_released(&Action::Attack) {
        f.act(Intent::AttackReleased, cfg);
    }
    if a.just_pressed(&Action::Block) || (a.pressed(&Action::Block) && f.phase() == Phase::Idle && buffer.is_none()) {
        f.act(Intent::BlockPressed, cfg);
    }
    if a.just_released(&Action::Block) {
        f.act(Intent::BlockReleased, cfg);
    }
    if a.just_pressed(&Action::Dodge) {
        f.act(Intent::Dodge, cfg);
    }
    if a.just_pressed(&Action::Kick) {
        f.act(Intent::Kick, cfg);
    }
}

fn tick_fighters(
    time: Res<Time>,
    rules: Res<CombatRules>,
    mut q: Query<(Entity, &mut Combatant, &GlobalTransform)>,
    mut out: MessageWriter<CombatMsg>,
) {
    let dt = time.delta_secs();
    for (e, mut c, tf) in &mut q {
        c.spike_cooldown -= dt;
        for event in c.fighter.tick(dt, &rules.0) {
            let strike = match event {
                Event::StrikeActive(s) => Some(s),
                _ => None,
            };
            out.write(CombatMsg { source: e, target: e, event, strike, point: tf.translation(), dir: tf.forward().into() });
        }
    }
}

fn hit_filter(owner: Entity) -> SpatialQueryFilter {
    SpatialQueryFilter::from_mask(Layer::Character).with_excluded_entities([owner])
}

/// Resolves `source` acting on `target` and publishes the outcome.
fn resolve(
    fighters: &mut Query<(Entity, &mut Combatant, &GlobalTransform)>,
    rules: &CombatConfig,
    source: Entity,
    target: Entity,
    point: Vec3,
    out: &mut MessageWriter<CombatMsg>,
) {
    let Ok([(_, mut a, a_tf), (_, mut d, d_tf)]) = fighters.get_many_mut([source, target]) else {
        return;
    };
    let strike = a.fighter.striking();
    let dir = (d_tf.translation() - a_tf.translation()).with_y(0.0).normalize_or(Vec3::Z);
    for event in rules::exchange(&mut a.fighter, &mut d.fighter, rules) {
        out.write(CombatMsg { source, target, event, strike, point, dir });
    }
}

fn sweep_blades(
    rules: Res<CombatRules>,
    feel: Res<PhysicsFeel>,
    spatial: SpatialQuery,
    mut blades: Query<(&mut Blade, &GlobalTransform)>,
    mut fighters: Query<(Entity, &mut Combatant, &GlobalTransform)>,
    mut out: MessageWriter<CombatMsg>,
) {
    let ball = Collider::sphere(feel.blade_radius);
    for (mut blade, tf) in &mut blades {
        let a = tf.transform_point(blade.axis * blade.from);
        let b = tf.transform_point(blade.axis * blade.to);
        let prev = blade.prev.replace((a, b));
        let Some((pa, pb)) = prev else { continue };
        let owner = blade.owner;
        let striking = fighters
            .get(owner)
            .ok()
            .and_then(|(_, c, _)| c.fighter.striking())
            .filter(|s| *s != Strike::Kick);
        if striking.is_none() {
            continue;
        }
        // Sweep each sample point from where it was to where it is: the
        // swept volume between frames, so fast swings can't tunnel.
        let n = feel.blade_samples.max(2);
        let mut hit: Option<(Entity, Vec3)> = None;
        for i in 0..n {
            let k = i as f32 / (n - 1) as f32;
            let from = pa.lerp(pb, k);
            let to = a.lerp(b, k);
            let delta = to - from;
            let Ok(dir) = Dir3::new(delta) else { continue };
            let cfg = ShapeCastConfig::from_max_distance(delta.length());
            if let Some(h) = spatial.cast_shape(&ball, from, Quat::IDENTITY, dir, &cfg, &hit_filter(owner))
                && fighters.contains(h.entity)
            {
                hit = Some((h.entity, h.point1));
                break;
            }
        }
        if let Some((target, point)) = hit {
            resolve(&mut fighters, &rules.0, owner, target, point, &mut out);
        }
    }
}

fn kicks(
    rules: Res<CombatRules>,
    feel: Res<PhysicsFeel>,
    spatial: SpatialQuery,
    heads: Query<&GlobalTransform, With<crate::player::Head>>,
    players: Query<(), With<Player>>,
    mut fighters: Query<(Entity, &mut Combatant, &GlobalTransform)>,
    mut out: MessageWriter<CombatMsg>,
) {
    let kickers: Vec<(Entity, Vec3, Vec3)> = fighters
        .iter()
        .filter(|(_, c, _)| c.fighter.striking() == Some(Strike::Kick))
        .map(|(e, _, tf)| {
            // The player kicks where they look (flattened); others kick forward.
            let fwd = if players.contains(e) {
                heads.iter().next().map(|h| h.forward().as_vec3()).unwrap_or(tf.forward().into())
            } else {
                tf.forward().into()
            };
            (e, tf.translation(), fwd.with_y(0.0).normalize_or(Vec3::NEG_Z))
        })
        .collect();
    let ball = Collider::sphere(feel.kick_radius);
    for (e, at, fwd) in kickers {
        let cfg = ShapeCastConfig::from_max_distance(feel.kick_reach);
        let origin = at + Vec3::Y * -0.2;
        if let Some(h) = spatial.cast_shape(&ball, origin, Quat::IDENTITY, Dir3::new(fwd).unwrap(), &cfg, &hit_filter(e))
            && fighters.contains(h.entity)
        {
            resolve(&mut fighters, &rules.0, e, h.entity, h.point1, &mut out);
        }
    }
}

fn hazards(
    rules: Res<CombatRules>,
    feel: Res<PhysicsFeel>,
    spatial: SpatialQuery,
    spikes: Query<(), With<Spikes>>,
    spike_tf: Query<&GlobalTransform, With<Spikes>>,
    mut q: Query<(Entity, &mut Combatant, &Transform, &Collider, Option<&mut Mover>)>,
    mut out: MessageWriter<CombatMsg>,
    mut hazard: MessageWriter<HazardMsg>,
) {
    for (e, mut c, tf, col, mover) in &mut q {
        if c.fighter.phase() == Phase::Dead {
            continue;
        }
        let mut events = Vec::new();
        let filter = SpatialQueryFilter::from_mask(Layer::Trigger);
        let touching = spatial
            .shape_intersections(col, tf.translation, tf.rotation, &filter)
            .into_iter()
            .find(|x| spikes.contains(*x));
        if let Some(spike) = touching
            && c.spike_cooldown <= 0.0
        {
            c.spike_cooldown = feel.spike_cooldown;
            if let (Some(mut m), Ok(s)) = (mover, spike_tf.get(spike)) {
                let away = (tf.translation - s.translation()).with_y(0.0).normalize_or_zero();
                m.knockback = away * feel.spike_bounce;
            }
            events = c.fighter.take_damage(feel.spike_damage, &rules.0);
            hazard.write(HazardMsg { target: e, spikes: true, point: tf.translation });
        } else if tf.translation.y < FALL_Y {
            events = c.fighter.take_damage(10_000.0, &rules.0);
            hazard.write(HazardMsg { target: e, spikes: false, point: tf.translation });
        }
        for event in events {
            out.write(CombatMsg { source: e, target: e, event, strike: None, point: tf.translation, dir: Vec3::ZERO });
        }
    }
}

/// Physical consequences: pushes, lunges, dodges.
fn react(
    feel: Res<PhysicsFeel>,
    pcfg: Res<PlayerConfig>,
    mut msgs: MessageReader<CombatMsg>,
    mut movers: Query<(&mut Mover, &GlobalTransform, Option<&DodgeDir>)>,
    heads: Query<&GlobalTransform, With<crate::player::Head>>,
    players: Query<(), With<Player>>,
) {
    for m in msgs.read() {
        let push = |movers: &mut Query<(&mut Mover, &GlobalTransform, Option<&DodgeDir>)>, e: Entity, v: Vec3| {
            if let Ok((mut mv, _, _)) = movers.get_mut(e) {
                mv.knockback += v;
            }
        };
        match (&m.event, m.strike) {
            (Event::Kicked, _) => push(&mut movers, m.target, m.dir * feel.kick_force),
            (Event::Hit { .. }, Some(Strike::Charged(_))) => push(&mut movers, m.target, m.dir * feel.charged_push),
            (Event::Hit { .. }, Some(Strike::Light)) => push(&mut movers, m.target, m.dir * feel.light_push),
            (Event::GuardBroken, _) => push(&mut movers, m.target, m.dir * feel.guard_break_push),
            (Event::Parried, _) => push(&mut movers, m.source, -m.dir * feel.parry_push),
            (Event::StrikeActive(s), _) if players.contains(m.source) => {
                // Lunge into the swing.
                let lunge = match s {
                    Strike::Light => pcfg.light_lunge,
                    Strike::Charged(c) => pcfg.charged_lunge * (0.5 + 0.5 * c),
                    Strike::Kick => 0.0,
                };
                let fwd = heads.iter().next().map(|h| h.forward().as_vec3());
                if let Some(f) = fwd {
                    push(&mut movers, m.source, f.with_y(0.0).normalize_or_zero() * lunge);
                }
            }
            _ => {}
        }
    }
}
