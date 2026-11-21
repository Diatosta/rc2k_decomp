//! `menu::draw_frame`: draws a menu once, without the 3D scene and without
//! showing the frame: what a screen loop does when it has set a screen up.
//!
//! Makes the menu the current one (`loop_flags.menu`). Unless the game is quitting:
//! `copy_picture_to_surface` (the background), lock the surface, and if that worked `draw_menu`,
//! `run_draw_list` and unlock.

use crate::errors::Refused;
use crate::menu::frame_callback::{copy_picture, draw_menu};
use crate::owners::Owners;

/// Draws one frame of `menu`.
pub fn draw_frame(
    o: Owners<'static>,
    menu: crate::menu::menus_store::MenuId,
) -> Result<(), Refused> {
    let menus = o.menus;
    menus.screens.loop_flags.menu.set(menu);
    if crate::menu::screen_loop::quitting(o.system) {
        return Ok(());
    }
    copy_picture(o.gfx, o.system)?;
    crate::gfx::with_locked_surface::with_locked_surface(
        crate::gfx::with_locked_surface::lock(o.gfx, o.system),
        || {
            draw_menu(o, menu)?;
            crate::menu::run_draw_list::run_draw_list(o)
        },
    )?;
    Ok(())
}
