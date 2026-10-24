//! `scene::draw_main_menu_carousel` (0x49012C): the main menu's carousel, drawn the
//! plain way.
//!
//! In:  ecx, passed on
//! Out: as `scene::draw_carousel`
//!
//! `dword_6365C4` = 0 (the way `scene::draw_scene_object` draws by), then
//! `scene::draw_carousel`.
//! One caller, `scene::draw_main_menu_objects`.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose call notes the variable.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_write, rva};
use crate::scene::draw_carousel::draw_carousel_in_game;

// ---- game data ----

const MODE_RVA: usize = rva(0x6365C4); // dword_6365C4

// ---- Rust version ----

/// The scene's mode 0, then the carousel; `lone_row`: ecx at the original's entry (see
/// `draw_carousel`).
pub unsafe fn draw_main_menu_carousel(lone_row: u32) -> Result<(), Refused> {
    unsafe {
        game_write(MODE_RVA, 0u32);
        draw_carousel_in_game(lone_row)
    }
}
