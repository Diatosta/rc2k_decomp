//! `marks::hexagons_setup` (0x4BBE62): the hexagons' frame set up: their two axes
//! (`flt_814C1A`, `flt_814C26`) times `flt_814BC2` into `unk_814C6E` / `unk_814C7A` and those
//! times `flt_82E286` .. into `unk_814C86` / `unk_814C92`; `scene::depth_buffer_on`,
//! `scene::set_texture` (`dword_5D5A24`), render states 0x11 = 1, 0x12 = 1, 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1, 3 = 1; six vertices of `g_polygonVertices` with rhw 0x100
//! (through `dword_62400C`), colour -1, specular 0xFF000000, texture places 0 /
//! `dword_82E2A2` (u 0, a, a, 0; v 0, 0, a, a); `g_polygonVertexCount` 6; `flt_814C02` 1 over
//! `flt_6241C8`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BC0B1` (0x4BC0BF). The x87 run is the original's instructions over the
//! game's variables; the device calls through `part` (1 the depth buffer, 2 the texture,
//! 3 a render state), in the game their Rust versions. Shadow mode runs the original alone
//! (it tells the device). Self-test: a copy whose calls go to stand-ins, against the Rust
//! version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::rva;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

const TEXTURE_RVA: usize = rva(0x5D5A24); // dword_5D5A24
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const SIDE_RVA: Slot = Slot::at(0x82E2A2); // dword_82E2A2
const DEPTH_OF_RVA: usize = rva(0x6241C8); // flt_6241C8
const INVERSE_RVA: Slot = Slot::at(0x814C02); // flt_814C02
const STATES: [(u32, u32); 8] = [
    (1, 0x11),
    (1, 0x12),
    (4, 0x15),
    (5, 0x13),
    (6, 0x14),
    (0, 0x1C),
    (1, 0x1B),
    (1, 3),
];

/// The original's 0x4BBE62 .. 0x4BBF1A: the axes scaled.
unsafe fn axes() {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{mk} + 0xAFCDA]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD2E]",
            "fld dword ptr [{mk} + 0xAFCDE]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD32]",
            "fld dword ptr [{mk} + 0xAFCE2]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD36]",
            "fld dword ptr [{mk} + 0xAFCE6]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD3A]",
            "fld dword ptr [{mk} + 0xAFCEA]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD3E]",
            "fld dword ptr [{mk} + 0xAFCEE]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD42]",
            "fld dword ptr [{mk} + 0xAFD2E]",
            "fmul dword ptr [{mk} + 0xC9346]",
            "fstp dword ptr [{mk} + 0xAFD46]",
            "fld dword ptr [{mk} + 0xAFD32]",
            "fmul dword ptr [{mk} + 0xC934A]",
            "fstp dword ptr [{mk} + 0xAFD4A]",
            "fld dword ptr [{mk} + 0xAFD36]",
            "fmul dword ptr [{mk} + 0xC934E]",
            "fstp dword ptr [{mk} + 0xAFD4E]",
            "fld dword ptr [{mk} + 0xAFD3A]",
            "fmul dword ptr [{mk} + 0xC9352]",
            "fstp dword ptr [{mk} + 0xAFD52]",
            "fld dword ptr [{mk} + 0xAFD3E]",
            "fmul dword ptr [{mk} + 0xC9356]",
            "fstp dword ptr [{mk} + 0xAFD56]",
            "fld dword ptr [{mk} + 0xAFD42]",
            "fmul dword ptr [{mk} + 0xC935A]",
            "fstp dword ptr [{mk} + 0xAFD5A]",
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn hexagons_setup() {
    use crate::scene::set_render_state::set_render_state;
    unsafe {
        axes();
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        for n in 0..6 {
            game_write(SCRATCH_RVA, 0x100u32);
            game_write(VERTICES_RVA + 0x20 * n + 0xC, x87::int_to_float(0x100));
        }
        for n in 0..6 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, 0xFFFF_FFFFu32);
            game_write(VERTICES_RVA + 0x20 * n + 0x14, 0xFF00_0000u32);
        }
        let side: u32 = game_read(SIDE_RVA);
        for (n, (u, v)) in [(0, 0), (side, 0), (side, side), (0, side)]
            .into_iter()
            .enumerate()
        {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        game_write(COUNT_RVA, 6u32);
        game_write(INVERSE_RVA, x87::reciprocal(game_read(DEPTH_OF_RVA)));
    }
}
