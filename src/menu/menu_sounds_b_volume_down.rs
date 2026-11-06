//! `menu::menu_sounds_b_volume_down` (0x4680DC): the second menu sounds' volume one less, heard (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: as `option_steps` says (not `pusha`'d)
//! A left/right handler of the sound options page's items. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::menu::option_steps::{SOUNDS_B_RVA, Then, Volume, volume_step};
use crate::menu::owner::menus;

const STEP: Volume = Volume {
    value: SOUNDS_B_RVA,
    up: false,
    then: Then::Sound(0x40),
};

/// The Rust version the typed tables run.
pub fn run() {
    let screens = &menus().screens;
    volume_step(screens, STEP);
}
