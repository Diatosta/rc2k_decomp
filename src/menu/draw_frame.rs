//! `menu::draw_frame` (0x47C420): draws a menu once, without the 3D scene and without
//! showing the frame: what a screen loop does when it has set a screen up.
//!
//! In:  edi = the menu
//! Out: every register preserved
//! Makes the menu the current one (`dword_75558D`). Unless the game is quitting:
//! `copy_picture_to_surface` (the background), lock the surface, and if that worked `draw_menu`,
//! `run_draw_list` and unlock. 89 callers, the screen loops. Rust callers run its `entry`
//! (`routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::frame_callback::{copy_picture, draw_menu, lock_failed};
use crate::menu::owner::Menus;

// ---- Rust version ----

/// `menu` is edi at entry.
pub fn draw_frame(menus: &Menus, menu: crate::menu::menus_store::MenuId) -> Result<(), Refused> {
    menus.screens.loop_flags.menu.set(menu);
    if crate::menu::screen_loop::quitting() {
        return Ok(());
    }
    copy_picture()?;
    if lock_failed() {
        return Ok(());
    }
    draw_menu(menu)?;
    crate::menu::run_draw_list::run_draw_list(menus)?;
    crate::platform::unlock_surface();
    Ok(())
}
