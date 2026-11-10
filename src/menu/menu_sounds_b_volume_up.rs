//! `menu::menu_sounds_b_volume_up` (0x46810A): the second menu sounds' volume one more, heard (see `menu/option_steps.rs`).
//!
//! A left/right handler of the sound options page's items. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::menu::option_steps::{SOUNDS_B_RVA, Then, Volume, volume_step};

const STEP: Volume = Volume {
    value: SOUNDS_B_RVA,
    up: true,
    then: Then::Sound(0x40),
};

pub fn run(menus: &crate::menu::owner::Menus) {
    let screens = &menus.screens;
    volume_step(screens, STEP);
}
