//! `scene::choose_carousel_pictures`: on the menus whose carousel shows pictures, the four
//! entries that can be seen get theirs.
//!
//! Only with the carousel menu at 0xB ..= 0xE: the entry before the front one
//! (the carousel's front, going round its count) and the three after it are the four
//! entries of `scene::carousel_pictures::update`. One caller, `scene::draw_main_menu_objects`.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::scene::carousel_pictures::update;

// ---- game data ----

pub fn choose_carousel_pictures(
    menus: &Menus,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let screens = &menus.screens;
    // Logic: `rc2k_core::scene::carousel::{shows_pictures, visible_entries}`.
    use rc2k_core::scene::carousel::{shows_pictures, visible_entries};
    if !shows_pictures(screens.state.carousel.menu.get()) {
        return Ok(());
    }
    update(
        menus,
        scene,
        visible_entries(
            screens.state.carousel.front.get(),
            screens.state.carousel.count.get(),
        ),
    )
}
