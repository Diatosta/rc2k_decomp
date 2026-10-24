//! `scene::draw_main_menu_carousel` (0x49012C): the main menu's carousel, drawn the
//! plain way.
//!
//! In:  ecx, passed on
//! Out: as `scene::draw_carousel`
//!
//! `dword_6365C4` = 0 (the way `scene::draw_scene_object` draws by), then
//! `scene::draw_carousel`. One caller, `scene::draw_main_menu_objects`.

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
