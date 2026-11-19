// Bevy systems take many params and query tuples by design.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod arena;
mod combat;
mod config;
mod enemy;
mod feel;
mod input;
mod mover;
mod player;
mod room;
mod script;
mod viewmodel;

use avian3d::prelude::*;
use bevy::{
    app::HierarchyPropagatePlugin, camera::visibility::RenderLayers, input::common_conditions::input_just_pressed,
    prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

/// Whether the live inspector (F1) is showing.
#[derive(Resource, Default)]
pub struct InspectorOpen(pub bool);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin { file_path: config::assets_dir().to_string_lossy().into_owned(), ..default() })
                .set(WindowPlugin {
                    primary_window: Some(Window { title: "teslike: protótipo de combate".into(), ..default() }),
                    ..default()
                }),
        )
        .add_plugins((
            PhysicsPlugins::default(),
            HierarchyPropagatePlugin::<RenderLayers>::new(PostUpdate),
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(|open: Res<InspectorOpen>| open.0),
            psx::PsxPlugin,
        ))
        .add_plugins(config::ConfigPlugin::<psx::PsxSettings>::default())
        .init_resource::<InspectorOpen>()
        .add_plugins((
            input::InputPlugin,
            mover::MoverPlugin,
            room::RoomPlugin,
            combat::CombatPlugin,
            player::PlayerPlugin,
            viewmodel::ViewmodelPlugin,
            feel::FeelPlugin,
            enemy::EnemyPlugin,
            arena::ArenaPlugin,
            script::ScriptPlugin,
        ))
        .add_systems(Update, (toggle_inspector.run_if(input_just_pressed(KeyCode::F1)), sky_matches_fog))
        .run();
}

fn toggle_inspector(mut open: ResMut<InspectorOpen>) {
    open.0 = !open.0;
}

impl config::ConfigFile for psx::PsxSettings {
    const FILE: &'static str = "psx.ron";
}

/// The world camera clears to the fog colour so distance fades into it.
fn sky_matches_fog(settings: Res<psx::PsxSettings>, mut cams: Query<&mut Camera, With<player::Head>>) {
    if settings.is_changed() {
        for mut c in &mut cams {
            c.clear_color = ClearColorConfig::Custom(settings.fog_color());
        }
    }
}
