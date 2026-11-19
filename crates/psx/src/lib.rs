//! PS1-style rendering for Bevy: low-resolution render target, vertex snapping,
//! nearest-neighbour textures, colour quantisation + ordered dithering and linear fog.
//!
//! Usage: add [`PsxPlugin`], tag every 3D camera with [`PsxCamera`] and keep using
//! `StandardMaterial` — meshes are converted to [`PsxMaterial`] automatically
//! (opt out per entity with [`NoPsx`]).

use std::collections::{HashMap, HashSet};

use bevy::{
    asset::{embedded_asset, AssetEvent, LoadState},
    camera::{visibility::RenderLayers, RenderTarget},
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin},
    prelude::*,
    render::render_resource::{AsBindGroup, Extent3d, ShaderType, TextureFormat},
    shader::ShaderRef,
    window::PrimaryWindow,
};
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use serde::{Deserialize, Serialize};

/// Render layer used by the internal camera/sprite that presents the low-res image.
pub const PSX_OUTPUT_LAYER: usize = 31;
/// Camera order of the output camera. Game cameras that render into the PSX image must use a
/// lower order; anything that should draw on top of the upscaled image (UI) a higher one.
pub const PSX_OUTPUT_ORDER: isize = 100;

/// The extended material every converted mesh renders with.
pub type PsxMaterial = ExtendedMaterial<StandardMaterial, PsxExtension>;

// ---------------------------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------------------------

/// Global look settings. Replace/mutate the resource at runtime; the plugin reacts to changes.
#[derive(Resource, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Resource, Default)]
#[serde(default)]
pub struct PsxSettings {
    /// Size of the low-resolution render target.
    pub resolution: UVec2,
    /// Vertex snapping grid in screen space (pixels). Zero on any axis disables snapping.
    pub snap_resolution: Vec2,
    /// Colour levels per channel (quantisation). `<= 1` disables quantisation.
    pub color_levels: f32,
    /// Ordered (Bayer 4x4) dither strength, 0..1.
    pub dither: f32,
    /// Fog colour, sRGB.
    pub fog_color: [f32; 3],
    /// Distance at which fog starts.
    pub fog_start: f32,
    /// Distance at which fog is fully opaque.
    pub fog_end: f32,
    /// PS1-style affine (non perspective-correct) texture mapping.
    pub affine_textures: bool,
}

impl Default for PsxSettings {
    fn default() -> Self {
        Self {
            resolution: UVec2::new(320, 240),
            snap_resolution: Vec2::new(160.0, 120.0),
            color_levels: 32.0,
            dither: 0.6,
            fog_color: [0.05, 0.05, 0.08],
            fog_start: 8.0,
            fog_end: 30.0,
            affine_textures: false,
        }
    }
}

impl PsxSettings {
    pub fn fog_color(&self) -> Color {
        Color::srgb(self.fog_color[0], self.fog_color[1], self.fog_color[2])
    }

    fn params(&self) -> PsxParams {
        PsxParams {
            a: Vec4::new(
                self.snap_resolution.x,
                self.snap_resolution.y,
                self.color_levels,
                self.dither.clamp(0.0, 1.0),
            ),
            b: Vec4::new(if self.affine_textures { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0),
        }
    }

    fn fog(&self) -> DistanceFog {
        DistanceFog {
            color: self.fog_color(),
            directional_light_color: Color::NONE,
            directional_light_exponent: 8.0,
            falloff: FogFalloff::Linear {
                start: self.fog_start,
                end: self.fog_end.max(self.fog_start + 0.001),
            },
        }
    }

    fn extent(&self) -> Extent3d {
        Extent3d {
            width: self.resolution.x.max(1),
            height: self.resolution.y.max(1),
            depth_or_array_layers: 1,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Material
// ---------------------------------------------------------------------------------------------

/// Uniform block of the PSX extension (mirrors `PsxParams` in `psx.wesl`).
#[derive(Clone, Copy, Debug, Default, ShaderType, Reflect)]
pub struct PsxParams {
    /// xy: snap grid, z: colour levels, w: dither strength.
    pub a: Vec4,
    /// x: affine textures flag.
    pub b: Vec4,
}

/// Extension for [`StandardMaterial`] implementing the PS1 look.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct PsxExtension {
    // Binding slots 0..100 belong to StandardMaterial.
    #[uniform(100)]
    pub params: PsxParams,
}

const SHADER_PATH: &str = "embedded://psx/psx.wesl";

impl MaterialExtension for PsxExtension {
    fn vertex_shader() -> ShaderRef {
        SHADER_PATH.into()
    }
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }
}

// ---------------------------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------------------------

/// Marks a 3D camera that renders into the shared low-resolution image.
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Default)]
pub struct PsxCamera;

