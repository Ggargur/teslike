//! Tunable numbers live in `assets/config/*.ron` and hot reload while the
//! game runs. A missing file is written out with the defaults, so every
//! knob is discoverable by opening the file.

use std::{
    marker::PhantomData,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use bevy::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

/// Absolute path of the workspace `assets/` folder.
pub fn assets_dir() -> PathBuf {
    std::env::var_os("TESLIKE_ASSETS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets")))
}

/// A resource backed by a `.ron` file under `assets/config/`.
pub trait ConfigFile: Resource + Component<Mutability = bevy::ecs::component::Mutable> + Clone + Default + Serialize + DeserializeOwned {
    const FILE: &'static str;
}

pub struct ConfigPlugin<T>(PhantomData<T>);

impl<T> Default for ConfigPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Resource)]
struct Watch<T> {
    modified: Option<SystemTime>,
    timer: Timer,
    _t: PhantomData<T>,
}

fn path<T: ConfigFile>() -> PathBuf {
    assets_dir().join("config").join(T::FILE)
}

fn mtime(p: &PathBuf) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

fn read<T: ConfigFile>() -> Option<T> {
    let p = path::<T>();
    let text = std::fs::read_to_string(&p).ok()?;
    match ron::from_str::<T>(&text) {
        Ok(v) => Some(v),
        Err(e) => {
            warn!("{}: {e}", p.display());
            None
        }
    }
}

impl<T: ConfigFile> Plugin for ConfigPlugin<T> {
    fn build(&self, app: &mut App) {
        let p = path::<T>();
        if !p.exists() {
            let pretty = ron::ser::PrettyConfig::default().struct_names(false);
            if let Ok(text) = ron::ser::to_string_pretty(&T::default(), pretty) {
                let _ = std::fs::create_dir_all(p.parent().unwrap());
                let _ = std::fs::write(&p, text);
            }
        }
        app.insert_resource(read::<T>().unwrap_or_default())
            .insert_resource(Watch::<T> {
                modified: mtime(&p),
                timer: Timer::new(Duration::from_millis(300), TimerMode::Repeating),
                _t: PhantomData,
            })
            .add_systems(PreUpdate, reload::<T>);
    }
}

fn reload<T: ConfigFile>(time: Res<Time<Real>>, mut watch: ResMut<Watch<T>>, mut res: ResMut<T>) {
    if !watch.timer.tick(time.delta()).just_finished() {
        return;
    }
    let m = mtime(&path::<T>());
    if m == watch.modified {
        return;
    }
    watch.modified = m;
    if let Some(v) = read::<T>() {
        info!("reloaded {}", T::FILE);
        *res = v;
    }
}
