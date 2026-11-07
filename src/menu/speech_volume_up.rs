//! `menu::speech_volume_up` (0x468055): the speech's volume one more, heard (see `menu/option_steps.rs`).
//!
//! A left/right handler of the sound options page's items. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::menu::option_steps::{SPEECH_LEVEL_RVA, Then, Volume, volume_step};

const STEP: Volume = Volume {
    value: SPEECH_LEVEL_RVA,
    up: true,
    then: Then::Speech,
};

/// The Rust version the typed tables run.
pub fn run(menus: &crate::menu::owner::Menus) {
    let screens = &menus.screens;
    volume_step(screens, STEP);
}
