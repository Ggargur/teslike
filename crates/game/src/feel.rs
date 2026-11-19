//! How combat is *felt*: hitstop, slow motion, trauma shake, FOV pulse,
//! target flash, sparks, layered sound and viewmodel recoil. Everything
//! reacts to [`CombatMsg`]; nothing here decides outcomes.

use bevy::{audio::Volume, prelude::*};
use rules::{Event, Phase, Strike};
use serde::{Deserialize, Serialize};

use crate::{
    combat::{CombatMsg, CombatSet, Combatant, HazardMsg},
    config::{ConfigFile, ConfigPlugin},
    player::{Head, Player, PlayerConfig},
    viewmodel::{SwordRig, ViewmodelConfig},
};

/// Per-event feel: how hard each moment lands.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Beat {
    /// Real seconds the world freezes.
    pub hitstop: f32,
    /// Camera trauma added (0..1).
    pub trauma: f32,
    /// FOV kick in degrees (negative = punch in).
    pub fov: f32,
    pub sparks: u32,
    pub volume: f32,
}

impl Default for Beat {
    fn default() -> Self {
        Self { hitstop: 0.0, trauma: 0.0, fov: 0.0, sparks: 0, volume: 1.0 }
    }
}

const fn beat(hitstop: f32, trauma: f32, fov: f32, sparks: u32, volume: f32) -> Beat {
    Beat { hitstop, trauma, fov, sparks, volume }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FeelConfig {
    pub light_hit: Beat,
    pub charged_hit: Beat,
    pub kick: Beat,
    pub blocked: Beat,
    pub parried: Beat,
    pub guard_broken: Beat,
    pub player_hurt: Beat,
    pub spikes: Beat,
    /// Parry slow motion: time scale and real-time duration.
    pub parry_slowmo_scale: f32,
    pub parry_slowmo_secs: f32,
    pub trauma_decay: f32,
    pub shake_max_deg: f32,
    pub shake_max_offset: f32,
    pub shake_frequency: f32,
    pub fov_stiffness: f32,
    pub fov_damping: f32,
    pub flash_secs: f32,
    pub spark_speed: f32,
    pub spark_life: f32,
    pub spark_size: f32,
    pub pitch_jitter: f32,
    pub swing_volume: f32,
    pub footstep_volume: f32,
    pub master_volume: f32,
}

impl Default for FeelConfig {
    fn default() -> Self {
        Self {
            light_hit: beat(0.06, 0.35, -2.0, 8, 1.0),
            charged_hit: beat(0.13, 0.7, -5.0, 18, 1.0),
            kick: beat(0.07, 0.45, -3.0, 4, 1.0),
            blocked: beat(0.045, 0.3, -1.0, 10, 0.9),
            parried: beat(0.09, 0.55, -6.0, 40, 1.0),
            guard_broken: beat(0.1, 0.65, -4.0, 22, 1.0),
            player_hurt: beat(0.08, 0.75, 4.0, 6, 1.0),
            spikes: beat(0.12, 0.6, -3.0, 20, 1.0),
            parry_slowmo_scale: 0.25,
            parry_slowmo_secs: 0.45,
            trauma_decay: 1.7,
            shake_max_deg: 5.0,
            shake_max_offset: 0.06,
            shake_frequency: 24.0,
            fov_stiffness: 220.0,
            fov_damping: 16.0,
            flash_secs: 0.07,
            spark_speed: 4.5,
            spark_life: 0.35,
            spark_size: 0.035,
            pitch_jitter: 0.08,
            swing_volume: 0.6,
            footstep_volume: 0.25,
            master_volume: 0.8,
        }
    }
}

impl ConfigFile for FeelConfig {
    const FILE: &'static str = "feel.ron";
}

#[derive(Resource, Default)]
pub struct TimeFx {
    hitstop: f32,
    slowmo: f32,
    slowmo_scale: f32,
}

impl TimeFx {
    /// Ends any hitstop or slow motion at once.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Resource, Default)]
pub struct Trauma(pub f32);

#[derive(Resource, Default)]
struct FovPulse {
    value: f32,
    vel: f32,
}

#[derive(Component)]
struct Spark {
    vel: Vec3,
    life: f32,
}

#[derive(Resource)]
pub struct Sfx {
    pub swing_light: Handle<AudioSource>,
    pub swing_heavy: Handle<AudioSource>,
    pub hit_thud: Handle<AudioSource>,
    pub hit_crunch: Handle<AudioSource>,
    pub hit_wood: Handle<AudioSource>,
    pub block: Handle<AudioSource>,
    pub parry: Handle<AudioSource>,
    pub guard_break: Handle<AudioSource>,
    pub kick: Handle<AudioSource>,
    pub spikes: Handle<AudioSource>,
    pub death: Handle<AudioSource>,
    pub telegraph: Handle<AudioSource>,
    pub charge: Handle<AudioSource>,
    pub dodge: Handle<AudioSource>,
    pub hurt: Handle<AudioSource>,
    pub footstep: Handle<AudioSource>,
}

/// Wooden targets sound like wood when hit.
#[derive(Component)]
pub struct WoodenTarget;

#[derive(Resource)]
struct SparkAssets {
    mesh: Handle<Mesh>,
    hot: Handle<StandardMaterial>,
    blood: Handle<StandardMaterial>,
}

pub struct FeelPlugin;

impl Plugin for FeelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConfigPlugin::<FeelConfig>::default())
            .init_resource::<TimeFx>()
            .init_resource::<Trauma>()
            .init_resource::<FovPulse>()
            .add_systems(Startup, load)
            .add_systems(First, drive_time)
            .add_systems(
                Update,
                (on_combat, on_hazard, charge_sound, footsteps, shake, fov, sparks).chain().after(CombatSet),
            );
    }
}

