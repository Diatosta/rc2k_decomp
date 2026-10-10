//! Saved games.

use crate::framework::GameHook;

pub mod gather_savegame;
pub mod load_savegame;
pub mod read_savegame_slot;
pub mod write_savegame_slot;

pub static HOOKS: &[&dyn GameHook] = &[
    &gather_savegame::HOOK,
    &write_savegame_slot::HOOK,
    &read_savegame_slot::HOOK,
    &load_savegame::HOOK,
];
