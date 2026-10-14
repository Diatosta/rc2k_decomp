//! The audio thread (`audio::thread`, 0x409CCE): the side that carries out what `sound::`
//! sends. See `voices.rs`.

use crate::framework::GameHook;

pub mod start_voice;
pub mod voices;

pub static HOOKS: &[&dyn GameHook] = &[&start_voice::HOOK];
