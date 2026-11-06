//! `scene::draw_main_menu_carousel` (0x49012C): the main menu's carousel, drawn the
//! plain way.
//!
//! In:  ecx, passed on
//! Out: as `scene::draw_carousel`
//!
//! `dword_6365C4` = 0 (the way `scene::draw_scene_object` draws by), then
//! `scene::draw_carousel`. One caller, `scene::draw_main_menu_objects`.

use crate::errors::Refused;
use crate::scene::draw_carousel::draw_carousel_in_game;
use crate::scene::owner::scene;

// ---- game data ----

// ---- Rust version ----

/// The scene's mode 0, then the carousel; `lone_row`: ecx at the original's entry (see
/// `draw_carousel`).
pub fn draw_main_menu_carousel(
    menus: &crate::menu::owner::Menus,
    race: &crate::race::owner::Race,
    lone_row: u32,
) -> Result<(), Refused> {
    scene().lights.state.draw_mode.set_bits(0u32);
    draw_carousel_in_game(menus, race, lone_row)
}