fn load(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let s = |n: &str| assets.load(format!("sounds/{n}.wav"));
    commands.insert_resource(Sfx {
        swing_light: s("swing_light"),
        swing_heavy: s("swing_heavy"),
        hit_thud: s("hit_thud"),
        hit_crunch: s("hit_crunch"),
        hit_wood: s("hit_wood"),
        block: s("block"),
        parry: s("parry"),
        guard_break: s("guard_break"),
        kick: s("kick"),
        spikes: s("spikes"),
        death: s("death"),
        telegraph: s("telegraph"),
        charge: s("charge"),
        dodge: s("dodge"),
        hurt: s("hurt"),
        footstep: s("footstep"),
    });
    commands.insert_resource(SparkAssets {
        mesh: meshes.add(Cuboid::from_length(1.0)),
        hot: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.4),
            emissive: LinearRgba::rgb(4.0, 2.5, 0.8),
            unlit: true,
            ..default()
        }),
        blood: materials.add(StandardMaterial { base_color: Color::srgb(0.55, 0.03, 0.02), unlit: true, ..default() }),
    });
}

/// Plays a one-shot sound with slight pitch variation.
pub fn play(commands: &mut Commands, cfg: &FeelConfig, h: &Handle<AudioSource>, volume: f32, pitch: f32) {
    let jitter = 1.0 + (rand_f32() * 2.0 - 1.0) * cfg.pitch_jitter;
    commands.spawn((
        AudioPlayer::new(h.clone()),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(volume * cfg.master_volume))
            .with_speed(pitch * jitter),
    ));
}

/// Tiny xorshift so we don't pull in a rand crate.
pub fn rand_f32() -> f32 {
    use std::cell::Cell;
    thread_local!(static S: Cell<u32> = const { Cell::new(0x9E37_79B9) });
    S.with(|s| {
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        s.set(x);
        (x >> 8) as f32 / (1u32 << 24) as f32
    })
}

fn drive_time(real: Res<Time<Real>>, mut fx: ResMut<TimeFx>, mut virt: ResMut<Time<Virtual>>) {
    let dt = real.delta_secs();
    fx.hitstop = (fx.hitstop - dt).max(0.0);
    fx.slowmo = (fx.slowmo - dt).max(0.0);
    let speed = if fx.hitstop > 0.0 {
        0.0
    } else if fx.slowmo > 0.0 {
        // Ease back to full speed over the last third.
        let k = (fx.slowmo / 0.15).min(1.0);
        1.0 + (fx.slowmo_scale - 1.0) * k
    } else {
        1.0
    };
    virt.set_relative_speed(speed);
}

