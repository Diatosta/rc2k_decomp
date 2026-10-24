//! `marks::set_quad_uvs` (0x4CFD6A): writing to the depth buffer off, and the four texture
//! places of the 0x20-byte set at `dword_920738` + 0xC60 put into the four vertices'
//! texture places at `flt_9605B8` (0x20 bytes apart: +0x18 of Direct3D vertices from
//! 0x9605A0, a reading).
//!
//! In:  nothing
//! Out: nothing (ebp, the last place's v, is read only by the original `jitter_point`
//!      under `trail_particles`; the Rust version reads no register)
//!
//! One caller, `sub_4CFB49` (the marks' frame). Self-test: the original and the Rust
//! version recorded on the layer (nothing drawn), the set filled by a pattern; put back
//! afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, game_write, rva};

const SETS_RVA: usize = rva(0x920738); // dword_920738: sets of four texture places
const SET: u32 = 0xC60;
const VERTICES_RVA: usize = rva(0x9605B8); // flt_9605B8: the first vertex's u, v
const VERTEX: usize = 0x20;

/// The Rust version (see the header).
pub unsafe fn set_quad_uvs() {
    unsafe {
        crate::scene::depth_test_off::depth_test_off();
        for n in 0..4 {
            let from = SETS_RVA + SET as usize + 8 * n;
            let uv: [u32; 2] = [game_read(from), game_read(from + 4)];
            game_write(VERTICES_RVA + VERTEX * n, uv);
        }
    }
}
