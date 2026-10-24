//! `marks::draw_square` (0x4BF80F): one square item drawn on the squares' frame. The four
//! vertices' texture places from `dword_8BC1AE` .. (u a, c, c, a; v b, b, d, d) plus the
//! item's +0xC / +0x10; its place (st(1) the across share times `flt_89CEEE`, st(0) the
//! along share times `flt_89CEF2`) along the two axes from the frame's place into
//! `flt_89CF92` ..; less `flt_89CF56` .. and `flt_89CF62` ..; the four corners (plus
//! `flt_89CF4A` .., `flt_89CF3E` .., less `flt_89CF4A` ..) into `flt_89CF92` .. `flt_89CFBE`;
//! unless the first corner's depth is at or below 4096.0 (a signed compare of the bits):
//! projected (`flt_624088` over each depth, the centre `flt_62409C` / `flt_6240A0`), depth
//! (plus `g_marksKindFloat`, times `flt_89CEEA`), colour edx, `scene::draw_fan`.
//!
//! In:  st(0) the along share, st(1) the across share (popped), esi the item, edx the
//!      colour
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BFFEC` (0x4C00FE). The x87 runs are the original's instructions over the
//! game's variables; the draw through `part` (1 `draw_fan`), in the game its Rust version.
//! Thunk by `two_in_hook!`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose draw goes to a stand-in, against the Rust version, items near and far.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::rva;
use crate::marks::store::{Slot, game_addr, game_read, game_write};

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const PLACES_RVA: Slot = Slot::at(0x8BC1AE); // dword_8BC1AE .. dword_8BC1BA
const CORNERS_RVA: Slot = Slot::at(0x89CF92); // flt_89CF92 .. flt_89CFBE: four of 12
const FAR_ENOUGH: i32 = 0x4580_0000; // 4096.0

/// The original's 0x4BF845 .. 0x4BF88D: the item's texture shift added to the places.
unsafe fn shifted_places(s: u32, p: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x18]",
            "fstp dword ptr [{p} + 0x18]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x38]",
            "fstp dword ptr [{p} + 0x38]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x58]",
            "fstp dword ptr [{p} + 0x58]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x78]",
            "fstp dword ptr [{p} + 0x78]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x1C]",
            "fstp dword ptr [{p} + 0x1C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x3C]",
            "fstp dword ptr [{p} + 0x3C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x5C]",
            "fstp dword ptr [{p} + 0x5C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x7C]",
            "fstp dword ptr [{p} + 0x7C]",
            s = in(reg) s,
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF88E .. 0x4BF9AA on the two values: the corners.
unsafe fn corners(v: &[[u8; 10]; 2]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{v} + 10]",
            "fld tbyte ptr [{v}]",
            "fmul dword ptr [{mk} + 0x137FB2]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x137FAE]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FDA]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FE6]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FB6]",
            "fstp dword ptr [{mk} + 0x138052]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FDE]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FEA]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FBA]",
            "fstp dword ptr [{mk} + 0x138056]",
            "fmul dword ptr [{mk} + 0x137FEE]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x137FE2]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FBE]",
            "fstp dword ptr [{mk} + 0x13805A]",
            "fld dword ptr [{mk} + 0x138052]",
            "fsub dword ptr [{mk} + 0x138016]",
            "fsub dword ptr [{mk} + 0x138022]",
            "fst dword ptr [{mk} + 0x138052]",
            "fadd dword ptr [{mk} + 0x13800A]",
            "fst dword ptr [{mk} + 0x13805E]",
            "fadd dword ptr [{mk} + 0x137FFE]",
            "fst dword ptr [{mk} + 0x13806A]",
            "fsub dword ptr [{mk} + 0x13800A]",
            "fstp dword ptr [{mk} + 0x138076]",
            "fld dword ptr [{mk} + 0x138056]",
            "fsub dword ptr [{mk} + 0x13801A]",
            "fsub dword ptr [{mk} + 0x138026]",
            "fst dword ptr [{mk} + 0x138056]",
            "fadd dword ptr [{mk} + 0x13800E]",
            "fst dword ptr [{mk} + 0x138062]",
            "fadd dword ptr [{mk} + 0x138002]",
            "fst dword ptr [{mk} + 0x13806E]",
            "fsub dword ptr [{mk} + 0x13800E]",
            "fstp dword ptr [{mk} + 0x13807A]",
            "fld dword ptr [{mk} + 0x13805A]",
            "fsub dword ptr [{mk} + 0x13801E]",
            "fsub dword ptr [{mk} + 0x13802A]",
            "fst dword ptr [{mk} + 0x13805A]",
            "fadd dword ptr [{mk} + 0x138012]",
            "fst dword ptr [{mk} + 0x138066]",
            "fadd dword ptr [{mk} + 0x138006]",
            "fst dword ptr [{mk} + 0x138072]",
            "fsub dword ptr [{mk} + 0x138012]",
            "fstp dword ptr [{mk} + 0x13807E]",
            mk = in(reg) crate::marks::store::base(),
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF9BA .. 0x4BFAD6: the corners projected, their depth.
unsafe fn projected(b: u32, p: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x13807E]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x138072]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x138066]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x13805A]",
            "fld dword ptr [{mk} + 0x138056]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x138052]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x4]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x0]",
            "fld dword ptr [{mk} + 0x138062]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x13805E]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x24]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x20]",
            "fld dword ptr [{mk} + 0x13806E]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x13806A]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x44]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x40]",
            "fld dword ptr [{mk} + 0x13807A]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x138076]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x64]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x60]",
            "fld dword ptr [{mk} + 0x13805A]",
            "fadd dword ptr [{mk} + 0x35C]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x138066]",
            "fadd dword ptr [{mk} + 0x35C]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x138072]",
            "fadd dword ptr [{mk} + 0x35C]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x13807E]",
            "fadd dword ptr [{mk} + 0x35C]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fstp dword ptr [{p} + 0x68]",
            "fstp dword ptr [{p} + 0x48]",
            "fstp dword ptr [{p} + 0x28]",
            "fstp dword ptr [{p} + 0x8]",
            b = in(reg) b,
            mk = in(reg) crate::marks::store::base(),
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): `item` (esi), `colour` (edx).
pub unsafe fn draw_square(item: u32, colour: u32, shares: [[u8; 10]; 2]) {
    unsafe {
        let b = game_addr(0) as u32;
        let vertices = game_addr(VERTICES_RVA) as u32;
        let [a, pb, c, d]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, v)) in [(a, pb), (c, pb), (c, d), (a, d)].into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        shifted_places(item, vertices);
        corners(&shares);
        if game_read::<i32>(CORNERS_RVA + 8) <= FAR_ENOUGH {
            return;
        }
        projected(b, vertices);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, colour);
        }
        crate::marks::clip_quad::draw_fan();
    }
}
