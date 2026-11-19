//! Pure combat rules. No Bevy here: the game feeds intents and time,
//! and gets back events describing *what happened*.

mod config;
mod fighter;

pub use config::*;
pub use fighter::*;
