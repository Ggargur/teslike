//! The test room: floor, walls, pillars, a ledge over a pit, a spike wall
//! and a flight of steps. Everything solid has a collider.

use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// Touching this hurts. Sensor volume in front of the spike wall.
#[derive(Component)]
pub struct Spikes;

/// Arena half-extent on X/Z.
pub const HALF: f32 = 9.0;
/// Floor ends here on +X; beyond is the pit.
pub const LEDGE_X: f32 = 5.0;
pub const WALL_H: f32 = 4.5;
/// Anything below this height fell into the pit.
pub const FALL_Y: f32 = -3.0;

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.03)))
            .insert_resource(GlobalAmbientLight { color: Color::srgb(0.6, 0.55, 0.7), brightness: 450.0, ..default() })
            .add_systems(Startup, spawn_room);
    }
}

/// Small procedural texture: base color with per-texel noise and mortar lines.
fn texture(images: &mut Assets<Image>, base: [u8; 3], bricks: bool, seed: u32) -> Handle<Image> {
    const N: u32 = 32;
    let mut data = Vec::with_capacity((N * N * 4) as usize);
    let mut s = seed.wrapping_mul(2654435761).max(1);
    for y in 0..N {
        for x in 0..N {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            let noise = (s % 40) as i32 - 20;
            let offset = if (y / 8) % 2 == 0 { 0 } else { 8 };
            let mortar = bricks && (y % 8 == 0 || (x + offset) % 16 == 0);
            let k = if mortar { -45 } else { noise };
            for c in base {
                data.push((c as i32 + k).clamp(0, 255) as u8);
            }
            data.push(255);
        }
    }
    let mut img = Image::new(
        Extent3d { width: N, height: N, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: bevy::image::ImageAddressMode::Repeat,
        address_mode_v: bevy::image::ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::nearest()
    });
    images.add(img)
}

/// A box mesh whose UVs repeat once per `tile` meters.
fn tiled_box(size: Vec3, tile: f32) -> Mesh {
    let mut mesh = Mesh::from(Cuboid::from_size(size));
    if let Some(bevy::mesh::VertexAttributeValues::Float32x3(pos)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION).cloned()
        && let Some(bevy::mesh::VertexAttributeValues::Float32x3(nrm)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL).cloned()
    {
        let uvs: Vec<[f32; 2]> = pos
            .iter()
            .zip(nrm.iter())
            .map(|(p, n)| {
                let (u, v) = if n[0].abs() > 0.5 {
                    (p[2], p[1])
                } else if n[1].abs() > 0.5 {
                    (p[0], p[2])
                } else {
                    (p[0], p[1])
                };
                [u / tile, -v / tile]
            })
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }
    mesh
}