/// Entities with this component keep their original material (no PSX conversion, no flash).
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Default)]
pub struct NoPsx;

/// While present, the entity and all its descendants render with a solid unlit flash colour.
/// The plugin counts `remaining` (seconds) down and removes the component at zero.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct PsxFlash {
    pub remaining: f32,
    pub color: Color,
}

/// Marker of the internal camera presenting the upscaled image.
#[derive(Component)]
pub struct PsxOutputCamera;

/// Marker of the internal sprite showing the low-resolution image.
#[derive(Component)]
pub struct PsxOutputSprite;

/// The shared low-resolution render target.
#[derive(Resource, Clone)]
pub struct PsxTarget(pub Handle<Image>);

/// Original material of a mesh while a [`PsxFlash`] is active.
#[derive(Component)]
struct FlashOriginal(Handle<PsxMaterial>);

/// Present on the entity carrying a [`PsxFlash`] once its materials were swapped.
#[derive(Component)]
struct FlashActive(Handle<PsxMaterial>);

#[derive(Resource, Default)]
struct MaterialCache {
    map: HashMap<AssetId<StandardMaterial>, Handle<PsxMaterial>>,
}

#[derive(Resource, Default)]
struct PendingSamplers(Vec<Handle<Image>>);

// ---------------------------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------------------------

pub struct PsxPlugin;

impl Plugin for PsxPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "psx.wesl");

        app.add_plugins(MaterialPlugin::<PsxMaterial>::default())
            .init_resource::<PsxSettings>()
            .init_resource::<MaterialCache>()
            .init_resource::<PendingSamplers>()
            .register_type::<PsxSettings>()
            .register_type::<PsxCamera>()
            .register_type::<NoPsx>()
            .register_type::<PsxFlash>();

        // Create the target image right away so cameras spawned in Startup can reference it.
        let settings = app.world().resource::<PsxSettings>().clone();
        let mut image = Image::new_target_texture(
            settings.extent().width,
            settings.extent().height,
            TextureFormat::Rgba8UnormSrgb,
            None,
        );
        image.sampler = ImageSampler::nearest();
        let handle = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        app.insert_resource(PsxTarget(handle));

        app.add_observer(on_add_camera)
            .add_observer(on_remove_flash)
            .add_systems(Startup, spawn_output)
            .add_systems(
                PostUpdate,
                (
                    sync_settings.run_if(resource_changed::<PsxSettings>),
                    layout_output,
                    sync_standard_changes,
                    convert_materials,
                    fix_samplers,
                    tick_flash,
                    apply_flash,
                )
                    .chain()
                    .before(bevy::camera::CameraUpdateSystems),
            );
    }
}

/// Configures camera components needed by the PSX pipeline.
fn on_add_camera(
    add: On<Add<PsxCamera>>,
    mut commands: Commands,
    target: Res<PsxTarget>,
    settings: Res<PsxSettings>,
) {
    commands.entity(add.entity).try_insert((
        RenderTarget::Image(target.0.clone().into()),
        Msaa::Off,
        // Colour grading must not alter the quantised palette.
        Tonemapping::None,
        DebandDither::Disabled,
        settings.fog(),
    ));
}

