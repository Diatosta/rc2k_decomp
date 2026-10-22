//! `menu::frame_callback` (0x474D4B): one frame of a menu screen, the callback nearly every
//! screen loop hands to `screens::run_frame`.
//!
//! In:  edi = the menu; ebx, ecx as they come (`draw_menu` uses them)
//! Out: carry = the frame could not be made; every register preserved
//! In this order: lock the surface; `copy_picture_to_surface` (the background);
//! `draw_menu`; every function of the draw list (`g_drawList`); unlock; `dword_6365CC` =
//! 4; begin the 3D scene (`sub_44E8A1`); every function of the scene list (`g_sceneList`),
//! each between `pusha` and `popa`; end the scene (`sub_44E952`); lock again; every
//! function of the overlay list (`g_overlayList`); if `byte_4E07A0` is 2 and
//! `dword_7585D8` is 1, clear that and draw string 0x1000 at (800, 587) with the font
//! 0x51E404; unlock; show the frame (`sub_4057D1`). A set carry after either lock, the
//! scene's beginning or the showing ends it there, with what was locked left locked.
//! The functions of the draw and overlay lists get the registers as the one before left
//! them; a list ends with -1.
//! The surface's lock and unlock, the 3D scene and the showing of the frame are the game's
//! (DirectDraw and Direct3D calls), reached like everything else here through an `Env`:
//! that is where a build without the game puts its own.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::screen_loop::{Env, Real};
use rc2k_core::menu::notice;

// ---- game data ----

pub const LOCK_SURFACE: usize = rva(0x4058AD); // gfx::lock_surface
pub const UNLOCK_SURFACE: usize = rva(0x405949); // gfx::unlock_surface
pub const COPY_PICTURE: usize = rva(0x4060E3); // gfx::copy_picture_to_surface
pub const DRAW_MENU: usize = rva(0x4746B7); // menu::draw_menu
const BEGIN_SCENE: usize = rva(0x44E8A1); // sub_44E8A1
const END_SCENE: usize = rva(0x44E952); // sub_44E952
const SHOW_FRAME: usize = rva(0x4057D1); // sub_4057D1
const DRAW_STRING_BY_ID: usize = rva(0x40234A); // text::draw_string_by_id
const DRAW_LIST_RVA: usize = rva(0x7554F8); // g_drawList
const SCENE_LIST_RVA: usize = rva(0x755504); // g_sceneList
const OVERLAY_LIST_RVA: usize = rva(0x7554FC); // g_overlayList
const SCENE_STATE_RVA: usize = rva(0x6365CC); // dword_6365CC
const NOTICE_KIND_RVA: usize = rva(0x4E07A0); // byte_4E07A0: 2 = the notice below may be drawn
const NOTICE_DUE_RVA: usize = rva(0x7585D8); // dword_7585D8: set to 1 by the main menu before a frame
const NOTICE_FONT_RVA: usize = rva(0x51E404); // stru_51E404
const END: u32 = u32::MAX;

// ---- Rust version ----

/// Calls the functions of the list esi points at, one after the other; esi moves along.
/// `each_alone`: every function gets the registers as they are before it (the scene list's
/// `pusha` and `popa`), not as the one before left them.
unsafe fn run_listed(g: &mut Regs, env: &mut dyn Env, each_alone: bool) {
    unsafe {
        while (g.esi as *const u32).read_unaligned() != END {
            let function = (g.esi as *const u32).read_unaligned() as usize;
            let rva = function.wrapping_sub(game_addr(0));
            if each_alone {
                let mut own = *g;
                env.call(rva, &mut own);
                g.eflags = own.eflags;
            } else {
                env.call(rva, g);
            }
            g.esi = g.esi.wrapping_add(4);
        }
    }
}

/// `entry` are the registers at entry. Returns whether the frame was made.
pub unsafe fn frame_callback(entry: &Regs, env: &mut dyn Env) -> bool {
    unsafe {
        let mut g = *entry;
        env.call(LOCK_SURFACE, &mut g);
        if g.carry() {
            return false;
        }
        env.call(COPY_PICTURE, &mut g);
        env.call(DRAW_MENU, &mut g);
        g.esi = game_read(DRAW_LIST_RVA);
        run_listed(&mut g, env, false);
        env.call(UNLOCK_SURFACE, &mut g);
        game_write(SCENE_STATE_RVA, 4u32);
        env.call(BEGIN_SCENE, &mut g);
        if g.carry() {
            return false;
        }
        g.esi = game_read(SCENE_LIST_RVA);
        run_listed(&mut g, env, true);
        env.call(END_SCENE, &mut g);
        env.call(LOCK_SURFACE, &mut g);
        if g.carry() {
            return false;
        }
        g.esi = game_read(OVERLAY_LIST_RVA);
        run_listed(&mut g, env, false);
        // Converted: `rc2k_core::menu::notice::drawn_now`.
        if notice::drawn_now(game_read(NOTICE_KIND_RVA), || game_read(NOTICE_DUE_RVA)) {
            game_write(NOTICE_DUE_RVA, 0u32);
            g.eax = notice::STRING;
            g.ebx = game_addr(NOTICE_FONT_RVA) as u32;
            (g.ecx, g.edx) = notice::AT;
            env.call(DRAW_STRING_BY_ID, &mut g);
        }
        env.call(UNLOCK_SURFACE, &mut g);
        env.call(SHOW_FRAME, &mut g);
        !g.carry()
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    let made = unsafe { frame_callback(&entry, &mut Real) };
    r.set_carry(!made);
    Ok(())
}

// ---- self-test: a copy of the original whose ten calls go to stand-ins that note them,
// with lists of the test's made of more such stand-ins ----
