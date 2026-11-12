//! `menu::draw_frame` (0x47C420): draws a menu once, without the 3D scene and without
//! showing the frame: what a screen loop does when it has set a screen up.
//!
//! Makes the menu the current one (`loop_flags.menu`). Unless the game is quitting:
//! `copy_picture_to_surface` (the background), lock the surface, and if that worked `draw_menu`,
//! `run_draw_list` and unlock. 89 callers, the screen loops. Rust callers run its `entry`
//! (`routes.rs`).

use crate::errors::Refused;
use crate::menu::frame_callback::{copy_picture, draw_menu, lock_failed};
use crate::owners::Owners;

/// `menu` is edi at entry.
pub fn draw_frame(o: Owners<'_>, menu: crate::menu::menus_store::MenuId) -> Result<(), Refused> {
    let menus = o.menus;
    menus.screens.loop_flags.menu.set(menu);
    if crate::menu::screen_loop::quitting() {
        return Ok(());
    }
    copy_picture()?;
    if lock_failed() {
        return Ok(());
    }
    draw_menu(menus, menu)?;
    crate::menu::run_draw_list::run_draw_list(o)?;
    crate::platform::unlock_surface();
    Ok(())
}