fn on_combat(
    mut commands: Commands,
    cfg: Res<FeelConfig>,
    sfx: Res<Sfx>,
    vm: Res<ViewmodelConfig>,
    sparks: Res<SparkAssets>,
    mut msgs: MessageReader<CombatMsg>,
    mut fx: ResMut<TimeFx>,
    mut trauma: ResMut<Trauma>,
    mut pulse: ResMut<FovPulse>,
    player: Single<Entity, With<Player>>,
    wooden: Query<(), With<WoodenTarget>>,
    mut rig: Single<&mut SwordRig>,
) {
    let me = *player;
    for m in msgs.read() {
        let mine = m.source == me;
        let at_me = m.target == me && m.source != me;
        let involved = mine || at_me;
        let beat = match (&m.event, m.strike) {
            (Event::StrikeActive(s), _) => {
                let (h, v) = match s {
                    Strike::Light => (&sfx.swing_light, cfg.swing_volume),
                    Strike::Charged(_) => (&sfx.swing_heavy, cfg.swing_volume * 1.2),
                    Strike::Kick => (&sfx.dodge, cfg.swing_volume),
                };
                play(&mut commands, &cfg, h, v * if mine { 1.0 } else { 0.7 }, 1.0);
                None
            }
            (Event::Hit { .. }, s) if at_me => {
                play(&mut commands, &cfg, &sfx.hurt, 1.0, 1.0);
                play(&mut commands, &cfg, &sfx.hit_thud, 0.8, 0.8);
                spawn_sparks(&mut commands, &sparks, &cfg, m.point, -m.dir, cfg.player_hurt.sparks, false);
                let _ = s;
                Some(cfg.player_hurt)
            }
            (Event::Hit { .. }, s) => {
                let b = match s {
                    Some(Strike::Charged(_)) => cfg.charged_hit,
                    Some(Strike::Kick) => cfg.kick,
                    _ => cfg.light_hit,
                };
                let heavy = matches!(s, Some(Strike::Charged(_)));
                if wooden.contains(m.target) {
                    play(&mut commands, &cfg, &sfx.hit_wood, b.volume, if heavy { 0.8 } else { 1.0 });
                } else {
                    // Layered: low body thud + crunchy mid.
                    play(&mut commands, &cfg, &sfx.hit_thud, b.volume, if heavy { 0.75 } else { 1.0 });
                    play(&mut commands, &cfg, &sfx.hit_crunch, b.volume * 0.8, if heavy { 0.85 } else { 1.05 });
                }
                if s != Some(Strike::Kick) {
                    spawn_sparks(&mut commands, &sparks, &cfg, m.point, -m.dir, b.sparks, wooden.contains(m.target));
                }
                commands.entity(m.target).try_insert(psx::PsxFlash { remaining: cfg.flash_secs, color: Color::WHITE });
                if mine {
                    rig.impulse(Vec3::from_array(vm.hit_recoil_pos), Vec3::from_array(vm.hit_recoil_rot) * if heavy { 1.5 } else { 1.0 });
                }
                Some(b)
            }
            (Event::Kicked, _) => {
                play(&mut commands, &cfg, &sfx.kick, cfg.kick.volume, 1.0);
                Some(cfg.kick)
            }
            (Event::Blocked, _) => {
                play(&mut commands, &cfg, &sfx.block, cfg.blocked.volume, 1.0);
                spawn_sparks(&mut commands, &sparks, &cfg, m.point, -m.dir, cfg.blocked.sparks, false);
                if involved {
                    rig.impulse(Vec3::from_array(vm.block_recoil_pos), Vec3::from_array(vm.block_recoil_rot));
                }
                Some(cfg.blocked)
            }
            (Event::Parried, _) => {
                play(&mut commands, &cfg, &sfx.parry, cfg.parried.volume, 1.0);
                play(&mut commands, &cfg, &sfx.block, 0.7, 0.8);
                spawn_sparks(&mut commands, &sparks, &cfg, m.point, -m.dir, cfg.parried.sparks, false);
                fx.slowmo = cfg.parry_slowmo_secs;
                fx.slowmo_scale = cfg.parry_slowmo_scale;
                if involved {
                    rig.impulse(Vec3::ZERO, Vec3::from_array(vm.parry_recoil_rot));
                }
                Some(cfg.parried)
            }
            (Event::GuardBroken, _) => {
                play(&mut commands, &cfg, &sfx.guard_break, cfg.guard_broken.volume, 1.0);
                spawn_sparks(&mut commands, &sparks, &cfg, m.point, -m.dir, cfg.guard_broken.sparks, false);
                if at_me {
                    rig.impulse(Vec3::from_array(vm.block_recoil_pos) * 2.0, Vec3::from_array(vm.block_recoil_rot) * 2.5);
                }
                Some(cfg.guard_broken)
            }
            (Event::Died, _) => {
                play(&mut commands, &cfg, &sfx.death, 1.0, 1.0);
                None
            }
        };
        if let Some(b) = beat {
            // Only moments the player takes part in freeze the world.
            if involved {
                fx.hitstop = fx.hitstop.max(b.hitstop);
                trauma.0 = (trauma.0 + b.trauma).min(1.0);
                pulse.vel += b.fov * 30.0;
            } else {
                trauma.0 = (trauma.0 + b.trauma * 0.3).min(1.0);
            }
        }
    }
}

