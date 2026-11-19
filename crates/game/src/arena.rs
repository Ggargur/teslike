//! The fight loop: one opponent at a time, instant restart on death or
//! victory, a fight counter and the HUD.

use bevy::{camera::visibility::RenderLayers, prelude::*};
use rules::{Event, Phase};
use serde::{Deserialize, Serialize};

use crate::{
    combat::{CombatMsg, CombatRules, Combatant},
    feel::TimeFx,
    config::{ConfigFile, ConfigPlugin},
    enemy::{Enemy, Kind, spawn_enemy},
    mover::Mover,
    player::{Head, Player, PlayerConfig, spawn_point},
};

#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ArenaConfig {
    /// Seconds between the end of a fight and the next one.
    pub restart_delay: f32,
    pub enemy_spawn: [f32; 3],
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self { restart_delay: 1.4, enemy_spawn: [2.0, 0.0, -3.0] }
    }
}

impl ConfigFile for ArenaConfig {
    const FILE: &'static str = "arena.ron";
}

#[derive(Resource)]
pub struct Arena {
    pub kind: Kind,
    pub fights: u32,
    pub wins: u32,
    pub losses: u32,
    pub streak: u32,
    /// Counting down to the next fight.
    pub restart: Option<f32>,
}

#[derive(Component)]
struct HudText;
#[derive(Component)]
struct HealthBar;
#[derive(Component)]
struct StaminaBar;
#[derive(Component)]
struct EnemyBar;
#[derive(Component)]
struct HelpText;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        let kind = match std::env::var("TESLIKE_OPPONENT").as_deref() {
            Ok("dummy") => Kind::Dummy,
            _ => Kind::Humanoid,
        };
        app.add_plugins(ConfigPlugin::<ArenaConfig>::default())
            .insert_resource(Arena { kind, fights: 1, wins: 0, losses: 0, streak: 0, restart: None })
            .add_systems(Startup, (spawn_hud, first_fight))
            .add_systems(
                Update,
                (score, countdown, switch_opponent, hud).chain().after(crate::combat::CombatSet),
            );
    }
}

fn first_fight(mut commands: Commands, assets: Res<AssetServer>, rules: Res<CombatRules>, arena: Res<Arena>, cfg: Res<ArenaConfig>) {
    spawn_enemy(&mut commands, &assets, &rules, arena.kind, Vec3::from_array(cfg.enemy_spawn));
}

fn score(
    mut msgs: MessageReader<CombatMsg>,
    mut arena: ResMut<Arena>,
    cfg: Res<ArenaConfig>,
    player: Single<Entity, With<Player>>,
    enemies: Query<(), With<Enemy>>,
) {
    for m in msgs.read() {
        if m.event != Event::Died || arena.restart.is_some() {
            continue;
        }
        if m.target == *player {
            arena.losses += 1;
            arena.streak = 0;
        } else if enemies.contains(m.target) {
            arena.wins += 1;
            arena.streak += 1;
        } else {
            continue;
        }
        arena.restart = Some(cfg.restart_delay);
    }
}

fn countdown(
    mut commands: Commands,
    time: Res<Time<Real>>,
    assets: Res<AssetServer>,
    rules: Res<CombatRules>,
    pcfg: Res<PlayerConfig>,
    cfg: Res<ArenaConfig>,
    mut arena: ResMut<Arena>,
    enemies: Query<Entity, With<Enemy>>,
    player: Single<(&mut Combatant, &mut Transform, &mut Mover), With<Player>>,
    mut head: Single<&mut Head>,
    mut fx: ResMut<TimeFx>,
) {
    let Some(left) = arena.restart.as_mut() else { return };
    *left -= time.delta_secs();
    if *left > 0.0 {
        return;
    }
    arena.restart = None;
    arena.fights += 1;
    reset(&mut commands, &assets, &rules, &pcfg, &cfg, arena.kind, &enemies, player, &mut head);
    fx.clear();
}

fn reset(
    commands: &mut Commands,
    assets: &AssetServer,
    rules: &CombatRules,
    pcfg: &PlayerConfig,
    cfg: &ArenaConfig,
    kind: Kind,
    enemies: &Query<Entity, With<Enemy>>,
    player: Single<(&mut Combatant, &mut Transform, &mut Mover), With<Player>>,
    head: &mut Head,
) {
    for e in enemies {
        commands.entity(e).despawn();
    }
    spawn_enemy(commands, assets, rules, kind, Vec3::from_array(cfg.enemy_spawn));
    let (mut c, mut tf, mut mv) = player.into_inner();
    *c = Combatant::new(&rules.0);
    tf.translation = spawn_point().translation + Vec3::Y * (pcfg.height / 2.0 + 0.05);
    mv.velocity = Vec3::ZERO;
    mv.knockback = Vec3::ZERO;
    head.reset(-std::f32::consts::FRAC_PI_4);
}

