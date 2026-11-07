//! `scene::choose_carousel_pictures` (0x4901B9): on the menus whose carousel shows
//! pictures, the four entries that can be seen get theirs.
//!
//! Only with `g_carouselMenu` at 0xB .. 0xE: the entry before the front one (`g_carouselFront`,
//! going round `g_carouselCount`) and the three after it are the four entries of
//! `scene::update_carousel_pictures`. One caller, `scene::draw_main_menu_objects`.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::scene::carousel_pictures::update;

// ---- game data ----

// ---- Rust version ----

pub fn choose_carousel_pictures(menus: &Menus) -> Result<(), BadInput> {
    let screens = &menus.screens;
    // Converted: `rc2k_core::scene::carousel::{shows_pictures, visible_entries}`.
    use rc2k_core::scene::carousel::{shows_pictures, visible_entries};
    if !shows_pictures(screens.state.carousel.menu.get()) {
        return Ok(());
    }
    update(
        menus,
        visible_entries(
            screens.state.carousel.front.get(),
            screens.state.carousel.count.get(),
        ),
    )
}
