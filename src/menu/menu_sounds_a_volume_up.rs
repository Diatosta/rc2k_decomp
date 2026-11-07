//! `menu::menu_sounds_a_volume_up` (0x4680AE): the first menu sounds' volume one more, heard (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: as `option_steps` says (not `pusha`'d)
//! A left/right handler of the sound options page's items. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::menu::option_steps::{SOUNDS_A_RVA, Then, Volume, volume_step};

const STEP: Volume = Volume {
    value: SOUNDS_A_RVA,
    up: true,
    then: Then::Sound(0x20),
};

/// The Rust version the typed tables run.
pub fn run(menus: &crate::menu::owner::Menus) {
    let screens = &menus.screens;
    volume_step(screens, STEP);
}
