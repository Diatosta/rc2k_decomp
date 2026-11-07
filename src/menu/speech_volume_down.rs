//! `menu::speech_volume_down` (0x46802A): the speech's volume one less, heard (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: as `option_steps` says (not `pusha`'d)
//! A left/right handler of the sound options page's items. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::menu::option_steps::{SPEECH_LEVEL_RVA, Then, Volume, volume_step};

const STEP: Volume = Volume {
    value: SPEECH_LEVEL_RVA,
    up: false,
    then: Then::Speech,
};

/// The Rust version the typed tables run.
pub fn run(menus: &crate::menu::owner::Menus) {
    let screens = &menus.screens;
    volume_step(screens, STEP);
}
