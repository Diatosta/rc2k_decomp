//! `scene::choose_carousel_pictures`: on the menus whose carousel shows pictures, the four
//! entries that can be seen get theirs.
//!
//! Only with the carousel menu at 0xB ..= 0xE: the entry before the front one (the carousel's
//! front, going round its count) and the three after it are the four entries of
//! `scene::carousel_pictures::update`.

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
    if !shows_pictures(screens.carousel.menu.get()) {
        return Ok(());
    }
    update(
        menus,
        scene,
        visible_entries(screens.carousel.front.get(), screens.carousel.count.get()),
    )
}
