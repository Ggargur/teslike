//! Procedural first-person sword: key poses per combat phase, followed by
//! springs, plus look sway and recoil impulses from game feel.

use bevy::{camera::visibility::RenderLayers, prelude::*};
use rules::{Phase, Strike};
use serde::{Deserialize, Serialize};

use crate::{
    combat::{Blade, CombatRules, Combatant},
    config::{ConfigFile, ConfigPlugin},
    player::{Head, Player, VIEWMODEL_LAYER},
};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Pose {
    /// Camera-space position of the grip (meters).
    pub pos: [f32; 3],
    /// Euler XYZ in degrees. Blade points +Y at zero.
    pub rot: [f32; 3],
}

impl Pose {
    const fn new(pos: [f32; 3], rot: [f32; 3]) -> Self {
        Self { pos, rot }
    }
    fn p(&self) -> Vec3 {
        Vec3::from_array(self.pos)
    }
    fn r(&self) -> Vec3 {
        Vec3::from_array(self.rot)
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewmodelConfig {
    pub idle: Pose,
    pub light_windup: Pose,
    pub light_end: Pose,
    pub charge: Pose,
    pub charged_end: Pose,
    pub block: Pose,
    pub stunned: Pose,
    pub kick: Pose,
    pub dodge: Pose,
    /// Spring stiffness / damping while following poses.
    pub stiffness: f32,
    pub damping: f32,
    /// Stiffer spring during a swing's active window.
    pub swing_stiffness: f32,
    pub swing_damping: f32,
    /// Extra invisible reach of the blade sweep beyond the visible tip.
    pub reach_extra: f32,
    /// How much the sword lags when turning (meters per radian).
    pub sway_pos: f32,
    /// Degrees per radian of look.
    pub sway_rot: f32,
    pub sway_max: f32,
    /// Trembling while charging (meters at full charge).
    pub charge_tremble: f32,
    /// Overshoot past the swing end (fraction of the arc).
    pub follow_through: f32,
    /// Recoil impulses (velocity added to the spring).
    pub hit_recoil_pos: [f32; 3],
    pub hit_recoil_rot: [f32; 3],
    pub block_recoil_pos: [f32; 3],
    pub block_recoil_rot: [f32; 3],
    pub parry_recoil_rot: [f32; 3],
    pub boot_rest: [f32; 3],
    pub boot_out: [f32; 3],
}

impl Default for ViewmodelConfig {
    fn default() -> Self {
        Self {
            idle: Pose::new([0.30, -0.30, -0.48], [-48.0, 0.0, 14.0]),
            light_windup: Pose::new([0.36, -0.10, -0.42], [-28.0, 10.0, -38.0]),
            light_end: Pose::new([-0.34, -0.42, -0.52], [-100.0, -10.0, 75.0]),
            charge: Pose::new([0.30, -0.16, -0.44], [-24.0, 12.0, -55.0]),
            charged_end: Pose::new([-0.18, -0.58, -0.62], [-125.0, 0.0, 35.0]),
            block: Pose::new([0.04, -0.17, -0.46], [-12.0, 0.0, 88.0]),
            stunned: Pose::new([0.48, -0.62, -0.30], [-25.0, 0.0, -55.0]),
            kick: Pose::new([0.52, -0.18, -0.30], [-30.0, 0.0, -10.0]),
            dodge: Pose::new([0.30, -0.42, -0.40], [-55.0, 0.0, 20.0]),
            stiffness: 260.0,
            damping: 22.0,
            swing_stiffness: 1400.0,
            swing_damping: 60.0,
            reach_extra: 0.55,
            sway_pos: 0.06,
            sway_rot: 18.0,
            sway_max: 0.06,
            charge_tremble: 0.006,
            follow_through: 0.12,
            hit_recoil_pos: [0.0, 0.6, 2.2],
            hit_recoil_rot: [260.0, 0.0, -180.0],
            block_recoil_pos: [0.0, -1.2, 1.2],
            block_recoil_rot: [-120.0, 0.0, 60.0],
            parry_recoil_rot: [500.0, 120.0, -300.0],
            boot_rest: [0.12, -1.3, -0.25],
            boot_out: [0.06, -0.55, -0.85],
        }
    }
}

impl ConfigFile for ViewmodelConfig {
    const FILE: &'static str = "viewmodel.ron";
}

/// The posed sword pivot. Spring state lives here.
#[derive(Component, Default)]
pub struct SwordRig {
    pos: Vec3,
    vel: Vec3,
    rot: Vec3,
    rot_vel: Vec3,
}

impl SwordRig {
    /// Adds an impulse (e.g. recoil) to the spring.
    pub fn impulse(&mut self, pos: Vec3, rot: Vec3) {
        self.vel += pos;
        self.rot_vel += rot;
    }
}

#[derive(Component, Default)]
struct Sway {
    offset: Vec3,
    rot: Vec3,
    last: Option<(f32, f32)>,
}

#[derive(Component)]
struct Boot;

pub struct ViewmodelPlugin;

impl Plugin for ViewmodelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConfigPlugin::<ViewmodelConfig>::default())
            .add_systems(PostStartup, spawn_viewmodel)
            .add_systems(Update, (animate_sword, sway, animate_boot, update_reach).after(crate::combat::CombatSet));
    }
}

