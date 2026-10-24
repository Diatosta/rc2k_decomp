//! `scene::choose_carousel_pictures` (0x4901B9): on the menus whose carousel shows
//! pictures, the four entries that can be seen get theirs.
//!
//! In:  nothing
//! Out: nothing changed on other menus; else as `scene::update_carousel_pictures`
//!
//! Only with `g_carouselMenu` at 0xB .. 0xE: the entry before the front one (`g_carouselFront`,
//! going round `g_carouselCount`) and the three after it are the four entries of
//! `scene::update_carousel_pictures`. One caller, `scene::draw_main_menu_objects`.

use crate::errors::BadInput;
use crate::framework::{game_read, rva};
use crate::scene::carousel_pictures::update;

// ---- game data ----

const MENU_RVA: usize = rva(0x75AA74); // g_carouselMenu
const COUNT_RVA: usize = rva(0x75AA78); // g_carouselCount
const FRONT_RVA: usize = rva(0x75AA7C); // g_carouselFront

// ---- Rust version ----

pub unsafe fn choose_carousel_pictures() -> Result<(), BadInput> {
    // Converted: `rc2k_core::scene::carousel::{shows_pictures, visible_entries}`.
    use rc2k_core::scene::carousel::{shows_pictures, visible_entries};
    unsafe {
        if !shows_pictures(game_read(MENU_RVA)) {
            return Ok(());
        }
        update(visible_entries(game_read(FRONT_RVA), game_read(COUNT_RVA)))
    }
}
