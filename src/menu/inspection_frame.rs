//! `menu::inspection_frame` (0x46D019): one frame of the inspection area after its first
//! (`menu::inspection_area_loop`'s later frame callback): `menu::frame_callback`'s steps in
//! another order and without its notice: lock the surface; the background
//! (`copy_picture_to_surface`); the draw list (`g_drawList`); unlock; `dword_6365CC` 4;
//! begin the 3D scene; the scene list (`g_sceneList`, each between `pusha` and `popa`);
//! end the scene; lock again; `draw_menu`; the overlay list (`g_overlayList`); unlock;
//! show the frame. A set carry after either lock, the scene's beginning or the showing ends
//! it there (carry set), with what was locked left locked; else the carry is cleared.
//!
//! In:  edi = the menu; ebx, ecx as they come (`draw_menu` uses them)
//! Out: carry = the frame could not be made; every register kept (`pusha` .. `popa`)
//! The draw and overlay lists' functions get the registers as the one before left them.
//! The surface, the scene and the showing are the game's (DirectDraw and Direct3D),
//! reached through an `Env` like `frame_callback`'s. Shadow mode runs the original alone (a
//! frame cannot be shown twice). Manual patch with a private copy (0x7F bytes, nine calls
//! relocated): the first call starts at the second byte. Self-test: a copy whose calls go to
//! `frame_test`'s stand-ins, with lists of the test's, each call failing in turn.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::frame_callback::{COPY_PICTURE, DRAW_MENU, LOCK_SURFACE, UNLOCK_SURFACE};
use crate::menu::screen_loop::{Env, Real};

const BEGIN_SCENE: usize = rva(0x44E8A1); // scene::begin_scene
const END_SCENE: usize = rva(0x44E952); // scene::end_scene
const SHOW_FRAME: usize = rva(0x4057D1); // gfx::show_frame
const DRAW_LIST_RVA: usize = rva(0x7554F8); // g_drawList
const SCENE_LIST_RVA: usize = rva(0x755504); // g_sceneList
const OVERLAY_LIST_RVA: usize = rva(0x7554FC); // g_overlayList
const SCENE_STATE_RVA: usize = rva(0x6365CC); // dword_6365CC
const END: u32 = u32::MAX;
/// The functions of the list esi points at; `each_alone` as `frame_callback`'s.
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

/// The Rust version: whether the frame was made.
pub unsafe fn inspection_frame(entry: &Regs, env: &mut dyn Env) -> bool {
    unsafe {
        let mut g = *entry;
        env.call(LOCK_SURFACE, &mut g);
        if g.carry() {
            return false;
        }
        env.call(COPY_PICTURE, &mut g);
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
        env.call(DRAW_MENU, &mut g);
        g.esi = game_read(OVERLAY_LIST_RVA);
        run_listed(&mut g, env, false);
        env.call(UNLOCK_SURFACE, &mut g);
        env.call(SHOW_FRAME, &mut g);
        !g.carry()
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    let made = unsafe { inspection_frame(&entry, &mut Real) };
    r.set_carry(!made);
    Ok(())
}