fn spawn_viewmodel(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cfg: Res<ViewmodelConfig>,
    head: Single<Entity, With<Head>>,
    player: Single<Entity, With<Player>>,
) {
    let glove = materials.add(StandardMaterial { base_color: Color::srgb(0.32, 0.2, 0.12), perceptual_roughness: 0.9, ..default() });
    let sleeve = materials.add(StandardMaterial { base_color: Color::srgb(0.36, 0.27, 0.2), perceptual_roughness: 0.9, ..default() });
    let layer = RenderLayers::layer(VIEWMODEL_LAYER);
    let root = commands
        .spawn((Name::new("Viewmodel"), Sway::default(), Transform::default(), Visibility::default(), bevy::app::Propagate(layer.clone())))
        .id();
    commands.entity(*head).add_child(root);
    let rig = commands
        .spawn((
            Name::new("Sword rig"),
            SwordRig { pos: cfg.idle.p(), rot: cfg.idle.r(), ..default() },
            Blade { owner: *player, axis: Vec3::Y, from: 0.15, to: 0.85 + cfg.reach_extra, prev: None },
            Transform::default(),
            Visibility::default(),
            ChildOf(root),
        ))
        .id();
    commands.spawn((
        Name::new("Sword"),
        WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset("models/sword.glb"))),
        Transform::default(),
        ChildOf(rig),
    ));
    // Fist around the grip and a forearm reaching out of view.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.075, 0.1, 0.09))),
        MeshMaterial3d(glove),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(rig),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.06, 0.3, 0.06))),
        MeshMaterial3d(sleeve.clone()),
        Transform::from_xyz(0.03, -0.05, 0.16).with_rotation(Quat::from_rotation_x(1.25)),
        ChildOf(rig),
    ));
    commands.spawn((
        Name::new("Boot"),
        Boot,
        Mesh3d(meshes.add(Cuboid::new(0.16, 0.14, 0.34))),
        MeshMaterial3d(sleeve),
        Transform::from_translation(Vec3::from_array(cfg.boot_rest)),
        ChildOf(root),
    ));
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn update_reach(cfg: Res<ViewmodelConfig>, mut q: Query<&mut Blade, With<SwordRig>>) {
    if cfg.is_changed() {
        for mut b in &mut q {
            b.to = 0.85 + cfg.reach_extra;
        }
    }
}

