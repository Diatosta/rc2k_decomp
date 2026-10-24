//! `menu::menu_sounds_a_volume_down` (0x468080): the first menu sounds' volume one less, heard (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: as `option_steps` says (not `pusha`'d)
//! A left/right handler of the sound options page's items. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::menu::option_steps::{SOUNDS_A_RVA, Then, Volume, volume_step};

const STEP: Volume = Volume {
    value_rva: SOUNDS_A_RVA,
    up: false,
    then: Then::Sound(0x10),
};

/// The Rust version the typed tables run.
pub fn run() {
    volume_step(STEP)
}
