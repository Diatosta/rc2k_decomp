//! `scene::choose_carousel_pictures` (0x4901B9): on the menus whose carousel shows
//! pictures, the four entries that can be seen get theirs.
//!
//! In:  nothing
//! Out: nothing changed on other menus; else as `scene::update_carousel_pictures`
//!
//! Only with `g_carouselMenu` at 0xB .. 0xE: the entry before the front one
//! (`g_carouselFront`, going round `g_carouselCount`) and the three after it are the
//! four entries of `scene::update_carousel_pictures`.
//! One caller, `scene::draw_main_menu_objects`.
//! Self-test: 80 settings by lot, with records of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::scene::carousel_pictures::update;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

const MENU_RVA: usize = rva(0x75AA74); // g_carouselMenu
const COUNT_RVA: usize = rva(0x75AA78); // g_carouselCount
const FRONT_RVA: usize = rva(0x75AA7C); // g_carouselFront

// ---- Rust version ----

pub unsafe fn choose_carousel_pictures() -> Result<Left, Refused> {
    // Converted: `rc2k_core::scene::carousel::{shows_pictures, visible_entries}`.
    use rc2k_core::scene::carousel::{shows_pictures, visible_entries};
    unsafe {
        if !shows_pictures(game_read(MENU_RVA)) {
            return Ok(Left::nothing());
        }
        Ok(update(visible_entries(
            game_read(FRONT_RVA),
            game_read(COUNT_RVA),
        ))?)
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { choose_carousel_pictures() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
