//! Validation scene: a room with pillars, a spinning flashing cube and a "viewmodel" camera.
//! Takes screenshots, hot-swaps the settings at runtime and exits by itself.
//! Run: `cargo run -p psx --example room` (set `PSX_SHOT_DIR` to choose the screenshot folder).

use bevy::{
    asset::RenderAssetUsages,
    camera::{visibility::RenderLayers, ClearColorConfig},
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        view::screenshot::{save_to_disk, Screenshot},
    },
};
use psx::{PsxCamera, PsxFlash, PsxPlugin, PsxSettings};

#[derive(Component)]
struct Spinner;
#[derive(Component)]
struct Orbit;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (1280, 720).into(),
                title: "psx room".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PsxPlugin)
        .insert_resource(PsxSettings::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (spin, flash, orbit, script))
        .run();
}

fn checker(images: &mut Assets<Image>, a: [u8; 4], b: [u8; 4]) -> Handle<Image> {
    const N: u32 = 16;
    let mut data = Vec::new();
    for y in 0..N {
        for x in 0..N {
            // Big squares plus a bit of per-texel noise so texture filtering is visible.
            let c = if (x / 4 + y / 4) % 2 == 0 { a } else { b };
            let n = ((x * 7 + y * 13) % 5) as u8 * 3;
            data.extend_from_slice(&[
                c[0].saturating_sub(n),
                c[1].saturating_sub(n),
                c[2].saturating_sub(n),
                255,
            ]);
        }
    }
    let mut img = Image::new(
        Extent3d {
            width: N,
            height: N,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    // Deliberately linear: the plugin must force nearest.
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    images.add(img)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    settings: Res<PsxSettings>,
) {
    let floor_tex = checker(&mut images, [150, 120, 90, 255], [90, 70, 50, 255]);
    let wall_tex = checker(&mut images, [120, 130, 150, 255], [80, 90, 110, 255]);
    let tiled = |scale: f32, tex: &Handle<Image>| StandardMaterial {
        base_color_texture: Some(tex.clone()),
        uv_transform: Affine2::from_scale(Vec2::splat(scale)),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
    };
    let floor = materials.add(tiled(10.0, &floor_tex));
    let wall = materials.add(tiled(4.0, &wall_tex));
    let pillar = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.55, 0.45),
        perceptual_roughness: 1.0,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(floor.clone()),
    ));
    // Ceiling.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(floor),
        Transform::from_xyz(0.0, 4.0, 0.0).with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
    ));
    for (pos, size) in [
        (Vec3::new(0.0, 2.0, -10.0), Vec3::new(20.0, 4.0, 0.3)),
        (Vec3::new(0.0, 2.0, 10.0), Vec3::new(20.0, 4.0, 0.3)),
        (Vec3::new(-10.0, 2.0, 0.0), Vec3::new(0.3, 4.0, 20.0)),
        (Vec3::new(10.0, 2.0, 0.0), Vec3::new(0.3, 4.0, 20.0)),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(wall.clone()),
            Transform::from_translation(pos),
        ));
    }
    for i in 0..6 {
        let x = -7.5 + i as f32 * 3.0;
        for z in [-6.0, 6.0] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.8, 4.0, 0.8))),
                MeshMaterial3d(pillar.clone()),
                Transform::from_xyz(x, 2.0, z),
            ));
        }
    }

    // Spinning cube that flashes.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.2, 1.2, 1.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(wall_tex),
            base_color: Color::srgb(1.0, 0.8, 0.8),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Spinner,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 4500.0,
            ..default()
        },
        Transform::from_xyz(3.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::from_layers(&[0, 1]),
    ));
    commands.insert_resource(GlobalAmbientLight {
        brightness: 250.0,
        ..default()
    });
    commands.spawn((
        PointLight {
            intensity: 400_000.0,
            color: Color::srgb(1.0, 0.7, 0.4),
            range: 20.0,
            ..default()
        },
        Transform::from_xyz(4.0, 3.0, -3.0),
    ));

    // World camera.
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(settings.fog_color()),
            ..default()
        },
        Projection::from(PerspectiveProjection {
            fov: 75f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 1.7, 7.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        PsxCamera,
        Orbit,
    ));
    // Viewmodel camera: own layer, only clears depth, renders on top into the same image.
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Projection::from(PerspectiveProjection {
            fov: 60f32.to_radians(),
            ..default()
        }),
        RenderLayers::layer(1),
        PsxCamera,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 0.15, 0.7))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.35))),
        Transform::from_xyz(0.3, -0.25, -0.7),
        RenderLayers::layer(1),
    ));
}

fn spin(time: Res<Time>, mut q: Query<&mut Transform, With<Spinner>>) {
    for mut t in &mut q {
        t.rotation = Quat::from_euler(EulerRot::YXZ, time.elapsed_secs(), time.elapsed_secs() * 0.6, 0.0);
    }
}

fn orbit(time: Res<Time>, mut q: Query<&mut Transform, With<Orbit>>) {
    let a = time.elapsed_secs() * 0.15 + 0.4;
    for mut t in &mut q {
        *t = Transform::from_xyz(a.sin() * 7.0, 1.7, a.cos() * 7.0)
            .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);
    }
}

fn flash(
    mut commands: Commands,
    time: Res<Time>,
    q: Query<(Entity, Has<PsxFlash>), With<Spinner>>,
) {
    // Flash for 0.3s every 1.5s.
    if (time.elapsed_secs() % 1.5) < time.delta_secs() {
        for (e, flashing) in &q {
            if !flashing {
                commands.entity(e).insert(PsxFlash {
                    remaining: 0.3,
                    color: Color::srgb(1.0, 0.1, 0.1),
                });
            }
        }
    }
}

/// Screenshots, runtime settings hot-swap, exit.
fn script(
    mut commands: Commands,
    time: Res<Time>,
    mut settings: ResMut<PsxSettings>,
    mut step: Local<u32>,
    mut exit: MessageWriter<AppExit>,
) {
    let dir = std::env::var("PSX_SHOT_DIR").unwrap_or_else(|_| ".".into());
    let t = time.elapsed_secs();
    let shot = |commands: &mut Commands, name: &str| {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("{dir}/{name}.png")));
    };
    match *step {
        0 if t > 3.0 => {
            shot(&mut commands, "psx_1_default");
            *step = 1;
        }
        1 if t > 3.5 => {
            // Hot swap: coarser resolution, harsher quantisation, affine textures, closer fog.
            *settings = PsxSettings {
                resolution: UVec2::new(200, 150),
                snap_resolution: Vec2::new(100.0, 75.0),
                color_levels: 8.0,
                dither: 1.0,
                affine_textures: true,
                fog_start: 4.0,
                fog_end: 16.0,
                ..default()
            };
            *step = 2;
        }
        2 if t > 5.0 => {
            shot(&mut commands, "psx_2_swapped");
            *step = 3;
        }
        3 if t > 5.5 => {
            *settings = PsxSettings::default();
            *step = 4;
        }
        4 if t > 7.0 => {
            shot(&mut commands, "psx_3_restored");
            *step = 5;
        }
        5 if t > 8.0 => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