fn animate_sword(
    time: Res<Time>,
    cfg: Res<ViewmodelConfig>,
    rules: Res<CombatRules>,
    player: Single<&Combatant, With<Player>>,
    rig: Single<(&mut SwordRig, &mut Transform)>,
) {
    let dt = time.delta_secs().min(1.0 / 30.0);
    if dt <= 0.0 {
        return;
    }
    let f = &player.fighter;
    let t = f.phase_time();
    let (mut rig, mut tf) = rig.into_inner();
    let lerp = |a: &Pose, b: &Pose, k: f32| (a.p().lerp(b.p(), k), a.r().lerp(b.r(), k));
    let mut swing = false;
    let (target_p, target_r) = match f.phase() {
        Phase::Idle | Phase::Staggered => (cfg.idle.p(), cfg.idle.r()),
        Phase::Charging => {
            let c = f.charge();
            let (p, r) = lerp(&cfg.light_windup, &cfg.charge, smooth(t / rules.0.charged.tap_threshold.max(0.01)));
            let tremble = Vec3::new((t * 61.0).sin(), (t * 47.0).cos(), 0.0) * cfg.charge_tremble * c;
            (p + tremble, r)
        }
        Phase::Windup(Strike::Light) => (cfg.light_windup.p(), cfg.light_windup.r()),
        Phase::Windup(Strike::Charged(_)) => (cfg.charge.p(), cfg.charge.r()),
        Phase::Active(Strike::Light) => {
            swing = true;
            let k = smooth(t / rules.0.light.active) * (1.0 + cfg.follow_through);
            lerp(&cfg.light_windup, &cfg.light_end, k)
        }
        Phase::Active(Strike::Charged(_)) => {
            swing = true;
            let k = smooth(t / rules.0.charged.strike.active) * (1.0 + cfg.follow_through);
            lerp(&cfg.charge, &cfg.charged_end, k)
        }
        Phase::Recovery(Strike::Light) => lerp(&cfg.light_end, &cfg.idle, smooth(t / rules.0.light.recovery)),
        Phase::Recovery(Strike::Charged(_)) => {
            lerp(&cfg.charged_end, &cfg.idle, smooth(t / rules.0.charged.strike.recovery))
        }
        Phase::Windup(Strike::Kick) | Phase::Active(Strike::Kick) | Phase::Recovery(Strike::Kick) => {
            (cfg.kick.p(), cfg.kick.r())
        }
        Phase::Blocking => (cfg.block.p(), cfg.block.r()),
        Phase::Dodging => (cfg.dodge.p(), cfg.dodge.r()),
        Phase::Stunned | Phase::Unbalanced => {
            let wobble = Vec3::new((t * 23.0).sin(), 0.0, (t * 17.0).cos()) * 6.0;
            (cfg.stunned.p(), cfg.stunned.r() + wobble)
        }
        Phase::Dead => (cfg.stunned.p() + Vec3::Y * -0.6, cfg.stunned.r()),
    };
    let (k, d) = if swing { (cfg.swing_stiffness, cfg.swing_damping) } else { (cfg.stiffness, cfg.damping) };
    // Semi-implicit spring: underdamped by default, so it overshoots a bit.
    let acc = (target_p - rig.pos) * k - rig.vel * d;
    rig.vel += acc * dt;
    let v = rig.vel;
    rig.pos += v * dt;
    let racc = (target_r - rig.rot) * k - rig.rot_vel * d;
    rig.rot_vel += racc * dt;
    let rv = rig.rot_vel;
    rig.rot += rv * dt;
    tf.translation = rig.pos;
    let r = rig.rot * (std::f32::consts::PI / 180.0);
    tf.rotation = Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z);
}

fn sway(
    time: Res<Time<Real>>,
    cfg: Res<ViewmodelConfig>,
    head: Single<&Head>,
    root: Single<(&mut Sway, &mut Transform)>,
) {
    let dt = time.delta_secs().min(1.0 / 30.0);
    let (mut s, mut tf) = root.into_inner();
    let (yaw, pitch) = (head.yaw, head.pitch);
    let (dyaw, dpitch) = s.last.map(|(y, p)| (yaw - y, pitch - p)).unwrap_or((0.0, 0.0));
    s.last = Some((yaw, pitch));
    // Lag behind the look, then settle back.
    let max = cfg.sway_max;
    s.offset += Vec3::new(dyaw * cfg.sway_pos, -dpitch * cfg.sway_pos, 0.0);
    s.offset = s.offset.clamp(Vec3::splat(-max), Vec3::splat(max));
    s.offset *= (-10.0 * dt).exp();
    s.rot += Vec3::new(dpitch, dyaw, dyaw * 0.5) * cfg.sway_rot.to_radians();
    s.rot = s.rot.clamp(Vec3::splat(-0.25), Vec3::splat(0.25));
    s.rot *= (-10.0 * dt).exp();
    tf.translation = s.offset;
    tf.rotation = Quat::from_euler(EulerRot::XYZ, s.rot.x, s.rot.y, s.rot.z);
}

fn animate_boot(
    cfg: Res<ViewmodelConfig>,
    rules: Res<CombatRules>,
    player: Single<&Combatant, With<Player>>,
    mut boot: Single<&mut Transform, With<Boot>>,
) {
    let f = &player.fighter;
    let t = f.phase_time();
    let k = &rules.0.kick;
    let out = match f.phase() {
        Phase::Windup(Strike::Kick) => smooth(t / k.windup) * 0.35,
        Phase::Active(Strike::Kick) => 0.35 + 0.65 * smooth(t / k.active),
        Phase::Recovery(Strike::Kick) => 1.0 - smooth(t / k.recovery),
        _ => 0.0,
    };
    let rest = Vec3::from_array(cfg.boot_rest);
    boot.translation = rest.lerp(Vec3::from_array(cfg.boot_out), out);
    boot.rotation = Quat::from_rotation_x(-0.6 * out);
}
