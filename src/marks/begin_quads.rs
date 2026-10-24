//! `marks::begin_quads` (0x4C4453): the render set up for the marks' quads and a quad's
//! four vertices made ready. `scene::set_texture` with `dword_5D1E10`,
//! `scene::depth_buffer_on`, `scene::depth_test_off`, render states 0x15 = 4, 0x13 = 5,
//! 0x14 = 6, 0x1C = 0, 0x1B = 1 (`scene::set_render_state`); then each of the four
//! `g_polygonVertices` gets +0xC `dword_91FFA6`, +0x10 -1, +0x14 0, and its texture place
//! from `dword_91FFAA` .. `dword_91FFB6` (u: AA, B2, B2, AA; v: AE, AE, B6, B6);
//! `g_polygonVertexCount` 4, `flt_8EB96D` 1 / `flt_6241C8`, `dword_91FFBA` 0.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C3EEE`. Its calls through `part` (places 1 .. 4: the texture, the depth
//! buffer, the depth test, a render state); in the game their Rust versions (the
//! device's calls through their hooks). Shadow mode runs the original alone (it tells the
//! device). Self-test: a copy whose eight calls go to stand-ins, against the Rust version
//! on the same stand-ins; the vertices, the globals and the calls compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::rva;
use crate::marks::store::{Slot, game_read, game_write};

const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices: 0x20 bytes each
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const COLOUR_RVA: Slot = Slot::at(0x91FFA6); // dword_91FFA6
const PLACES_RVA: Slot = Slot::at(0x91FFAA); // dword_91FFAA .. dword_91FFB6
const SCALE_RVA: Slot = Slot::at(0x8EB96D); // flt_8EB96D
const OF_RVA: usize = rva(0x6241C8); // flt_6241C8
const CLEARED_RVA: Slot = Slot::at(0x91FFBA); // dword_91FFBA
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// The Rust version (see the header): `scene::set_texture`, `scene::depth_buffer_on`,
/// `scene::depth_test_off`, the render states (`scene::set_render_state`).
pub unsafe fn begin_quads() {
    unsafe {
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::depth_test_off::depth_test_off();
        for (value, state) in STATES {
            crate::scene::set_render_state::set_render_state(value, state);
        }
        let colour: u32 = game_read(COLOUR_RVA);
        let [a, b, c, d]: [u32; 4] = game_read(PLACES_RVA);
        let uv = [(a, b), (c, b), (c, d), (a, d)];
        for (k, (u, v)) in uv.into_iter().enumerate() {
            let at = VERTICES_RVA + 0x20 * k;
            game_write(at + 0xC, colour);
            game_write(at + 0x10, 0xFFFF_FFFFu32);
            game_write(at + 0x14, 0u32);
            game_write(at + 0x18, u);
            game_write(at + 0x1C, v);
        }
        game_write(COUNT_RVA, 4u32);
        game_write(SCALE_RVA, crate::math::x87::reciprocal(game_read(OF_RVA)));
        game_write(CLEARED_RVA, 0u32);
    }
}