fn on_hazard(
    mut commands: Commands,
    cfg: Res<FeelConfig>,
    sfx: Res<Sfx>,
    sparks: Res<SparkAssets>,
    mut msgs: MessageReader<HazardMsg>,
    mut fx: ResMut<TimeFx>,
    mut trauma: ResMut<Trauma>,
) {
    for m in msgs.read() {
        if m.spikes {
            play(&mut commands, &cfg, &sfx.spikes, cfg.spikes.volume, 1.0);
            spawn_sparks(&mut commands, &sparks, &cfg, m.point, Vec3::X, cfg.spikes.sparks, true);
            fx.hitstop = fx.hitstop.max(cfg.spikes.hitstop);
            trauma.0 = (trauma.0 + cfg.spikes.trauma).min(1.0);
            commands.entity(m.target).try_insert(psx::PsxFlash { remaining: cfg.flash_secs * 2.0, color: Color::srgb(1.0, 0.2, 0.1) });
        }
    }
}

fn spawn_sparks(commands: &mut Commands, a: &SparkAssets, cfg: &FeelConfig, at: Vec3, normal: Vec3, n: u32, blood: bool) {
    for _ in 0..n {
        let r = Vec3::new(rand_f32() - 0.5, rand_f32() - 0.3, rand_f32() - 0.5) * 2.0;
        let vel = (normal.normalize_or(Vec3::Y) * 0.6 + r).normalize_or(Vec3::Y) * cfg.spark_speed * (0.4 + rand_f32());
        let size = cfg.spark_size * (0.6 + rand_f32() * 0.8) * if blood { 1.4 } else { 1.0 };
        commands.spawn((
            Spark { vel, life: cfg.spark_life * (0.5 + rand_f32()) },
            Mesh3d(a.mesh.clone()),
            MeshMaterial3d(if blood { a.blood.clone() } else { a.hot.clone() }),
            Transform::from_translation(at).with_scale(Vec3::splat(size)),
        ));
    }
}

fn sparks(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Spark, &mut Transform)>) {
    let dt = time.delta_secs();
    for (e, mut s, mut tf) in &mut q {
        s.life -= dt;
        if s.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        s.vel.y -= 14.0 * dt;
        tf.translation += s.vel * dt;
        tf.scale *= (-3.0 * dt).exp();
    }
}

fn charge_sound(
    mut commands: Commands,
    cfg: Res<FeelConfig>,
    sfx: Res<Sfx>,
    player: Single<&Combatant, With<Player>>,
    mut was: Local<bool>,
) {
    let charging = player.fighter.phase() == Phase::Charging && player.fighter.charge() > 0.0;
    if charging && !*was {
        play(&mut commands, &cfg, &sfx.charge, 0.6, 1.0);
    }
    *was = charging;
}

fn footsteps(
    mut commands: Commands,
    cfg: Res<FeelConfig>,
    sfx: Res<Sfx>,
    head: Single<&Head>,
    mut last: Local<i32>,
) {
    let step = (head.bob_phase / std::f32::consts::PI).floor() as i32;
    if step != *last {
        *last = step;
        if head.bob_weight > 0.3 {
            play(&mut commands, &cfg, &sfx.footstep, cfg.footstep_volume * head.bob_weight.min(1.0), 1.0);
        }
    }
}

fn shake(real: Res<Time<Real>>, cfg: Res<FeelConfig>, mut trauma: ResMut<Trauma>, mut head: Single<&mut Head>) {
    let dt = real.delta_secs();
    trauma.0 = (trauma.0 - cfg.trauma_decay * dt).max(0.0);
    let s = trauma.0 * trauma.0;
    let t = real.elapsed_secs() * cfg.shake_frequency;
    // Sum of incommensurate sines: smooth, non-repeating enough.
    let n = |a: f32, b: f32| ((t * a).sin() + (t * b + 1.3).sin() * 0.5) / 1.5;
    let max = cfg.shake_max_deg.to_radians();
    head.shake_rot = Vec3::new(n(1.0, 2.3), n(1.7, 0.6), n(0.8, 3.1)) * max * s;
    head.shake_offset = Vec3::new(n(1.3, 2.9), n(2.1, 0.7), 0.0) * cfg.shake_max_offset * s;
}

fn fov(
    real: Res<Time<Real>>,
    cfg: Res<FeelConfig>,
    pcfg: Res<PlayerConfig>,
    mut pulse: ResMut<FovPulse>,
    mut proj: Single<&mut Projection, With<Head>>,
) {
    let dt = real.delta_secs().min(1.0 / 30.0);
    let acc = -pulse.value * cfg.fov_stiffness - pulse.vel * cfg.fov_damping;
    pulse.vel += acc * dt;
    pulse.value += pulse.vel * dt;
    if let Projection::Perspective(p) = proj.as_mut() {
        p.fov = (pcfg.fov_deg + pulse.value).clamp(30.0, 120.0).to_radians();
    }
}