fn spawn_room(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut mat = |images: &mut Assets<Image>, base: [u8; 3], bricks: bool, seed: u32| {
        materials.add(StandardMaterial {
            base_color_texture: Some(texture(images, base, bricks, seed)),
            perceptual_roughness: 0.95,
            reflectance: 0.1,
            ..default()
        })
    };
    let floor_mat = mat(&mut images, [92, 84, 76], true, 1);
    let wall_mat = mat(&mut images, [110, 96, 84], true, 2);
    let pillar_mat = mat(&mut images, [130, 120, 105], false, 3);
    let pit_mat = mat(&mut images, [40, 36, 40], false, 4);
    let metal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.52, 0.5),
        metallic: 0.6,
        perceptual_roughness: 0.4,
        ..default()
    });

    let mut solid = |name: &str, size: Vec3, at: Vec3, m: &Handle<StandardMaterial>| {
        commands.spawn((
            Name::new(name.to_string()),
            Mesh3d(meshes.add(tiled_box(size, 2.0))),
            MeshMaterial3d(m.clone()),
            Transform::from_translation(at),
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
        ));
    };

    // Floor up to the ledge, and the pit bottom far below.
    let floor_w = LEDGE_X + HALF;
    solid("Floor", Vec3::new(floor_w, 1.0, HALF * 2.0), Vec3::new(LEDGE_X - floor_w / 2.0, -0.5, 0.0), &floor_mat);
    let pit_w = HALF - LEDGE_X;
    solid("Pit bottom", Vec3::new(pit_w, 1.0, HALF * 2.0), Vec3::new(LEDGE_X + pit_w / 2.0, -9.0, 0.0), &pit_mat);
    // Ledge wall going down into the pit.
    solid("Ledge face", Vec3::new(0.4, 8.0, HALF * 2.0), Vec3::new(LEDGE_X - 0.2, -4.5, 0.0), &wall_mat);

    // Walls (the +X wall reaches the pit bottom).
    let h = WALL_H;
    solid("Wall -X", Vec3::new(1.0, h, HALF * 2.0 + 2.0), Vec3::new(-HALF - 0.5, h / 2.0, 0.0), &wall_mat);
    solid("Wall +X", Vec3::new(1.0, h + 9.0, HALF * 2.0 + 2.0), Vec3::new(HALF + 0.5, (h - 9.0) / 2.0, 0.0), &wall_mat);
    solid("Wall -Z", Vec3::new(HALF * 2.0, h + 9.0, 1.0), Vec3::new(0.0, (h - 9.0) / 2.0, -HALF - 0.5), &wall_mat);
    solid("Wall +Z", Vec3::new(HALF * 2.0, h + 9.0, 1.0), Vec3::new(0.0, (h - 9.0) / 2.0, HALF + 0.5), &wall_mat);
    solid("Ceiling", Vec3::new(HALF * 2.0 + 2.0, 1.0, HALF * 2.0 + 2.0), Vec3::new(0.0, h + 0.5, 0.0), &wall_mat);

    // Pillars.
    for (x, z) in [(-4.0, -4.0), (-4.0, 4.0), (2.0, -5.0)] {
        solid("Pillar", Vec3::new(0.9, h, 0.9), Vec3::new(x, h / 2.0, z), &pillar_mat);
    }

    // Steps up to a small platform in the -Z/-X corner.
    for i in 0..4 {
        let rise = 0.25 * (i + 1) as f32;
        solid(
            "Step",
            Vec3::new(1.0, rise, 2.5),
            Vec3::new(-2.5 - i as f32, rise / 2.0, -HALF + 1.25),
            &pillar_mat,
        );
    }
    solid("Platform", Vec3::new(2.5, 1.0, 2.5), Vec3::new(-7.75, 0.5, -HALF + 1.25), &pillar_mat);

    // Spike wall on -X: a row of cones plus a sensor volume.
    let cone = meshes.add(Cone { radius: 0.12, height: 0.7 });
    for iy in 0..3 {
        for iz in 0..9 {
            let z = -2.0 + iz as f32 * 0.5;
            let y = 0.35 + iy as f32 * 0.55;
            commands.spawn((
                Mesh3d(cone.clone()),
                MeshMaterial3d(metal.clone()),
                Transform::from_xyz(-HALF + 0.35, y, z).with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
            ));
        }
    }
    commands.spawn((
        Name::new("Spikes"),
        Spikes,
        Transform::from_xyz(-HALF + 0.45, 1.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(0.7, 2.0, 4.6),
        Sensor,
        CollisionLayers::new(crate::mover::Layer::Trigger, [crate::mover::Layer::Character]),
    ));

    // Warm torch lights.
    for (x, z) in [(-7.5, -6.0), (-7.5, 6.0), (3.0, 7.5), (0.0, -7.5)] {
        commands.spawn((
            PointLight { color: Color::srgb(1.0, 0.7, 0.4), intensity: 150_000.0, range: 18.0, ..default() },
            Transform::from_xyz(x, 3.2, z),
            // Lights the viewmodel layer too.
            bevy::camera::visibility::RenderLayers::from_layers(&[0, crate::player::VIEWMODEL_LAYER]),
        ));
    }
}
