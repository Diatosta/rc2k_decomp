//! `d3d::draw_marks_hexagons` (0x41ADB9): draws the marks' hexagons, an indexed list of
//! triangles.
//!
//! In:  nothing: the vertices at `unk_625A44`, their number in `dword_625A40`, the indices
//!      (16 bits) at `g_marksHexagonFans`, their number in `dword_62423C`
//! Out: eax = Direct3D's result (0: drawn); ecx, edx as Direct3D leaves them
//! The device's (`g_d3dDevice`, an `IDirect3DDevice2`) DrawIndexedPrimitive (+0x78) with
//! a triangle list (4) of transformed and lit vertices (3) and no flags. Callers:
//! `marks::hexagons_top`, `marks::hexagons_frame`.
//! Platform: the original always runs. In the standalone program the rasteriser draws
//! each triangle (`platform_3d::draw_indexed_list`) and the result is 0 (before 2026-10-19
//! the standalone skipped it). No self-test.

use crate::framework::{RawHook, Regs, hook_asm, rva, standalone};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("d3d::draw_marks_hexagons", rva(0x41ADB9), thunk, &ORIGINAL);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

#[cfg(feature = "standalone")]
const VERTICES_RVA: usize = rva(0x625A44); // unk_625A44
#[cfg(feature = "standalone")]
const VERTEX_COUNT_RVA: usize = rva(0x625A40); // dword_625A40
#[cfg(feature = "standalone")]
const INDICES_RVA: usize = rva(0x624240); // g_marksHexagonFans
#[cfg(feature = "standalone")]
const INDEX_COUNT_RVA: usize = rva(0x62423C); // dword_62423C

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if standalone() {
        #[cfg(feature = "standalone")]
        unsafe {
            use crate::framework::{game_addr, game_read};
            crate::platform_3d::draw_indexed_list(
                game_addr(VERTICES_RVA) as u32,
                game_read(VERTEX_COUNT_RVA),
                game_addr(INDICES_RVA) as u32,
                game_read(INDEX_COUNT_RVA),
            )
        };
        r.eax = 0;
        return;
    }
    unsafe { call_original(r) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if standalone() {
        #[cfg(feature = "standalone")]
        unsafe {
            use crate::framework::{game_addr, game_read};
            crate::platform_3d::draw_indexed_list(
                game_addr(VERTICES_RVA) as u32,
                game_read(VERTEX_COUNT_RVA),
                game_addr(INDICES_RVA) as u32,
                game_read(INDEX_COUNT_RVA),
            )
        };
        r.eax = 0;
        return Ok(());
    }
    unsafe { crate::framework::call_game(r, crate::framework::game_addr(rva(0x41ADB9))) };
    Ok(())
}
