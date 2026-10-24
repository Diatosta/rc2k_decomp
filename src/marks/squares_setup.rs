//! `marks::squares_setup` (0x4BFEAC): the squares' frame set up: `scene::depth_buffer_on`,
//! `scene::set_texture` (`dword_5D1E10`), render states 0x11 = 2, 0x12 = 2, 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1; four vertices of `g_polygonVertices` with rhw 0x100
//! (through `dword_62400C`), colour -1, specular 0xFF000000, texture places from
//! `dword_8BC212` .. (u a, c, c, a; v b, b, d, d); `g_polygonVertexCount` 4; `flt_89CEEA` 1
//! over `flt_6241C8`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BFFEC` (0x4C000F). The device calls through `part` (1 the depth buffer, 2
//! the texture, 3 a render state), in the game their Rust versions. Shadow mode runs the
//! original alone (it tells the device). Self-test: a copy whose calls go to stand-ins,
//! against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::rva;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const PLACES_RVA: Slot = Slot::at(0x8BC212); // dword_8BC212 .. dword_8BC21E
const DEPTH_OF_RVA: usize = rva(0x6241C8); // flt_6241C8
const INVERSE_RVA: Slot = Slot::at(0x89CEEA); // flt_89CEEA
const STATES: [(u32, u32); 7] = [
    (2, 0x11),
    (2, 0x12),
    (4, 0x15),
    (5, 0x13),
    (6, 0x14),
    (0, 0x1C),
    (1, 0x1B),
];

/// The Rust version (see the header).
pub unsafe fn squares_setup() {
    unsafe { setup_with(PLACES_RVA) }
}

/// The set-up with the texture places at `places` (`marks::pong_setup` has its own).
pub unsafe fn setup_with(places: Slot) {
    use crate::scene::set_render_state::set_render_state;
    unsafe {
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        for n in 0..4 {
            game_write(SCRATCH_RVA, 0x100u32);
            game_write(VERTICES_RVA + 0x20 * n + 0xC, x87::int_to_float(0x100));
        }
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, 0xFFFF_FFFFu32);
            game_write(VERTICES_RVA + 0x20 * n + 0x14, 0xFF00_0000u32);
        }
        let [a, b, c, d]: [u32; 4] = game_read(places);
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        game_write(COUNT_RVA, 4u32);
        game_write(INVERSE_RVA, x87::reciprocal(game_read(DEPTH_OF_RVA)));
    }
}
