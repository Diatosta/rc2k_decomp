//! `scene::draw_main_menu_carousel` (0x49012C): the main menu's carousel, drawn the
//! plain way.
//!
//! Out: as `scene::draw_carousel`
//!
//! `dword_6365C4` = 0 (the way `scene::draw_scene_object` draws by), then
//! `scene::draw_carousel`. One caller, `scene::draw_main_menu_objects`.

use crate::errors::Refused;
use crate::scene::draw_carousel::draw_carousel_in_game;

// ---- game data ----

// ---- Rust version ----

/// The scene's mode 0, then the carousel; `lone_row`: ecx at the original's entry (see
/// `draw_carousel`).
pub fn draw_main_menu_carousel(
    menus: &crate::menu::owner::Menus,
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    lone_row: u32,
) -> Result<(), Refused> {
    let scene = ctx.scene;
    scene.lights.state.draw_mode.set(0u32);
    draw_carousel_in_game(menus, ctx, lone_row)
}
