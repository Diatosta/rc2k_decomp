//! `menu::music_volume_up` (0x467F71): the music's volume one more (see `menu/option_steps.rs`).
//!
//! A left/right handler of the sound options page's items. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::menu::option_steps::{MUSIC_LEVEL_RVA, Then, Volume, volume_step};

const STEP: Volume = Volume {
    value: MUSIC_LEVEL_RVA,
    up: true,
    then: Then::Music,
};

/// The Rust version the typed tables run.
pub fn run(menus: &crate::menu::owner::Menus) {
    let screens = &menus.screens;
    volume_step(screens, STEP);
}
