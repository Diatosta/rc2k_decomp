//! `menu::draw_frame` (0x47C420): draws a menu once, without the 3D scene and without
//! showing the frame: what a screen loop does when it has set a screen up.
//!
//! In:  edi = the menu; ebx, ecx as they come (`draw_menu` uses them)
//! Out: every register preserved
//! Makes the menu the current one (`dword_75558D`). Unless the game is quitting:
//! `copy_picture_to_surface` (the background), lock the surface, and if that worked
//! `draw_menu`, `run_draw_list` and unlock.
//! 89 callers, the screen loops. The lock and the unlock are the game's, through an `Env`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_write, rva};
use crate::menu::frame_callback::{COPY_PICTURE, DRAW_MENU, LOCK_SURFACE, UNLOCK_SURFACE};
use crate::menu::item_at::CURRENT_MENU_RVA;
use crate::menu::screen_loop::{Env, Real};

const RUN_DRAW_LIST: usize = rva(0x4681ED); // menu::run_draw_list

// ---- Rust version ----

/// `entry` are the registers at entry.
pub unsafe fn draw_frame(entry: &Regs, env: &mut dyn Env) {
    unsafe {
        game_write(CURRENT_MENU_RVA, entry.edi);
        if env.quitting() {
            return;
        }
        let mut g = *entry;
        env.call(COPY_PICTURE, &mut g);
        env.call(LOCK_SURFACE, &mut g);
        if g.carry() {
            return;
        }
        env.call(DRAW_MENU, &mut g);
        env.call(RUN_DRAW_LIST, &mut g);
        env.call(UNLOCK_SURFACE, &mut g);
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { draw_frame(&entry, &mut Real) };
    Ok(())
}

// ---- self-test: a copy of the original whose five calls go to stand-ins that note them,
// and which reads the quit switch from a test variable ----