fn spawn_output(mut commands: Commands, target: Res<PsxTarget>) {
    let layer = RenderLayers::layer(PSX_OUTPUT_LAYER);
    commands.spawn((
        Camera2d,
        Camera {
            order: PSX_OUTPUT_ORDER,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Msaa::Off,
        layer.clone(),
        PsxOutputCamera,
        Name::new("PsxOutputCamera"),
    ));
    commands.spawn((
        Sprite::from_image(target.0.clone()),
        layer,
        PsxOutputSprite,
        Name::new("PsxOutputSprite"),
    ));
}

/// Applies settings changes: target size, material uniforms, camera fog.
fn sync_settings(
    settings: Res<PsxSettings>,
    target: Res<PsxTarget>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<PsxMaterial>>,
    mut fogs: Query<&mut DistanceFog, With<PsxCamera>>,
) {
    let extent = settings.extent();
    if images
        .get(&target.0)
        .is_some_and(|i| i.texture_descriptor.size != extent)
        && let Some(mut image) = images.get_mut(&target.0)
    {
        image.resize(extent);
    }

    let params = settings.params();
    let ids: Vec<_> = materials.ids().collect();
    for id in ids {
        let outdated = materials
            .get(id)
            .is_some_and(|m| params_differ(&m.extension.params, &params));
        if outdated && let Some(mut m) = materials.get_mut(id) {
            m.extension.params = params;
        }
    }

    let fog = settings.fog();
    for mut f in &mut fogs {
        *f = fog.clone();
    }
}

fn params_differ(a: &PsxParams, b: &PsxParams) -> bool {
    a.a != b.a || a.b != b.b
}

/// Scales the low-res sprite by an integer factor (when possible), centred (letterbox).
fn layout_output(
    settings: Res<PsxSettings>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut sprite: Single<&mut Sprite, With<PsxOutputSprite>>,
    target: Res<PsxTarget>,
) {
    let Some(window) = window else { return };
    let (pw, ph) = (window.physical_width() as f32, window.physical_height() as f32);
    if pw < 1.0 || ph < 1.0 {
        return;
    }
    let res = settings.resolution.max(UVec2::ONE).as_vec2();
    let fit = (pw / res.x).min(ph / res.y);
    let scale = if fit >= 1.0 { fit.floor() } else { fit };
    // Camera2d units are logical pixels.
    let size = res * scale / window.scale_factor();
    if sprite.custom_size != Some(size) {
        sprite.custom_size = Some(size);
    }
    if sprite.image != target.0 {
        sprite.image = target.0.clone();
    }
}

fn base_params(settings: &PsxSettings) -> PsxExtension {
    PsxExtension {
        params: settings.params(),
    }
}

/// Collects every texture of a material so its sampler can be forced to nearest.
fn material_textures(m: &StandardMaterial) -> Vec<Handle<Image>> {
    [
        &m.base_color_texture,
        &m.emissive_texture,
        &m.metallic_roughness_texture,
        &m.normal_map_texture,
        &m.occlusion_texture,
    ]
    .into_iter()
    .flatten()
    .cloned()
    .collect()
}

/// Keeps converted materials in sync with edits (and removals) of their StandardMaterial.
fn sync_standard_changes(
    mut events: MessageReader<AssetEvent<StandardMaterial>>,
    standard: Res<Assets<StandardMaterial>>,
    mut psx: ResMut<Assets<PsxMaterial>>,
    mut cache: ResMut<MaterialCache>,
    mut pending: ResMut<PendingSamplers>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } => {
                if let (Some(h), Some(src)) = (cache.map.get(id), standard.get(*id))
                    && let Some(mut dst) = psx.get_mut(h)
                {
                    pending.0.extend(material_textures(src));
                    dst.base = src.clone();
                }
            }
            AssetEvent::Removed { id } | AssetEvent::Unused { id } => {
                cache.map.remove(id);
            }
            _ => {}
        }
    }
}

type ConvertItem = (
    Entity,
    &'static MeshMaterial3d<StandardMaterial>,
    Option<&'static FlashOriginal>,
);

/// Swaps `MeshMaterial3d<StandardMaterial>` for an equivalent `MeshMaterial3d<PsxMaterial>`.
fn convert_materials(
    mut commands: Commands,
    settings: Res<PsxSettings>,
    standard: Res<Assets<StandardMaterial>>,
    mut psx: ResMut<Assets<PsxMaterial>>,
    mut cache: ResMut<MaterialCache>,
    mut pending: ResMut<PendingSamplers>,
    meshes: Query<ConvertItem, Without<NoPsx>>,
) {
    for (entity, handle, flashed) in &meshes {
        let converted = match cache.map.get(&handle.id()) {
            Some(h) => h.clone(),
            None => {
                let Some(src) = standard.get(&handle.0) else {
                    continue; // not loaded yet, retry next frame
                };
                pending.0.extend(material_textures(src));
                let h = psx.add(PsxMaterial {
                    base: src.clone(),
                    extension: base_params(&settings),
                });
                cache.map.insert(handle.id(), h.clone());
                h
            }
        };
        let mut e = commands.entity(entity);
        e.remove::<MeshMaterial3d<StandardMaterial>>();
        if flashed.is_some() {
            // A flash is active: only update what we restore to afterwards.
            e.insert(FlashOriginal(converted));
        } else {
            e.insert(MeshMaterial3d(converted));
        }
    }
}

