//! `menu::music_volume_down` (0x467F48): the music's volume one less (see `menu/option_steps.rs`).
//!
//! A left/right handler of the sound options page's items. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::menu::option_steps::{MUSIC_LEVEL_RVA, Then, Volume, volume_step};

const STEP: Volume = Volume {
    value: MUSIC_LEVEL_RVA,
    up: false,
    then: Then::Music,
};

/// The Rust version the typed tables run.
pub fn run(menus: &crate::menu::owner::Menus) {
    let screens = &menus.screens;
    volume_step(screens, STEP);
}
