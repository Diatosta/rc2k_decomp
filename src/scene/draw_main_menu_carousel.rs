//! `scene::draw_main_menu_carousel`: the main menu's carousel, drawn the plain way:
//! `LightState.draw_mode` = 0 (the way `scene::draw_scene_object` draws by), then
//! `scene::draw_carousel`.

use crate::errors::Refused;
use crate::scene::draw_carousel::draw_carousel;

// ---- game data ----

/// The scene's mode 0, then the carousel; `lone_row`: see `draw_carousel`.
pub fn draw_main_menu_carousel(
    menus: &crate::menu::owner::Menus,
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    lone_row: u32,
) -> Result<(), Refused> {
    let scene = ctx.scene;
    scene.lights.state.get().draw_mode.set(0u32);
    draw_carousel(menus, ctx, lone_row)
}