fn nearest_descriptor(sampler: &ImageSampler) -> ImageSamplerDescriptor {
    let mut d = match sampler {
        ImageSampler::Descriptor(d) => d.clone(),
        ImageSampler::Default => ImageSamplerDescriptor::default(),
    };
    d.mag_filter = ImageFilterMode::Nearest;
    d.min_filter = ImageFilterMode::Nearest;
    d.mipmap_filter = ImageFilterMode::Nearest;
    d.anisotropy_clamp = 1;
    d
}

fn is_nearest(sampler: &ImageSampler) -> bool {
    match sampler {
        ImageSampler::Default => false,
        ImageSampler::Descriptor(d) => {
            d.mag_filter == ImageFilterMode::Nearest
                && d.min_filter == ImageFilterMode::Nearest
                && d.mipmap_filter == ImageFilterMode::Nearest
                && d.anisotropy_clamp == 1
        }
    }
}

/// Forces nearest sampling on pending textures (retries until they finish loading).
fn fix_samplers(
    mut pending: ResMut<PendingSamplers>,
    mut images: ResMut<Assets<Image>>,
    server: Res<AssetServer>,
    target: Res<PsxTarget>,
) {
    if pending.0.is_empty() {
        return;
    }
    let mut seen = HashSet::new();
    pending.0.retain(|h| {
        if h.id() == target.0.id() || !seen.insert(h.id()) {
            return false;
        }
        let Some(sampler) = images.get(h).map(|i| i.sampler.clone()) else {
            return !matches!(server.get_load_state(h.id()), Some(LoadState::Failed(_)));
        };
        if !is_nearest(&sampler)
            && let Some(mut image) = images.get_mut(h)
        {
            image.sampler = ImageSampler::Descriptor(nearest_descriptor(&sampler));
        }
        false
    });
}

// ---------------------------------------------------------------------------------------------
// Flash
// ---------------------------------------------------------------------------------------------

fn flash_material(color: Color, settings: &PsxSettings) -> PsxMaterial {
    PsxMaterial {
        base: StandardMaterial {
            base_color: color,
            unlit: true,
            fog_enabled: false,
            ..default()
        },
        extension: base_params(settings),
    }
}

fn tick_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut PsxFlash)>,
) {
    for (entity, mut flash) in &mut flashes {
        flash.remaining -= time.delta_secs();
        if flash.remaining <= 0.0 {
            commands.entity(entity).try_remove::<PsxFlash>();
        }
    }
}

/// Starts flashes (swaps materials) and keeps the flash colour up to date.
fn apply_flash(
    mut commands: Commands,
    settings: Res<PsxSettings>,
    mut psx: ResMut<Assets<PsxMaterial>>,
    new: Query<(Entity, &PsxFlash), Without<FlashActive>>,
    active: Query<(&PsxFlash, &FlashActive)>,
    children: Query<&Children>,
    meshes: Query<&MeshMaterial3d<PsxMaterial>, Without<NoPsx>>,
) {
    for (flash, state) in &active {
        let outdated = psx
            .get(&state.0)
            .is_some_and(|m| m.base.base_color != flash.color);
        if outdated && let Some(mut m) = psx.get_mut(&state.0) {
            m.base.base_color = flash.color;
        }
    }

    for (root, flash) in &new {
        let handle = psx.add(flash_material(flash.color, &settings));
        for e in std::iter::once(root).chain(children.iter_descendants(root)) {
            if let Ok(original) = meshes.get(e) {
                commands
                    .entity(e)
                    .insert((FlashOriginal(original.0.clone()), MeshMaterial3d(handle.clone())));
            }
        }
        commands.entity(root).insert(FlashActive(handle));
    }
}

/// Restores the original materials when a [`PsxFlash`] is removed (or its entity despawned).
fn on_remove_flash(
    remove: On<Remove<PsxFlash>>,
    mut commands: Commands,
    children: Query<&Children>,
    originals: Query<&FlashOriginal>,
) {
    let root = remove.entity;
    for e in std::iter::once(root).chain(children.iter_descendants(root)) {
        if let Ok(original) = originals.get(e) {
            commands
                .entity(e)
                .try_insert(MeshMaterial3d(original.0.clone()))
                .try_remove::<FlashOriginal>();
        }
    }
    commands.entity(root).try_remove::<FlashActive>();
}
