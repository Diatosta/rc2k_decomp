//! `scene::clear_depth_buffer` (0x44E967): clears the depth buffer over the whole
//! screen.
//!
//! In:  nothing
//! Out: nothing changed without 3D; else eax, ecx, edx as Direct3D leaves them and
//!      ebx = the screen's height
//!
//! Only with `dword_5CDA8C` not 0: the rectangle `dword_624138` .. = 0, 0, the
//! screen's width and height, and `d3d::clear_viewport` (0x44E9D9: the viewport's
//! Clear with that one rectangle and flag 2, the depth buffer; Direct3D, not ported).
//! Three callers.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose call notes the rectangle.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

// ---- game data ----

const HAS_3D_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const RECTANGLE_RVA: usize = rva(0x624138); // dword_624138 .. dword_624144
const WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const CLEAR_RVA: usize = rva(0x44E9D9); // d3d::clear_viewport

// ---- Rust version ----

/// `clear`: what stands for `d3d::clear_viewport`.
pub unsafe fn clear_depth_buffer(r: &mut Regs, clear: &mut dyn FnMut(&mut Regs)) {
    unsafe {
        if game_read::<u32>(HAS_3D_RVA) == 0 {
            return;
        }
        r.eax = game_read(WIDTH_RVA);
        r.ebx = game_read(HEIGHT_RVA);
        game_write(RECTANGLE_RVA, [0, 0, r.eax, r.ebx]);
        clear(r);
    }
}

pub unsafe fn clear_depth_buffer_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe { clear_depth_buffer(r, &mut |r| crate::calls::call(CLEAR_RVA, r)) };
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { clear_depth_buffer(r, &mut |r| crate::calls::call(CLEAR_RVA, r)) };
    Ok(())
}