fn switch_opponent(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    assets: Res<AssetServer>,
    rules: Res<CombatRules>,
    pcfg: Res<PlayerConfig>,
    cfg: Res<ArenaConfig>,
    mut arena: ResMut<Arena>,
    enemies: Query<Entity, With<Enemy>>,
    player: Single<(&mut Combatant, &mut Transform, &mut Mover), With<Player>>,
    mut head: Single<&mut Head>,
    mut help: Single<&mut Visibility, With<HelpText>>,
    mut fx: ResMut<TimeFx>,
) {
    if keys.just_pressed(KeyCode::KeyH) {
        help.toggle_visible_hidden();
    }
    let kind = if keys.just_pressed(KeyCode::F2) {
        Kind::Dummy
    } else if keys.just_pressed(KeyCode::F3) {
        Kind::Humanoid
    } else if keys.just_pressed(KeyCode::KeyR) {
        arena.kind
    } else {
        return;
    };
    arena.kind = kind;
    arena.restart = None;
    arena.fights += 1;
    reset(&mut commands, &assets, &rules, &pcfg, &cfg, kind, &enemies, player, &mut head);
    fx.clear();
}

fn bar(parent: &mut ChildSpawnerCommands, color: Color, marker: impl Component, width: f32) {
    parent
        .spawn((Node { width: px(width), height: px(10.0), border: UiRect::all(px(1.0)), margin: UiRect::bottom(px(4.0)), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)), BorderColor::all(Color::srgb(0.3, 0.28, 0.25))))
        .with_children(|b| {
            b.spawn((marker, Node { width: percent(100.0), height: percent(100.0), ..default() }, BackgroundColor(color)));
        });
}

fn spawn_hud(mut commands: Commands, assets: Res<AssetServer>) {
    let face: Handle<Font> = assets.load("fonts/DejaVuSansMono-Bold.ttf");
    // UI draws above the PSX output (order 100).
    commands.spawn((Camera2d, Camera { order: 101, clear_color: ClearColorConfig::None, ..default() }, IsDefaultUiCamera, Msaa::Off, RenderLayers::layer(30)));
    let font = TextFont { font: face.clone().into(), font_size: FontSize::Px(18.0), ..default() };
    commands
        .spawn(Node { width: percent(100.0), height: percent(100.0), padding: UiRect::all(px(16.0)), flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, ..default() })
        .with_children(|root| {
            root.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() }).with_children(|top| {
                top.spawn((HudText, Text::new(""), font.clone(), TextColor(Color::srgb(0.95, 0.9, 0.8))));
                bar(top, Color::srgb(0.75, 0.15, 0.1), EnemyBar, 260.0);
            });
            root.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|bottom| {
                bottom.spawn((
                    HelpText,
                    Text::new("WASD andar · Shift correr · Mouse olhar · LMB atacar (segure p/ carregar) · RMB bloquear (no instante = Aparo) · Espaço esquiva · F chute\nF2 boneco · F3 humanoide · R reiniciar · F1 inspector · H esconder ajuda"),
                    TextFont { font: face.into(), font_size: FontSize::Px(13.0), ..default() },
                    TextColor(Color::srgba(0.9, 0.85, 0.75, 0.8)),
                    Node { margin: UiRect::bottom(px(8.0)), ..default() },
                ));
                bar(bottom, Color::srgb(0.7, 0.1, 0.08), HealthBar, 220.0);
                bar(bottom, Color::srgb(0.75, 0.7, 0.2), StaminaBar, 220.0);
            });
        });
    // Crosshair dot.
    commands.spawn((
        Node { position_type: PositionType::Absolute, left: percent(50.0), top: percent(50.0), width: px(3.0), height: px(3.0), margin: UiRect { left: px(-1.5), top: px(-1.5), ..default() }, ..default() },
        BackgroundColor(Color::srgba(1.0, 0.95, 0.85, 0.6)),
    ));
}

fn hud(
    arena: Res<Arena>,
    rules: Res<CombatRules>,
    player: Single<&Combatant, With<Player>>,
    enemies: Query<&Combatant, (With<Enemy>, Without<Player>)>,
    mut text: Single<&mut Text, With<HudText>>,
    mut bars: ParamSet<(
        Single<&mut Node, With<HealthBar>>,
        Single<&mut Node, With<StaminaBar>>,
        Single<&mut Node, With<EnemyBar>>,
    )>,
) {
    let r = &rules.0;
    let f = &player.fighter;
    bars.p0().width = percent(100.0 * f.health() / r.max_health);
    bars.p1().width = percent(100.0 * f.stamina() / r.stamina.max.max(1.0));
    let enemy = enemies.iter().next().map(|c| c.fighter.health() / r.max_health).unwrap_or(0.0);
    bars.p2().width = percent(100.0 * enemy);
    let status = match (arena.restart.is_some(), f.phase() == Phase::Dead) {
        (true, true) => "  ·  MORTO",
        (true, false) => "  ·  VITÓRIA",
        _ => "",
    };
    text.0 = format!(
        "Luta {}  ·  V {}  D {}  ·  sequência {}{}",
        arena.fights, arena.wins, arena.losses, arena.streak, status
    );
}
